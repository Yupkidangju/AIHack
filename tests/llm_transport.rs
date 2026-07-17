use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use aihack::{
    core::{GameSession, NarrativeTopic},
    llm::narrative::NarrativeRequest,
};
use aihack_llm::{
    config::{validate_user_text, LlmConfigError, LlmInputCode, LlmRequestKind, LocalLlmConfig},
    decision::DecisionRequest,
    service::{LlmPayload, LlmRequestInput, LocalLlmPort, LocalLlmService},
    soft_adjudication::SoftAdjudicationRequest,
    transport::{
        LlmResponseError, LlmValidationCode, OpenAiNarrativeTransport, RESPONSE_BODY_LIMIT,
    },
    worker::{LlmEnqueueError, NarrativeWorker, WORKER_CAPACITY},
    ClientRevision,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const LLM_ENV_NAMES: [&str; 7] = [
    "AIHACK_LLM_ENABLED",
    "AIHACK_LLM_ENDPOINT",
    "AIHACK_LLM_MODEL",
    "AIHACK_LLM_CONNECT_TIMEOUT_MS",
    "AIHACK_LLM_NARRATIVE_TIMEOUT_MS",
    "AIHACK_LLM_DECISION_TIMEOUT_MS",
    "AIHACK_LLM_MAX_CHARS",
];

struct EnvGuard(Vec<(String, Option<String>)>);

impl EnvGuard {
    fn clean() -> Self {
        let saved = LLM_ENV_NAMES
            .iter()
            .map(|name| ((*name).to_string(), std::env::var(name).ok()))
            .collect();
        for name in LLM_ENV_NAMES {
            std::env::remove_var(name);
        }
        Self(saved)
    }

    fn set(&self, name: &str, value: &str) {
        std::env::set_var(name, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (name, value) in &self.0 {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

#[test]
fn disabled_config_uses_safe_local_defaults() {
    let config = LocalLlmConfig::disabled();

    assert!(!config.enabled());
    assert_eq!(config.endpoint(), "http://127.0.0.1:11434/v1");
    assert_eq!(config.model(), "");
    assert_eq!(config.connect_timeout_ms(), 500);
    assert_eq!(config.request_timeout_ms(&LlmRequestKind::Narrative), 2_000);
    assert_eq!(config.request_timeout_ms(&LlmRequestKind::Decision), 1_500);
    assert_eq!(config.max_output_chars(), 240);
}

#[test]
fn enabled_config_accepts_only_explicit_loopback_values() {
    let _lock = ENV_LOCK.lock().unwrap();
    let env = EnvGuard::clean();
    env.set("AIHACK_LLM_ENABLED", "true");
    env.set("AIHACK_LLM_ENDPOINT", "http://localhost:11434/v1");
    env.set("AIHACK_LLM_MODEL", "local-model");

    let config = LocalLlmConfig::from_env().unwrap();

    assert!(config.enabled());
    assert_eq!(config.endpoint(), "http://localhost:11434/v1");
    assert_eq!(config.model(), "local-model");
}

#[test]
fn enabled_config_rejects_non_loopback_or_credentialed_endpoints() {
    let _lock = ENV_LOCK.lock().unwrap();
    let env = EnvGuard::clean();
    env.set("AIHACK_LLM_ENABLED", "true");
    env.set("AIHACK_LLM_MODEL", "local-model");

    for endpoint in [
        "http://192.0.2.1:11434/v1",
        "https://127.0.0.1:11434/v1",
        "http://user:pass@127.0.0.1:11434/v1",
        "http://127.0.0.1:11434/v1?token=x",
        "http://127.0.0.1:11434/v1#fragment",
    ] {
        env.set("AIHACK_LLM_ENDPOINT", endpoint);
        assert_eq!(
            LocalLlmConfig::from_env(),
            Err(LlmConfigError::InvalidEndpoint)
        );
    }
}

#[test]
fn config_reports_typed_boolean_model_and_range_errors() {
    let _lock = ENV_LOCK.lock().unwrap();
    let env = EnvGuard::clean();
    env.set("AIHACK_LLM_ENABLED", "yes");
    assert!(matches!(
        LocalLlmConfig::from_env(),
        Err(LlmConfigError::InvalidBoolean { .. })
    ));

    env.set("AIHACK_LLM_ENABLED", "true");
    assert_eq!(
        LocalLlmConfig::from_env(),
        Err(LlmConfigError::MissingModel)
    );

    env.set("AIHACK_LLM_MODEL", "local-model");
    env.set("AIHACK_LLM_CONNECT_TIMEOUT_MS", "99");
    assert!(matches!(
        LocalLlmConfig::from_env(),
        Err(LlmConfigError::InvalidRange { .. })
    ));
}

#[test]
fn soft_user_text_boundary_trims_and_rejects_empty_long_or_control_input() {
    assert_eq!(validate_user_text("  answer  "), Ok("answer".to_string()));
    assert_eq!(validate_user_text("   "), Err(LlmInputCode::EmptyUserText));
    assert_eq!(
        validate_user_text(&"가".repeat(241)),
        Err(LlmInputCode::TextTooLong)
    );
    for text in ["line\nfeed", "tab\ttext", "ansi\u{1b}[31m", "c1\u{85}text"] {
        assert_eq!(
            validate_user_text(text),
            Err(LlmInputCode::ControlCharacter)
        );
    }
}

fn enabled_config(address: SocketAddr, request_timeout_ms: u64) -> LocalLlmConfig {
    let _lock = ENV_LOCK.lock().unwrap();
    let env = EnvGuard::clean();
    env.set("AIHACK_LLM_ENABLED", "true");
    env.set("AIHACK_LLM_ENDPOINT", &format!("http://{address}/v1"));
    env.set("AIHACK_LLM_MODEL", "local-model");
    env.set(
        "AIHACK_LLM_NARRATIVE_TIMEOUT_MS",
        &request_timeout_ms.to_string(),
    );
    LocalLlmConfig::from_env().unwrap()
}

fn narrative_request() -> NarrativeRequest {
    let session = GameSession::new_for_playing(42);
    NarrativeRequest {
        revision: ClientRevision {
            turn: session.turn(),
            snapshot_hash: session.snapshot().stable_hash(),
        },
        topic: NarrativeTopic::SituationSummary,
        observation: session.observation(),
    }
}

fn spawn_http_server(
    status: &str,
    headers: &[(&str, &str)],
    body: String,
    delay: Duration,
) -> (SocketAddr, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let status = status.to_string();
    let headers = headers
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect::<Vec<_>>();
    let (request_tx, request_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        let mut expected_len = None;
        loop {
            let read = stream.read(&mut buffer).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if expected_len.is_none() {
                if let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let header_text = String::from_utf8_lossy(&request[..header_end]);
                    let content_length = header_text
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    expected_len = Some(header_end + 4 + content_length);
                }
            }
            if expected_len.is_some_and(|expected| request.len() >= expected) {
                break;
            }
        }
        let _ = request_tx.send(String::from_utf8_lossy(&request).into_owned());
        thread::sleep(delay);
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, value) in headers {
            response.push_str(&format!("{name}: {value}\r\n"));
        }
        response.push_str("\r\n");
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body.as_bytes());
    });
    (address, request_rx, handle)
}

#[test]
fn transport_posts_canonical_request_and_returns_valid_narrative() {
    let body = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"{\"kind\":\"NARRATIVE\",\"text\":\"A quiet corridor.\"}"},"finish_reason":"stop"}]}"#;
    let (address, request_rx, server) =
        spawn_http_server("200 OK", &[], body.to_string(), Duration::ZERO);
    let transport = OpenAiNarrativeTransport::new(enabled_config(address, 2_000)).unwrap();

    let text = transport.complete(&narrative_request()).unwrap();

    assert_eq!(text, "A quiet corridor.");
    let request = request_rx.recv().unwrap();
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    let (_, request_body) = request.split_once("\r\n\r\n").unwrap();
    let request_json: serde_json::Value = serde_json::from_str(request_body).unwrap();
    assert_eq!(request_json["model"], "local-model");
    assert!(request_json["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Never emit a state patch"));
    let canonical_input: serde_json::Value =
        serde_json::from_str(request_json["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(canonical_input["revision"]["turn"], 0);
    assert!(canonical_input["revision"].get("snapshot_hash").is_some());
    assert!(canonical_input.get("action_space").is_some());
    assert!(canonical_input["observation"]
        .get("legal_actions")
        .is_none());
    server.join().unwrap();
}

#[test]
fn transport_returns_strict_decision_and_soft_adjudication_payloads() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let revision = ClientRevision {
        turn: session.turn(),
        snapshot_hash: session.snapshot().stable_hash(),
    };

    let decision_body = r#"{"choices":[{"message":{"content":"{\"kind\":\"DECISION\",\"action\":{\"type\":\"WAIT\"},\"rationale\":\"Hold position.\",\"confidence\":0.75}"}}]}"#;
    let (decision_address, _, decision_server) =
        spawn_http_server("200 OK", &[], decision_body.to_string(), Duration::ZERO);
    let decision_transport =
        OpenAiNarrativeTransport::new(enabled_config(decision_address, 2_000)).unwrap();
    let decision = decision_transport
        .complete_decision(&DecisionRequest {
            revision: revision.clone(),
            observation: observation.clone(),
            action_space: observation.action_space.clone(),
        })
        .unwrap();
    assert_eq!(decision.rationale, "Hold position.");
    assert_eq!(decision.confidence, 0.75);
    decision_server.join().unwrap();

    let soft_body = r#"{"choices":[{"message":{"content":"{\"kind\":\"SOFT_ADJUDICATION\",\"verdict\":\"FAVORABLE\",\"reasonCode\":\"PLAUSIBLE\",\"message\":\"The attempt is plausible.\"}"}}]}"#;
    let (soft_address, _, soft_server) =
        spawn_http_server("200 OK", &[], soft_body.to_string(), Duration::ZERO);
    let soft_transport =
        OpenAiNarrativeTransport::new(enabled_config(soft_address, 2_000)).unwrap();
    let soft = soft_transport
        .complete_soft_adjudication(&SoftAdjudicationRequest {
            revision,
            observation,
            user_text: "I greet the guard.".to_string(),
        })
        .unwrap();
    assert_eq!(soft.reason_code, "PLAUSIBLE");
    assert_eq!(soft.message, "The attempt is plausible.");
    soft_server.join().unwrap();
}

