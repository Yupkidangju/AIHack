use std::{sync::Arc, time::Duration};

use aihack::{
    core::{GameSession, NarrativeTopic},
    llm::narrative::{
        narrative_log_lines, request_narrative_with_timeout, NarrativeError, NarrativeProvider,
        NarrativeRequest, NarrativeSource,
    },
};

struct SuccessProvider;
impl NarrativeProvider for SuccessProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, NarrativeError> {
        Ok("A quiet tension hangs in the dungeon air.".to_string())
    }
}

struct FailingProvider;
impl NarrativeProvider for FailingProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, NarrativeError> {
        Err(NarrativeError::Provider("offline".to_string()))
    }
}

struct EmptyProvider;
impl NarrativeProvider for EmptyProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, NarrativeError> {
        Ok("   ".to_string())
    }
}

struct SlowProvider;
impl NarrativeProvider for SlowProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, NarrativeError> {
        Err(NarrativeError::Timeout)
    }
}

fn request() -> NarrativeRequest {
    NarrativeRequest {
        topic: NarrativeTopic::SituationSummary,
        observation: GameSession::new_for_playing(42).observation(),
    }
}

#[test]
fn provider_success_returns_narrative() {
    let response = request_narrative_with_timeout(
        Some(Arc::new(SuccessProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(response.source, NarrativeSource::Provider);
    assert!(!response.text.is_empty());
    assert!(!response.timed_out);
}

#[test]
fn timeout_returns_fallback() {
    let response = request_narrative_with_timeout(
        Some(Arc::new(SlowProvider)),
        request(),
        Duration::from_millis(1),
    );
    assert_eq!(response.source, NarrativeSource::Fallback);
    assert!(response.timed_out);
    assert!(!response.text.is_empty());
}

#[test]
fn provider_failure_uses_fallback() {
    let response = request_narrative_with_timeout(
        Some(Arc::new(FailingProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(response.source, NarrativeSource::Fallback);
    assert!(!response.text.is_empty());
}

#[test]
fn empty_response_uses_fallback() {
    let response = request_narrative_with_timeout(
        Some(Arc::new(EmptyProvider)),
        request(),
        Duration::from_millis(10),
    );
    assert_eq!(response.source, NarrativeSource::Fallback);
    assert!(!response.text.is_empty());
}

#[test]
fn narrative_does_not_affect_snapshot_hash() {
    let session = GameSession::new_for_playing(42);
    let before = session.snapshot().stable_hash();
    let response = request_narrative_with_timeout(
        Some(Arc::new(SuccessProvider)),
        NarrativeRequest {
            topic: NarrativeTopic::SituationSummary,
            observation: session.observation(),
        },
        Duration::from_millis(10),
    );
    let after = session.snapshot().stable_hash();
    assert_eq!(before, after);
    let lines = narrative_log_lines(&response);
    assert_eq!(lines.len(), 2);
}
