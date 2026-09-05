use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use uuid::Uuid;

use crate::{
    config::LocalLlmConfig,
    narrative::{request_narrative, NarrativeProvider, NarrativeRequest, NarrativeResponse},
    transport::{LlmResponseError, OpenAiNarrativeTransport},
};

pub const WORKER_CAPACITY: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmEnqueueError {
    Disabled,
    Busy { capacity: u16 },
    InvalidEndpoint,
    InvalidModel,
    UnsupportedSchema { expected: u16, actual: u16 },
    InvalidInput { code: crate::config::LlmInputCode },
    WorkerStopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeResponseEnvelope {
    pub request_id: RequestId,
    pub response: NarrativeResponse,
}

struct WorkerRequest {
    request_id: RequestId,
    request: NarrativeRequest,
}

pub struct NarrativeWorker {
    request_tx: Option<SyncSender<WorkerRequest>>,
    response_rx: Option<Receiver<NarrativeResponseEnvelope>>,
    handle: Option<JoinHandle<()>>,
    done_rx: Option<Receiver<()>>,
}

impl NarrativeWorker {
    pub fn disabled() -> Self {
        let (_response_tx, response_rx) = mpsc::sync_channel(WORKER_CAPACITY);
        Self {
            request_tx: None,
            response_rx: Some(response_rx),
            handle: None,
            done_rx: None,
        }
    }

    pub fn from_config(config: LocalLlmConfig) -> Result<Self, LlmResponseError> {
        if !config.enabled() {
            return Ok(Self::disabled());
        }
        let provider = Arc::new(OpenAiNarrativeTransport::new(config)?);
        Self::start(provider).map_err(|_| LlmResponseError::Unavailable)
    }

    pub fn start(provider: Arc<dyn NarrativeProvider>) -> Result<Self, LlmEnqueueError> {
        let (request_tx, request_rx) = mpsc::sync_channel::<WorkerRequest>(WORKER_CAPACITY);
        let (response_tx, response_rx) =
            mpsc::sync_channel::<NarrativeResponseEnvelope>(WORKER_CAPACITY);
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name("aihack-llm".to_string())
            .spawn(move || {
                run_worker(provider, request_rx, response_tx);
                let _ = done_tx.try_send(());
            })
            .map_err(|_| LlmEnqueueError::WorkerStopped)?;
        Ok(Self {
            request_tx: Some(request_tx),
            response_rx: Some(response_rx),
            handle: Some(handle),
            done_rx: Some(done_rx),
        })
    }

    pub fn enqueue(&self, request: NarrativeRequest) -> Result<RequestId, LlmEnqueueError> {
        let request_tx = self.request_tx.as_ref().ok_or(LlmEnqueueError::Disabled)?;
        let request_id = RequestId::new();
        match request_tx.try_send(WorkerRequest {
            request_id: request_id.clone(),
            request,
        }) {
            Ok(()) => Ok(request_id),
            Err(TrySendError::Full(_)) => Err(LlmEnqueueError::Busy {
                capacity: WORKER_CAPACITY as u16,
            }),
            Err(TrySendError::Disconnected(_)) => Err(LlmEnqueueError::WorkerStopped),
        }
    }

    pub fn try_recv(&self) -> Option<NarrativeResponseEnvelope> {
        self.response_rx.as_ref()?.try_recv().ok()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<NarrativeResponseEnvelope> {
        self.response_rx.as_ref()?.recv_timeout(timeout).ok()
    }

    /// 요청 송신을 닫고 지정된 grace 동안만 worker 종료를 기다린다.
    /// 기한을 넘긴 worker는 join handle을 분리해 앱 종료를 막지 않는다.
    pub fn shutdown_with_grace(&mut self, grace: Duration) -> bool {
        self.request_tx.take();
        self.response_rx.take();
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

impl Drop for NarrativeWorker {
    fn drop(&mut self) {
        let _ = self.shutdown_with_grace(Duration::from_millis(250));
    }
}

fn run_worker(
    provider: Arc<dyn NarrativeProvider>,
    request_rx: Receiver<WorkerRequest>,
    response_tx: SyncSender<NarrativeResponseEnvelope>,
) {
    while let Ok(request) = request_rx.recv() {
        let response = request_narrative(Some(Arc::clone(&provider)), request.request);
        if response_tx
            .send(NarrativeResponseEnvelope {
                request_id: request.request_id,
                response,
            })
            .is_err()
        {
            break;
        }
    }
}