#[test]
fn transport_rejects_decision_metadata_before_it_reaches_the_ui() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let body = r#"{"choices":[{"message":{"content":"{\"kind\":\"DECISION\",\"action\":{\"type\":\"WAIT\"},\"rationale\":\"Unsafe confidence.\",\"confidence\":1.5}"}}]}"#;
    let (address, _, server) = spawn_http_server("200 OK", &[], body.to_string(), Duration::ZERO);
    let transport = OpenAiNarrativeTransport::new(enabled_config(address, 2_000)).unwrap();

    let result = transport.complete_decision(&DecisionRequest {
        revision: ClientRevision {
            turn: session.turn(),
            snapshot_hash: session.snapshot().stable_hash(),
        },
        action_space: observation.action_space.clone(),
        observation,
    });

    assert_eq!(
        result,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::InvalidConfidence,
        })
    );
    server.join().unwrap();
}

#[test]
fn local_llm_service_round_trips_narrative_on_the_bounded_worker() {
    let body = r#"{"choices":[{"message":{"content":"{\"kind\":\"NARRATIVE\",\"text\":\"Worker narrative.\"}"}}]}"#;
    let (address, _, server) =
        spawn_http_server("200 OK", &[], body.to_string(), Duration::from_millis(100));
    let mut service = LocalLlmService::from_config(enabled_config(address, 2_000)).unwrap();
    let request = narrative_request();
    let request_id = service
        .enqueue(LlmRequestInput {
            revision: request.revision.clone(),
            observation: request.observation,
            kind: LlmRequestKind::Narrative,
        })
        .unwrap();
    let duplicate = narrative_request();
    assert_eq!(
        service.enqueue(LlmRequestInput {
            revision: duplicate.revision,
            observation: duplicate.observation,
            kind: LlmRequestKind::Narrative,
        }),
        Err(LlmEnqueueError::Busy {
            capacity: WORKER_CAPACITY as u16,
        })
    );

    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    let envelope = loop {
        if let Some(envelope) = service.try_recv() {
            break envelope;
        }
        assert!(std::time::Instant::now() < deadline);
        thread::yield_now();
    };
    assert_eq!(envelope.request_id, request_id);
    assert!(matches!(
        envelope.result,
        Ok(LlmPayload::Narrative(ref payload)) if payload.text == "Worker narrative."
    ));
    assert!(service.shutdown_with_grace(Duration::from_millis(250)));
    server.join().unwrap();
}

