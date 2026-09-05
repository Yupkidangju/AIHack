use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use aihack_ai_contract::{
    ActionSpace, ClientRevision, EntityObservation, GameEvent, ItemObservation, Observation,
    PlayerObservation, RunStateSummary, TileObservation,
};
use serde::Serialize;

use crate::{
    config::{validate_user_text, LlmRequestKind, LocalLlmConfig},
    transport::{LlmResponseError, OpenAiNarrativeTransport, REQUEST_BODY_LIMIT},
    worker::{LlmEnqueueError, RequestId, WORKER_CAPACITY},
};

pub use aihack_ai_contract::llm::LlmPayload;

pub const LLM_SCHEMA_VERSION: u16 = 1;
pub type SessionRevision = ClientRevision;
pub type VisibleTile = TileObservation;
pub type VisibleEntity = EntityObservation;
pub type InventoryObservation = ItemObservation;
pub type GameEventSummary = GameEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LlmObservationView {
    pub turn: u64,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub visible_tiles: Vec<VisibleTile>,
    pub visible_entities: Vec<VisibleEntity>,
    pub inventory: Vec<InventoryObservation>,
    pub last_events: Vec<GameEventSummary>,
}

impl From<&Observation> for LlmObservationView {
    fn from(observation: &Observation) -> Self {
        Self {
            turn: observation.turn,
            run_state: observation.run_state,
            player: observation.player.clone(),
            visible_tiles: observation.visible_tiles.clone(),
            visible_entities: observation.visible_entities.clone(),
            inventory: observation.inventory.clone(),
            last_events: observation.last_events.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmRequestInput {
    pub schema_version: u16,
    pub revision: SessionRevision,
    pub observation: LlmObservationView,
    pub action_space: ActionSpace,
    pub kind: LlmRequestKind,
}

impl LlmRequestInput {
    pub fn from_observation(
        revision: ClientRevision,
        observation: &Observation,
        kind: LlmRequestKind,
    ) -> Self {
        Self {
            schema_version: observation.schema_version,
            revision,
            observation: LlmObservationView::from(observation),
            action_space: observation.action_space.clone(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmResponseEnvelope {
    pub schema_version: u16,
    pub request_id: RequestId,
    pub revision: ClientRevision,
    pub result: Result<LlmPayload, LlmResponseError>,
}

pub trait LocalLlmPort {
    fn enqueue(&self, input: LlmRequestInput) -> Result<RequestId, LlmEnqueueError>;
    fn try_recv(&self) -> Option<LlmResponseEnvelope>;
}

#[derive(Default)]
struct ResponseQueue {
    entries: Mutex<VecDeque<LlmResponseEnvelope>>,
    ready: Condvar,
}

struct WorkerRequest {
    request_id: RequestId,
    input: LlmRequestInput,
}

pub struct LocalLlmService {
    request_tx: Option<SyncSender<WorkerRequest>>,
    response_queue: Arc<ResponseQueue>,
    handle: Option<JoinHandle<()>>,
    done_rx: Option<Receiver<()>>,
    outstanding: Arc<Mutex<[bool; 3]>>,
}

impl LocalLlmService {
    pub fn disabled() -> Self {
        Self {
            request_tx: None,
            response_queue: Arc::new(ResponseQueue::default()),
            handle: None,
            done_rx: None,
            outstanding: Arc::new(Mutex::new([false; 3])),
        }
    }

    pub fn from_config(config: LocalLlmConfig) -> Result<Self, LlmResponseError> {
        if !config.enabled() {
            return Ok(Self::disabled());
        }
        let transport = OpenAiNarrativeTransport::new(config)?;
        let (request_tx, request_rx) = mpsc::sync_channel(WORKER_CAPACITY);
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let response_queue = Arc::new(ResponseQueue::default());
        let worker_response_queue = Arc::clone(&response_queue);
        let outstanding = Arc::new(Mutex::new([false; 3]));
        let worker_outstanding = Arc::clone(&outstanding);
        let handle = thread::Builder::new()
            .name("aihack-llm".to_string())
            .spawn(move || {
                run_worker(
                    transport,
                    request_rx,
                    worker_response_queue,
                    worker_outstanding,
                );
                let _ = done_tx.try_send(());
            })
            .map_err(|_| LlmResponseError::Unavailable)?;
        Ok(Self {
            request_tx: Some(request_tx),
            response_queue,
            handle: Some(handle),
            done_rx: Some(done_rx),
            outstanding,
        })
    }

    pub fn shutdown_with_grace(&mut self, grace: Duration) -> bool {
        self.request_tx.take();
        let Some(handle) = self.handle.take() else {
            self.done_rx.take();
            return true;
        };
        let finished = self
            .done_rx
            .take()
            .is_some_and(|done_rx| done_rx.recv_timeout(grace).is_ok());
        if finished {
            let _ = handle.join();
        }
        finished
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<LlmResponseEnvelope> {
        let queue = self.response_queue.entries.lock().ok()?;
        let (mut queue, _) = self
            .response_queue
            .ready
            .wait_timeout_while(queue, timeout, |entries| entries.is_empty())
            .ok()?;
        queue.pop_front()
    }

    pub fn wait_for_response(&self, timeout: Duration) -> bool {
        let Ok(queue) = self.response_queue.entries.lock() else {
            return false;
        };
        let Ok((queue, _)) =
            self.response_queue
                .ready
                .wait_timeout_while(queue, timeout, |entries| entries.is_empty())
        else {
            return false;
        };
        !queue.is_empty()
    }
}

impl LocalLlmPort for LocalLlmService {
    fn enqueue(&self, mut input: LlmRequestInput) -> Result<RequestId, LlmEnqueueError> {
        let request_tx = self.request_tx.as_ref().ok_or(LlmEnqueueError::Disabled)?;
        prepare_request_input(&mut input)?;
        let kind_index = request_kind_index(&input.kind);
        {
            let mut outstanding = self
                .outstanding
                .lock()
                .map_err(|_| LlmEnqueueError::WorkerStopped)?;
            if outstanding[kind_index] {
                return Err(LlmEnqueueError::Busy {
                    capacity: WORKER_CAPACITY as u16,
                });
            }
            outstanding[kind_index] = true;
        }
        let request_id = RequestId::new();
        if let Err(error) = request_tx.try_send(WorkerRequest {
            request_id: request_id.clone(),
            input,
        }) {
            if let Ok(mut outstanding) = self.outstanding.lock() {
                outstanding[kind_index] = false;
            }
            return Err(match error {
                TrySendError::Full(_) => LlmEnqueueError::Busy {
                    capacity: WORKER_CAPACITY as u16,
                },
                TrySendError::Disconnected(_) => LlmEnqueueError::WorkerStopped,
            });
        }
        Ok(request_id)
    }

    fn try_recv(&self) -> Option<LlmResponseEnvelope> {
        self.response_queue.entries.lock().ok()?.pop_front()
    }
}

impl Drop for LocalLlmService {
    fn drop(&mut self) {
        let _ = self.shutdown_with_grace(Duration::from_millis(250));
    }
}

fn run_worker(
    transport: OpenAiNarrativeTransport,
    request_rx: Receiver<WorkerRequest>,
    response_queue: Arc<ResponseQueue>,
    outstanding: Arc<Mutex<[bool; 3]>>,
) {
    while let Ok(request) = request_rx.recv() {
        let kind_index = request_kind_index(&request.input.kind);
        let revision = request.input.revision.clone();
        let result = transport.complete_input(&request.input);
        if let Ok(mut outstanding) = outstanding.lock() {
            outstanding[kind_index] = false;
        }
        push_response(
            &response_queue,
            LlmResponseEnvelope {
                schema_version: 1,
                request_id: request.request_id,
                revision,
                result,
            },
        );
    }
}

pub fn validate_response_schema(schema_version: u16) -> Result<(), LlmResponseError> {
    if schema_version == LLM_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(LlmResponseError::UnsupportedSchema {
            expected: LLM_SCHEMA_VERSION,
            actual: schema_version,
        })
    }
}

pub(crate) fn prepare_request_input(input: &mut LlmRequestInput) -> Result<(), LlmEnqueueError> {
    if input.schema_version != LLM_SCHEMA_VERSION {
        return Err(LlmEnqueueError::UnsupportedSchema {
            expected: LLM_SCHEMA_VERSION,
            actual: input.schema_version,
        });
    }
    if input.observation.visible_tiles.len() > 800
        || input.observation.visible_entities.len() > 128
        || input.observation.inventory.len() > 52
        || input.observation.last_events.len() > 20
        || input.action_space.commands.len() > 64
    {
        return Err(LlmEnqueueError::InvalidInput {
            code: crate::config::LlmInputCode::PayloadTooLarge,
        });
    }
    if let LlmRequestKind::SoftAdjudication { user_text } = &mut input.kind {
        *user_text =
            validate_user_text(user_text).map_err(|code| LlmEnqueueError::InvalidInput { code })?;
    }
    if canonical_request_json(input)
        .map_err(|_| LlmEnqueueError::InvalidInput {
            code: crate::config::LlmInputCode::PayloadTooLarge,
        })?
        .len()
        > REQUEST_BODY_LIMIT
    {
        return Err(LlmEnqueueError::InvalidInput {
            code: crate::config::LlmInputCode::PayloadTooLarge,
        });
    }
    Ok(())
}

pub(crate) fn canonical_request_json(input: &LlmRequestInput) -> serde_json::Result<String> {
    let (kind, user_text) = match &input.kind {
        LlmRequestKind::Narrative => (WireRequestKind::Narrative, None),
        LlmRequestKind::Decision => (WireRequestKind::Decision, None),
        LlmRequestKind::SoftAdjudication { user_text } => {
            (WireRequestKind::SoftAdjudication, Some(user_text.as_str()))
        }
    };
    serde_json::to_string(&LlmWireInput {
        schema_version: input.schema_version,
        revision: &input.revision,
        kind,
        observation: &input.observation,
        action_space: &input.action_space,
        user_text,
    })
}

#[derive(Serialize)]
struct LlmWireInput<'a> {
    schema_version: u16,
    revision: &'a ClientRevision,
    kind: WireRequestKind,
    observation: &'a LlmObservationView,
    action_space: &'a ActionSpace,
    #[serde(rename = "userText", skip_serializing_if = "Option::is_none")]
    user_text: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum WireRequestKind {
    Narrative,
    Decision,
    SoftAdjudication,
}

fn push_response(response_queue: &ResponseQueue, response: LlmResponseEnvelope) {
    if let Ok(mut queue) = response_queue.entries.lock() {
        if queue.len() == WORKER_CAPACITY {
            queue.pop_front();
        }
        queue.push_back(response);
        response_queue.ready.notify_one();
    }
}

fn request_kind_index(kind: &LlmRequestKind) -> usize {
    match kind {
        LlmRequestKind::Narrative => 0,
        LlmRequestKind::Decision => 1,
        LlmRequestKind::SoftAdjudication { .. } => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aihack_ai_contract::llm::NarrativePayload;
    use aihack_ai_contract::SnapshotHash;

    #[test]
    fn response_queue_drops_the_oldest_presentation_when_full() {
        let queue = ResponseQueue::default();
        for turn in 0..=WORKER_CAPACITY as u64 {
            push_response(
                &queue,
                LlmResponseEnvelope {
                    schema_version: 1,
                    request_id: RequestId::new(),
                    revision: ClientRevision {
                        turn,
                        snapshot_hash: SnapshotHash(format!("hash-{turn}")),
                    },
                    result: Ok(LlmPayload::Narrative(NarrativePayload {
                        text: format!("response-{turn}"),
                    })),
                },
            );
        }

        let queue = queue.entries.lock().unwrap();
        assert_eq!(queue.len(), WORKER_CAPACITY);
        assert_eq!(queue.front().unwrap().revision.turn, 1);
        assert_eq!(queue.back().unwrap().revision.turn, WORKER_CAPACITY as u64);
    }
}
