use aihack::{
    core::GameSession,
    llm::soft_adjudication::{
        fallback_soft_adjudication, parse_soft_adjudication_payload_json, soft_adjudication_lines,
        SoftAdjudicationSource,
    },
    ui::tui::{TuiApp, UiCommandCandidate, UiRuntimeConfig},
};
use aihack_ai_contract::llm::SoftVerdict;
use aihack_llm::transport::{LlmResponseError, LlmValidationCode};

#[test]
fn soft_adjudication_wire_json_is_strict_and_uses_upper_snake_verdicts() {
    let payload = parse_soft_adjudication_payload_json(
        r#"{"kind":"SOFT_ADJUDICATION","verdict":"NEUTRAL","reasonCode":"SOCIAL_UNCERTAIN","message":"The attempt is plausible."}"#,
    )
    .unwrap();

    assert_eq!(payload.verdict, SoftVerdict::Neutral);
    assert_eq!(payload.reason_code, "SOCIAL_UNCERTAIN");
    assert_eq!(payload.message, "The attempt is plausible.");

    for invalid_json in [
        r#"{"kind":"SOFT_ADJUDICATION","verdict":"NEUTRAL","reasonCode":"OK","message":"valid","effect":1}"#,
        r#"{"kind":"DECISION","verdict":"NEUTRAL","reasonCode":"OK","message":"valid"}"#,
    ] {
        assert!(parse_soft_adjudication_payload_json(invalid_json).is_err());
    }
}

#[test]
fn invalid_reason_codes_messages_and_controls_are_rejected_at_the_boundary() {
    for (reason_code, message, expected_code) in [
        ("lowercase", "valid", LlmValidationCode::InvalidReasonCode),
        ("A-B", "valid", LlmValidationCode::InvalidReasonCode),
        (" OK ", "valid", LlmValidationCode::InvalidReasonCode),
        ("", "valid", LlmValidationCode::InvalidReasonCode),
        (
            "A23456789012345678901234567890123",
            "valid",
            LlmValidationCode::InvalidReasonCode,
        ),
        ("OK", "", LlmValidationCode::EmptyText),
        (
            "OK",
            "unsafe\u{1b}[31m",
            LlmValidationCode::ControlCharacter,
        ),
    ] {
        let json = serde_json::json!({
            "kind": "SOFT_ADJUDICATION",
            "verdict": "FAVORABLE",
            "reasonCode": reason_code,
            "message": message,
        });
        assert_eq!(
            parse_soft_adjudication_payload_json(&json.to_string()),
            Err(LlmResponseError::InvalidSchema {
                code: expected_code,
            })
        );
    }

    let long_message = "가".repeat(241);
    let json = serde_json::json!({
        "kind": "SOFT_ADJUDICATION",
        "verdict": "UNFAVORABLE",
        "reasonCode": "TOO_LONG",
        "message": long_message,
    });
    assert_eq!(
        parse_soft_adjudication_payload_json(&json.to_string()),
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::TextTooLong,
        })
    );
}

#[test]
fn unavailable_fallback_is_neutral_and_plain_text_remains_readable() {
    let response = fallback_soft_adjudication();
    assert_eq!(response.payload.verdict, SoftVerdict::Neutral);
    assert_eq!(response.payload.reason_code, "LLM_UNAVAILABLE");
    assert_eq!(response.source, SoftAdjudicationSource::Fallback);

    let lines = soft_adjudication_lines(&response);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("Neutral"));
    assert!(lines[0].contains("LLM_UNAVAILABLE"));
    assert!(lines[2].contains("[N] Dismiss"));
}

#[test]
fn high_contrast_and_reduced_motion_keep_semantic_text_and_dismiss_is_ui_only() {
    let config = UiRuntimeConfig {
        high_contrast: true,
        reduced_motion: true,
        ..UiRuntimeConfig::default()
    };
    let mut app = TuiApp::new(GameSession::new_for_playing(42), config);
    let before = app.revision();
    app.set_soft_adjudication(fallback_soft_adjudication());

    let lines = app.soft_adjudication_lines();
    assert!(lines[0].contains("Neutral"));
    assert!(lines[0].contains("LLM_UNAVAILABLE"));
    let panel_lines = aihack::ui::tui::render_panels::inspect_lines(
        &app.observation(),
        None,
        aihack::ui::tui::UiPanel::Inspect,
        &lines,
    );
    assert!(panel_lines[0].contains("Soft judgment"));

    app.handle_candidate_owned(UiCommandCandidate::DismissLlmResult)
        .unwrap();
    assert_eq!(app.revision(), before);
    assert!(app.soft_adjudication_lines()[0].contains("idle"));
}

#[test]
fn displaying_soft_adjudication_has_no_core_save_or_replay_truth_effect() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before_revision = app.revision();
    let before_observation = app.observation();

    app.set_soft_adjudication(fallback_soft_adjudication());

    assert_eq!(app.revision(), before_revision);
    assert_eq!(app.observation(), before_observation);
    assert!(app
        .soft_adjudication_lines()
        .iter()
        .any(|line| line.contains("LLM_UNAVAILABLE")));

    app.quick_save().unwrap();
    app.quick_load().unwrap();
    assert_eq!(app.revision(), before_revision);
    assert!(app.soft_adjudication_lines()[0].contains("idle"));
}
