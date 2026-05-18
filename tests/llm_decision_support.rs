use std::{sync::Arc, time::Duration};

use aihack::{
    core::{ActionIntent, CommandIntent, GameSession},
    llm::decision::{
        decision_log_lines, execute_suggestion, request_decision_with_timeout, DecisionError,
        DecisionProvider, DecisionRequest, DecisionSource, SuggestedAction,
    },
};

struct LegalProvider;
impl DecisionProvider for LegalProvider {
    fn suggest(
        &self,
        request: &DecisionRequest,
        _timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError> {
        Ok(SuggestedAction {
            action: request.action_space.commands[0],
            rationale: "legal".to_string(),
            source: DecisionSource::Provider,
        })
    }
}

struct IllegalProvider;
impl DecisionProvider for IllegalProvider {
    fn suggest(
        &self,
        _request: &DecisionRequest,
        _timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError> {
        Ok(SuggestedAction {
            action: ActionIntent::Command(CommandIntent::Open(aihack::core::Direction::North)),
            rationale: "illegal".to_string(),
            source: DecisionSource::Provider,
        })
    }
}

struct TimeoutProvider;
impl DecisionProvider for TimeoutProvider {
    fn suggest(
        &self,
        _request: &DecisionRequest,
        _timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError> {
        Err(DecisionError::Timeout)
    }
}

struct FailingProvider;
impl DecisionProvider for FailingProvider {
    fn suggest(
        &self,
        _request: &DecisionRequest,
        _timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError> {
        Err(DecisionError::Provider("offline".to_string()))
    }
}

fn request() -> DecisionRequest {
    let observation = GameSession::new_for_playing(42).observation();
    DecisionRequest {
        observation: observation.clone(),
        action_space: observation.action_space,
    }
}

#[test]
fn provider_success_returns_legal_suggestion() {
    let suggestion = request_decision_with_timeout(
        Some(Arc::new(LegalProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(suggestion.source, DecisionSource::Provider);
}

#[test]
fn illegal_suggestion_is_rejected() {
    let suggestion = request_decision_with_timeout(
        Some(Arc::new(IllegalProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(suggestion.source, DecisionSource::Fallback);
    assert!(matches!(
        suggestion.action,
        ActionIntent::Command(CommandIntent::Wait) | ActionIntent::Noop
    ));
}

#[test]
fn timeout_or_failure_uses_fallback() {
    let timeout = request_decision_with_timeout(
        Some(Arc::new(TimeoutProvider)),
        request(),
        Duration::from_millis(1),
    );
    assert_eq!(timeout.source, DecisionSource::Fallback);
    let failure = request_decision_with_timeout(
        Some(Arc::new(FailingProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(failure.source, DecisionSource::Fallback);
}

#[test]
fn suggestion_does_not_affect_snapshot_hash() {
    let session = GameSession::new_for_playing(42);
    let before = session.snapshot().stable_hash();
    let _ = request_decision_with_timeout(
        Some(Arc::new(LegalProvider)),
        request(),
        Duration::from_millis(10),
    );
    let after = session.snapshot().stable_hash();
    assert_eq!(before, after);
}

#[test]
fn approved_suggestion_executes_via_submit() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    let observation = session.observation();
    let request = DecisionRequest {
        observation: observation.clone(),
        action_space: observation.action_space,
    };
    let suggestion = request_decision_with_timeout(
        Some(Arc::new(LegalProvider)),
        request.clone(),
        Duration::from_millis(10),
    );
    let outcome = execute_suggestion(&mut session, &request, &suggestion).unwrap();
    assert!(outcome.accepted || !outcome.turn_advanced || !outcome.events.is_empty());
    let lines = decision_log_lines(&suggestion, Some(true));
    assert_eq!(lines.len(), 2);
}

#[test]
fn rejected_suggestion_does_not_execute() {
    let mut session = GameSession::new_for_playing(42);
    let request = request();
    let illegal = SuggestedAction {
        action: ActionIntent::Command(CommandIntent::Open(aihack::core::Direction::North)),
        rationale: "illegal".to_string(),
        source: DecisionSource::Provider,
    };
    let before = session.snapshot().stable_hash();
    let outcome = execute_suggestion(&mut session, &request, &illegal);
    assert!(outcome.is_none());
    assert_eq!(before, session.snapshot().stable_hash());
}