#[test]
fn local_llm_service_disabled_mode_rejects_without_a_request_id() {
    let service = LocalLlmService::disabled();
    let request = narrative_request();
    assert_eq!(
        service.enqueue(LlmRequestInput {
            revision: request.revision,
            observation: request.observation,
            kind: LlmRequestKind::Narrative,
        }),
        Err(LlmEnqueueError::Disabled)
    );
}

#[test]
fn transport_classifies_unavailable_timeout_and_redirect_without_following() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let unavailable_address = listener.local_addr().unwrap();
    drop(listener);
    let unavailable = OpenAiNarrativeTransport::new(enabled_config(unavailable_address, 500))
        .unwrap()
        .complete(&narrative_request());
    assert_eq!(unavailable, Err(LlmResponseError::Unavailable));

    let timeout_body = r#"{"choices":[]}"#.to_string();
    let (timeout_address, _, timeout_server) =
        spawn_http_server("200 OK", &[], timeout_body, Duration::from_millis(250));
    let timeout = OpenAiNarrativeTransport::new(enabled_config(timeout_address, 100))
        .unwrap()
        .complete(&narrative_request());
    assert_eq!(timeout, Err(LlmResponseError::Timeout));
    timeout_server.join().unwrap();

    let (redirect_address, _, redirect_server) = spawn_http_server(
        "302 Found",
        &[("Location", "http://192.0.2.1/escape")],
        String::new(),
        Duration::ZERO,
    );
    let redirect = OpenAiNarrativeTransport::new(enabled_config(redirect_address, 500))
        .unwrap()
        .complete(&narrative_request());
    assert_eq!(redirect, Err(LlmResponseError::HttpStatus { code: 302 }));
    redirect_server.join().unwrap();
}

