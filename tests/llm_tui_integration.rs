use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use aihack::{
    core::{CommandIntent, GameSession},
    ui::tui::{key_to_candidate, LlmUiStatus, TuiApp, UiCommandCandidate, UiRuntimeConfig},
};
use aihack_llm::config::LlmRequestKind;
use aihack_llm::{
    config::LocalLlmConfig,
    service::{LocalLlmPort, LocalLlmService},
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn unused() -> &'static Path {
    Path::new("/tmp/aihack-unused.json")
}

#[test]
fn uppercase_llm_ctas_do_not_replace_lowercase_roguelike_commands() {
    let observation = GameSession::new_for_playing(42).observation();

    assert_eq!(
        key_to_candidate('G', &observation),
        Some(UiCommandCandidate::LlmNarrative)
    );
    assert_eq!(
        key_to_candidate('A', &observation),
        Some(UiCommandCandidate::LlmSuggest)
    );
    assert_eq!(
        key_to_candidate('J', &observation),
        Some(UiCommandCandidate::LlmJudge)
    );
    assert_ne!(
        key_to_candidate('j', &observation),
        Some(UiCommandCandidate::LlmJudge)
    );
}

#[test]
fn disabled_llm_cta_is_typed_and_does_not_mutate_core() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before = app.revision();

    app.handle_candidate(UiCommandCandidate::LlmNarrative, unused(), unused())
        .unwrap();

    assert_eq!(app.llm_status(), &LlmUiStatus::Disabled);
    assert_eq!(app.revision(), before);
    assert!(app.take_llm_request().is_none());
}

#[test]
fn judge_modal_bounds_unicode_input_and_queues_only_valid_trimmed_text() {
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    let before = app.revision();
    app.handle_candidate(UiCommandCandidate::LlmJudge, unused(), unused())
        .unwrap();
    assert_eq!(app.soft_input(), Some(""));

    for _ in 0..241 {
        app.handle_candidate(UiCommandCandidate::LlmInput('가'), unused(), unused())
            .unwrap();
    }
    assert_eq!(app.soft_input().unwrap().chars().count(), 240);
    app.handle_candidate(UiCommandCandidate::LlmBackspace, unused(), unused())
        .unwrap();
    app.handle_candidate(UiCommandCandidate::LlmInput('!'), unused(), unused())
        .unwrap();
    app.handle_candidate(UiCommandCandidate::LlmSubmitInput, unused(), unused())
        .unwrap();

    let request = app.take_llm_request().unwrap();
    assert!(matches!(
        request,
        LlmRequestKind::SoftAdjudication { ref user_text }
            if user_text.chars().count() == 240 && user_text.ends_with('!')
    ));
    assert_eq!(app.soft_input(), None);
    assert_eq!(app.revision(), before);
}

#[test]
fn empty_judge_submission_stays_in_the_modal_without_a_request() {
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    app.handle_candidate(UiCommandCandidate::LlmJudge, unused(), unused())
        .unwrap();
    app.handle_candidate(UiCommandCandidate::LlmInput(' '), unused(), unused())
        .unwrap();
    app.handle_candidate(UiCommandCandidate::LlmSubmitInput, unused(), unused())
        .unwrap();

    assert_eq!(app.soft_input(), Some(" "));
    assert_eq!(app.llm_status(), &LlmUiStatus::Invalid);
    assert!(app.take_llm_request().is_none());
}

#[test]
fn status_badges_and_modal_instructions_are_textual() {
    let disabled = aihack::ui::tui::render_panels::llm_status_lines(&LlmUiStatus::Disabled);
    assert!(disabled[0].contains("LLM: OFF"));
    assert!(disabled[1].contains("core play is available"));

    let pending = aihack::ui::tui::render_panels::llm_status_lines(&LlmUiStatus::Pending {
        kind: LlmRequestKind::Narrative,
        request_id: "12345678-rest-hidden".to_string(),
    });
    assert!(pending[0].contains("LLM: WAIT"));
    assert!(pending[1].contains("12345678"));
    assert!(!pending[1].contains("rest-hidden"));

    let modal = aihack::ui::tui::render_panels::soft_input_lines("plausible attempt");
    assert!(modal.iter().any(|line| line.contains("17/240")));
    assert!(modal.iter().any(|line| line.contains("Enter")));
    assert!(modal.iter().any(|line| line.contains("Esc")));
}

