use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use aihack_ai_contract::{llm::NarrativePayload, ClientRevision, NarrativeTopic, Observation};

use crate::{
    config::{validate_user_text, LlmRequestKind, LocalLlmConfig},
    decision::DecisionRequest,
    narrative::NarrativeRequest,
    soft_adjudication::SoftAdjudicationRequest,
    transport::{LlmResponseError, OpenAiNarrativeTransport},
    worker::{LlmEnqueueError, RequestId, WORKER_CAPACITY},
};

pub use aihack_ai_contract::llm::LlmPayload;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmRequestInput {
    pub revision: ClientRevision,
    pub observation: Observation,
    pub kind: LlmRequestKind,
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

struct WorkerRequest {
    request_id: RequestId,
    input: LlmRequestInput,
}

pub struct LocalLlmService {
    request_tx: Option<SyncSender<WorkerRequest>>,
    response_queue: Arc<Mutex<VecDeque<LlmResponseEnvelope>>>,
    handle: Option<JoinHandle<()>>,
    done_rx: Option<Receiver<()>>,
    outstanding: Arc<Mutex<[bool; 3]>>,
}

impl LocalLlmService {
    pub fn disabled() -> Self {
        Self {
            request_tx: None,
            response_queue: Arc::new(Mutex::new(VecDeque::new())),
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
        let response_queue = Arc::new(Mutex::new(VecDeque::new()));
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
}

impl LocalLlmPort for LocalLlmService {
    fn enqueue(&self, mut input: LlmRequestInput) -> Result<RequestId, LlmEnqueueError> {
        let request_tx = self.request_tx.as_ref().ok_or(LlmEnqueueError::Disabled)?;
        if let LlmRequestKind::SoftAdjudication { user_text } = &mut input.kind {
            *user_text = validate_user_text(user_text)
                .map_err(|code| LlmEnqueueError::InvalidInput { code })?;
        }
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
        self.response_queue.lock().ok()?.pop_front()
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
    response_queue: Arc<Mutex<VecDeque<LlmResponseEnvelope>>>,
    outstanding: Arc<Mutex<[bool; 3]>>,
) {
    while let Ok(request) = request_rx.recv() {
        let kind_index = request_kind_index(&request.input.kind);
        let revision = request.input.revision.clone();
        let result = match request.input.kind {
            LlmRequestKind::Narrative => transport
                .complete(&NarrativeRequest {
                    revision: revision.clone(),
                    topic: NarrativeTopic::SituationSummary,
                    observation: request.input.observation,
                })
                .map(|text| LlmPayload::Narrative(NarrativePayload { text })),
            LlmRequestKind::Decision => {
                let action_space = request.input.observation.action_space.clone();
                transport
                    .complete_decision(&DecisionRequest {
                        revision: revision.clone(),
                        observation: request.input.observation,
                        action_space,
                    })
                    .map(LlmPayload::Decision)
            }
            LlmRequestKind::SoftAdjudication { user_text } => transport
                .complete_soft_adjudication(&SoftAdjudicationRequest {
                    revision: revision.clone(),
                    observation: request.input.observation,
                    user_text,
                })
                .map(LlmPayload::SoftAdjudication),
        };
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

fn push_response(
    response_queue: &Mutex<VecDeque<LlmResponseEnvelope>>,
    response: LlmResponseEnvelope,
) {
    if let Ok(mut queue) = response_queue.lock() {
        if queue.len() == WORKER_CAPACITY {
            queue.pop_front();
        }
        queue.push_back(response);
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
    use aihack_ai_contract::SnapshotHash;

    #[test]
    fn response_queue_drops_the_oldest_presentation_when_full() {
        let queue = Mutex::new(VecDeque::new());
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

        let queue = queue.lock().unwrap();
        assert_eq!(queue.len(), WORKER_CAPACITY);
        assert_eq!(queue.front().unwrap().revision.turn, 1);
        assert_eq!(queue.back().unwrap().revision.turn, WORKER_CAPACITY as u64);
    }
}