#[test]
fn transport_rejects_invalid_empty_unknown_and_oversized_responses() {
    let cases = [
        (
            "not-json".to_string(),
            LlmResponseError::InvalidSchema {
                code: LlmValidationCode::InvalidJson,
            },
        ),
        (
            r#"{"choices":[{"message":{"content":"{\"kind\":\"NARRATIVE\",\"text\":\"   \"}"}}]}"#.to_string(),
            LlmResponseError::InvalidSchema {
                code: LlmValidationCode::EmptyText,
            },
        ),
        (
            r#"{"choices":[{"message":{"content":"{\"kind\":\"NARRATIVE\",\"text\":\"ok\",\"unknown\":true}"}}]}"#.to_string(),
            LlmResponseError::InvalidSchema {
                code: LlmValidationCode::InvalidJson,
            },
        ),
        (
            "x".repeat(RESPONSE_BODY_LIMIT + 1),
            LlmResponseError::BodyTooLarge {
                limit_bytes: RESPONSE_BODY_LIMIT,
            },
        ),
    ];

    for (body, expected) in cases {
        let (address, _, server) = spawn_http_server("200 OK", &[], body, Duration::ZERO);
        let result = OpenAiNarrativeTransport::new(enabled_config(address, 500))
            .unwrap()
            .complete(&narrative_request());
        assert_eq!(result, Err(expected));
        server.join().unwrap();
    }
}