#[test]
fn displayed_llm_footer_ctas_have_the_same_mouse_candidates() {
    use aihack::ui::tui::{compute_layout, llm_footer_click_candidate};

    let layout = compute_layout(100, 32);
    let ready = aihack::ui::tui::render_panels::llm_footer_line(&LlmUiStatus::Ready, false, false);
    for (label, expected) in [
        ("[G] Narrative", UiCommandCandidate::LlmNarrative),
        ("[A] Suggest", UiCommandCandidate::LlmSuggest),
        ("[J] Judge", UiCommandCandidate::LlmJudge),
    ] {
        let offset = ready.find(label).unwrap() as u16;
        assert_eq!(
            llm_footer_click_candidate(
                layout.command,
                layout.command.x + offset + label.len() as u16 - 1,
                layout.command.y + 2,
                &ready,
            ),
            Some(expected)
        );
    }

    let result = aihack::ui::tui::render_panels::llm_footer_line(&LlmUiStatus::Ready, true, true);
    assert!(result.contains("[Y] Apply"));
    assert!(result.contains("[N] Dismiss"));

    let failed =
        aihack::ui::tui::render_panels::llm_footer_line(&LlmUiStatus::Unavailable, false, true);
    assert!(failed.contains("[R] Retry"));
    assert!(failed.contains("[N] Dismiss"));
}

#[test]
fn live_decision_waits_for_explicit_apply_and_soft_verdict_never_submits() {
    let _lock = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let decision_body = r#"{"choices":[{"message":{"content":"{\"kind\":\"DECISION\",\"action\":{\"type\":\"WAIT\"},\"rationale\":\"Hold position.\",\"confidence\":0.75}"}}]}"#;
    let (mut decision_service, decision_server) = service_for_response(decision_body);
    let mut decision_app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    decision_app
        .handle_candidate(UiCommandCandidate::LlmSuggest, unused(), unused())
        .unwrap();
    decision_app.dispatch_llm_request(&decision_service);
    poll_until_ready(&mut decision_app, &decision_service);
    assert_eq!(decision_app.observation().turn, 0);
    assert!(decision_app.decision_lines()[0].contains("Provider"));

    decision_app
        .handle_candidate(UiCommandCandidate::LlmApply, unused(), unused())
        .unwrap();
    assert_eq!(decision_app.observation().turn, 1);
    assert!(decision_service.shutdown_with_grace(Duration::from_millis(250)));
    decision_server.join().unwrap();

    let soft_body = r#"{"choices":[{"message":{"content":"{\"kind\":\"SOFT_ADJUDICATION\",\"verdict\":\"NEUTRAL\",\"reasonCode\":\"SOCIAL_UNCERTAIN\",\"message\":\"Plausible, with no core effect.\"}"}}]}"#;
    let (mut soft_service, soft_server) = service_for_response(soft_body);
    let mut soft_app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    let before = soft_app.revision();
    soft_app
        .handle_candidate(UiCommandCandidate::LlmJudge, unused(), unused())
        .unwrap();
    for character in "I greet the guard.".chars() {
        soft_app
            .handle_candidate(UiCommandCandidate::LlmInput(character), unused(), unused())
            .unwrap();
    }
    soft_app
        .handle_candidate(UiCommandCandidate::LlmSubmitInput, unused(), unused())
        .unwrap();
    soft_app.dispatch_llm_request(&soft_service);
    poll_until_ready(&mut soft_app, &soft_service);
    assert_eq!(soft_app.revision(), before);
    assert!(soft_app.soft_adjudication_lines()[0].contains("SOCIAL_UNCERTAIN"));
    assert!(soft_service.shutdown_with_grace(Duration::from_millis(250)));
    soft_server.join().unwrap();
}

