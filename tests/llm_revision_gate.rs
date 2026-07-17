use aihack::{
    core::{ActionIntent, CommandIntent, Direction, GameSession},
    llm::decision::{
        execute_validated_decision, parse_decision_payload_json, DecisionGate, DecisionPayload,
        DecisionRequest, DecisionResponseEnvelope,
    },
};
use aihack_llm::transport::{LlmResponseError, LlmValidationCode};

fn revision(session: &GameSession) -> aihack_llm::ClientRevision {
    aihack_llm::ClientRevision {
        turn: session.turn(),
        snapshot_hash: session.snapshot().stable_hash(),
    }
}

fn request(session: &GameSession) -> DecisionRequest {
    let observation = session.observation();
    DecisionRequest {
        revision: revision(session),
        action_space: observation.action_space.clone(),
        observation,
    }
}

fn payload(action: ActionIntent) -> DecisionPayload {
    DecisionPayload {
        action,
        rationale: "Current action is legal.".to_string(),
        confidence: 0.75,
    }
}

#[test]
fn unknown_request_id_is_rejected_without_consuming_the_outstanding_request() {
    let session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let expected_id = gate.begin(&request).unwrap();
    let mut other_gate = DecisionGate::new();
    let unknown_id = other_gate.begin(&request).unwrap();

    let unknown = gate.validate(
        DecisionResponseEnvelope::new(
            unknown_id,
            request.revision.clone(),
            payload(request.action_space.commands[0]),
        ),
        &revision(&session),
        &request.action_space,
    );
    assert_eq!(
        unknown,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::UnknownRequestId,
        })
    );

    let valid = gate.validate(
        DecisionResponseEnvelope::new(
            expected_id,
            request.revision.clone(),
            payload(request.action_space.commands[0]),
        ),
        &revision(&session),
        &request.action_space,
    );
    assert!(valid.is_ok());
}

#[test]
fn a_second_outstanding_request_of_the_same_kind_is_rejected() {
    let session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();

    gate.begin(&request).unwrap();

    assert_eq!(
        gate.begin(&request),
        Err(aihack::llm::decision::DecisionGateError::AlreadyOutstanding)
    );
}

#[test]
fn provider_error_clears_only_the_matching_outstanding_request() {
    let session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let request_id = gate.begin(&request).unwrap();
    let mut other_gate = DecisionGate::new();
    let unknown_id = other_gate.begin(&request).unwrap();

    assert_eq!(
        gate.complete_error(&unknown_id),
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::UnknownRequestId,
        })
    );
    assert_eq!(
        gate.begin(&request),
        Err(aihack::llm::decision::DecisionGateError::AlreadyOutstanding)
    );

    gate.complete_error(&request_id).unwrap();
    assert!(gate.begin(&request).is_ok());
}

#[test]
fn matching_response_becomes_stale_after_the_session_revision_changes() {
    let mut session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let request_id = gate.begin(&request).unwrap();
    session.submit(CommandIntent::Wait);
    let before_validation = session.snapshot().stable_hash();

    let result = gate.validate(
        DecisionResponseEnvelope::new(
            request_id,
            request.revision.clone(),
            payload(request.action_space.commands[0]),
        ),
        &revision(&session),
        &session.observation().action_space,
    );

    assert_eq!(result, Err(LlmResponseError::Stale));
    assert_eq!(before_validation, session.snapshot().stable_hash());
}

#[test]
fn action_outside_the_current_action_space_is_rejected() {
    let session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let request_id = gate.begin(&request).unwrap();

    let result = gate.validate(
        DecisionResponseEnvelope::new(
            request_id,
            request.revision.clone(),
            payload(ActionIntent::Command(CommandIntent::Open(Direction::North))),
        ),
        &revision(&session),
        &request.action_space,
    );

    assert_eq!(
        result,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::InvalidAction,
        })
    );
}

#[test]
fn malformed_confidence_or_rationale_is_rejected_at_the_response_boundary() {
    for (rationale, confidence, code) in [
        (
            "valid".to_string(),
            f32::NAN,
            LlmValidationCode::InvalidConfidence,
        ),
        ("x".repeat(161), 0.5, LlmValidationCode::TextTooLong),
        (
            "unsafe\u{1b}[31m".to_string(),
            0.5,
            LlmValidationCode::ControlCharacter,
        ),
    ] {
        let session = GameSession::new_for_playing(42);
        let request = request(&session);
        let mut gate = DecisionGate::new();
        let request_id = gate.begin(&request).unwrap();
        let result = gate.validate(
            DecisionResponseEnvelope::new(
                request_id,
                request.revision.clone(),
                DecisionPayload {
                    action: request.action_space.commands[0],
                    rationale,
                    confidence,
                },
            ),
            &revision(&session),
            &request.action_space,
        );
        assert_eq!(result, Err(LlmResponseError::InvalidSchema { code }));
    }
}

#[test]
fn decision_wire_json_is_strict_and_maps_only_known_action_shapes() {
    let session = GameSession::new_for_playing(42);
    let action_space = session.observation().action_space;
    let parsed = parse_decision_payload_json(
        r#"{"kind":"DECISION","action":{"type":"WAIT"},"rationale":"Hold position.","confidence":0.5}"#,
        &action_space,
    )
    .unwrap();
    assert_eq!(parsed.action, ActionIntent::Command(CommandIntent::Wait));

    let unknown = parse_decision_payload_json(
        r#"{"kind":"DECISION","action":{"type":"WAIT"},"rationale":"ok","confidence":0.5,"extra":true}"#,
        &action_space,
    );
    assert_eq!(
        unknown,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::InvalidJson,
        })
    );

    let wrong_kind = parse_decision_payload_json(
        r#"{"kind":"NARRATIVE","action":{"type":"WAIT"},"rationale":"ok","confidence":0.5}"#,
        &action_space,
    );
    assert_eq!(
        wrong_kind,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::WrongKind,
        })
    );

    let invalid_action = parse_decision_payload_json(
        r#"{"kind":"DECISION","action":{"type":"OPEN","direction":"NORTH"},"rationale":"ok","confidence":0.5}"#,
        &action_space,
    );
    assert_eq!(
        invalid_action,
        Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::InvalidAction,
        })
    );
}

#[test]
fn only_a_validated_command_uses_the_normal_submit_path() {
    let mut session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let request_id = gate.begin(&request).unwrap();
    let validated = gate
        .validate(
            DecisionResponseEnvelope::new(
                request_id,
                request.revision.clone(),
                payload(ActionIntent::Command(CommandIntent::Wait)),
            ),
            &revision(&session),
            &request.action_space,
        )
        .unwrap();

    let outcome = execute_validated_decision(&mut session, &validated).unwrap();

    assert!(outcome.accepted);
    assert_eq!(session.turn(), 1);
}

#[test]
fn validated_decision_is_not_executed_if_revision_changes_before_submit() {
    let mut session = GameSession::new_for_playing(42);
    let request = request(&session);
    let mut gate = DecisionGate::new();
    let request_id = gate.begin(&request).unwrap();
    let validated = gate
        .validate(
            DecisionResponseEnvelope::new(
                request_id,
                request.revision.clone(),
                payload(ActionIntent::Command(CommandIntent::Wait)),
            ),
            &revision(&session),
            &request.action_space,
        )
        .unwrap();
    session.submit(CommandIntent::Wait);
    let before_execute = session.snapshot().stable_hash();

    let outcome = execute_validated_decision(&mut session, &validated);

    assert!(outcome.is_none());
    assert_eq!(session.turn(), 1);
    assert_eq!(before_execute, session.snapshot().stable_hash());
}