#[test]
fn transport_rejects_request_payloads_over_the_canonical_limit() {
    let mut request = narrative_request();
    let tile = request.observation.visible_tiles[0].clone();
    request.observation.visible_tiles = vec![tile; 4_000];
    let config = enabled_config("127.0.0.1:9".parse().unwrap(), 500);
    let result = OpenAiNarrativeTransport::new(config)
        .unwrap()
        .complete(&request);

    assert_eq!(
        result,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::PayloadTooLarge,
        })
    );
}

struct BlockingProvider {
    entered: mpsc::SyncSender<()>,
    gate: Arc<(Mutex<bool>, Condvar)>,
}

impl aihack_llm::narrative::NarrativeProvider for BlockingProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, aihack_llm::narrative::NarrativeError> {
        let _ = self.entered.try_send(());
        let (lock, ready) = &*self.gate;
        let mut released = lock.lock().unwrap();
        while !*released {
            released = ready.wait(released).unwrap();
        }
        Ok("worker response".to_string())
    }
}

#[test]
fn worker_is_disabled_without_a_provider() {
    let worker = NarrativeWorker::disabled();

    assert_eq!(
        worker.enqueue(narrative_request()),
        Err(LlmEnqueueError::Disabled)
    );
    assert!(worker.try_recv().is_none());
}

#[test]
fn worker_uses_one_thread_and_reports_busy_at_capacity_sixteen() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let provider = Arc::new(BlockingProvider {
        entered: entered_tx,
        gate: Arc::clone(&gate),
    });
    let worker = NarrativeWorker::start(provider).unwrap();

    worker.enqueue(narrative_request()).unwrap();
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    for _ in 0..WORKER_CAPACITY {
        worker.enqueue(narrative_request()).unwrap();
    }
    assert_eq!(
        worker.enqueue(narrative_request()),
        Err(LlmEnqueueError::Busy {
            capacity: WORKER_CAPACITY as u16,
        })
    );

    let (lock, ready) = &*gate;
    *lock.lock().unwrap() = true;
    ready.notify_all();
}

#[test]
fn worker_round_trips_an_opaque_request_id() {
    let worker = NarrativeWorker::start(Arc::new(SuccessProvider)).unwrap();
    let request_id = worker.enqueue(narrative_request()).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    let response = loop {
        if let Some(response) = worker.try_recv() {
            break response;
        }
        assert!(std::time::Instant::now() < deadline);
        thread::yield_now();
    };

    assert_eq!(response.request_id, request_id);
    assert_eq!(response.response.text, "worker response");
    assert!(!request_id.as_str().is_empty());
}

#[test]
fn worker_shutdown_stops_waiting_when_the_grace_period_expires() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let provider = Arc::new(BlockingProvider {
        entered: entered_tx,
        gate: Arc::clone(&gate),
    });
    let mut worker = NarrativeWorker::start(provider).unwrap();
    worker.enqueue(narrative_request()).unwrap();
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();

    let started = std::time::Instant::now();
    assert!(!worker.shutdown_with_grace(Duration::from_millis(10)));
    assert!(started.elapsed() < Duration::from_millis(250));

    let (lock, ready) = &*gate;
    *lock.lock().unwrap() = true;
    ready.notify_all();
}

struct SuccessProvider;

impl aihack_llm::narrative::NarrativeProvider for SuccessProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, aihack_llm::narrative::NarrativeError> {
        Ok("worker response".to_string())
    }
}