#[test]
fn response_becomes_stale_when_core_advances_before_tui_acceptance() {
    let _lock = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let body = r#"{"choices":[{"message":{"content":"{\"kind\":\"DECISION\",\"action\":{\"type\":\"WAIT\"},\"rationale\":\"Hold.\",\"confidence\":0.5}"}}]}"#;
    let (mut service, server) = service_for_response(body);
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    app.handle_candidate(UiCommandCandidate::LlmSuggest, unused(), unused())
        .unwrap();
    app.dispatch_llm_request(&service);
    app.handle_candidate(
        UiCommandCandidate::Command(CommandIntent::Wait),
        unused(),
        unused(),
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(1);
    while matches!(app.llm_status(), LlmUiStatus::Pending { .. }) {
        app.poll_llm_response(&service);
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert_eq!(app.llm_status(), &LlmUiStatus::Stale);
    app.handle_candidate(UiCommandCandidate::LlmApply, unused(), unused())
        .unwrap();
    assert_eq!(app.observation().turn, 1);
    assert!(service.shutdown_with_grace(Duration::from_millis(250)));
    server.join().unwrap();
}

#[test]
fn unsupported_response_schema_is_rejected_before_tui_payload_acceptance() {
    let _lock = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let body = r#"{"choices":[{"message":{"content":"{\"kind\":\"NARRATIVE\",\"text\":\"Must not render.\"}"}}]}"#;
    let (mut service, server) = service_for_response(body);
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    let before = app.revision();
    app.handle_candidate(UiCommandCandidate::LlmNarrative, unused(), unused())
        .unwrap();
    app.dispatch_llm_request(&service);

    let deadline = Instant::now() + Duration::from_secs(1);
    let mut envelope = loop {
        if let Some(envelope) = service.try_recv() {
            break envelope;
        }
        assert!(Instant::now() < deadline);
        thread::yield_now();
    };
    envelope.schema_version = 2;
    app.accept_llm_response(envelope);

    assert_eq!(app.llm_status(), &LlmUiStatus::Invalid);
    assert_eq!(app.revision(), before);
    assert!(!app
        .narrative_lines()
        .iter()
        .any(|line| line.contains("Must not render")));
    assert!(service.shutdown_with_grace(Duration::from_millis(250)));
    server.join().unwrap();
}

#[test]
fn connection_failure_shows_down_and_fallback_without_core_effect() {
    let _lock = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (address, unavailable_server) = spawn_disconnect_server();
    let mut service = LocalLlmService::from_config(config_for_address(address)).unwrap();
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    let before = app.revision();
    app.handle_candidate(UiCommandCandidate::LlmNarrative, unused(), unused())
        .unwrap();
    app.dispatch_llm_request(&service);

    let deadline = Instant::now() + Duration::from_secs(1);
    while matches!(app.llm_status(), LlmUiStatus::Pending { .. }) {
        app.poll_llm_response(&service);
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert_eq!(app.llm_status(), &LlmUiStatus::Unavailable);
    assert_eq!(app.revision(), before);
    assert_eq!(app.narrative_lines()[1], "Local narrator unavailable.");
    app.handle_candidate(UiCommandCandidate::LlmRetry, unused(), unused())
        .unwrap();
    assert!(app.take_llm_request().is_none());
    thread::sleep(Duration::from_millis(260));
    app.handle_candidate(UiCommandCandidate::LlmRetry, unused(), unused())
        .unwrap();
    assert!(matches!(
        app.take_llm_request(),
        Some(LlmRequestKind::Narrative)
    ));
    assert!(service.shutdown_with_grace(Duration::from_millis(250)));
    unavailable_server.join().unwrap();
}

fn poll_until_ready(app: &mut TuiApp, service: &LocalLlmService) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while matches!(app.llm_status(), LlmUiStatus::Pending { .. }) {
        app.poll_llm_response(service);
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert_eq!(app.llm_status(), &LlmUiStatus::Ready);
}

fn service_for_response(body: &str) -> (LocalLlmService, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let body = body.to_string();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..read]);
            let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if request.len() >= header_end + 4 + content_length {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    (
        LocalLlmService::from_config(config_for_address(address)).unwrap(),
        server,
    )
}

fn spawn_disconnect_server() -> (std::net::SocketAddr, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        drop(stream);
    });
    (address, server)
}

fn config_for_address(address: std::net::SocketAddr) -> LocalLlmConfig {
    let saved = [
        (
            "AIHACK_LLM_ENABLED",
            std::env::var("AIHACK_LLM_ENABLED").ok(),
        ),
        (
            "AIHACK_LLM_ENDPOINT",
            std::env::var("AIHACK_LLM_ENDPOINT").ok(),
        ),
        ("AIHACK_LLM_MODEL", std::env::var("AIHACK_LLM_MODEL").ok()),
    ];
    std::env::set_var("AIHACK_LLM_ENABLED", "true");
    std::env::set_var("AIHACK_LLM_ENDPOINT", format!("http://{address}/v1"));
    std::env::set_var("AIHACK_LLM_MODEL", "local-model");
    let config = LocalLlmConfig::from_env().unwrap();
    for (name, value) in saved {
        match value {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
    }
    config
}
