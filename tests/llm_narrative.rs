use std::{sync::Arc, time::Duration};

use aihack::{
    core::{GameSession, NarrativeTopic},
    llm::narrative::{
        narrative_log_lines, request_narrative_with_timeout, NarrativeError, NarrativeProvider,
        NarrativeRequest, NarrativeSource,
    },
};
use aihack_llm::ClientRevision;

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

struct TextProvider(String);

impl NarrativeProvider for TextProvider {
    fn generate(
        &self,
        _request: &NarrativeRequest,
        _timeout: Duration,
    ) -> Result<String, NarrativeError> {
        Ok(self.0.clone())
    }
}

fn request() -> NarrativeRequest {
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
    assert!(response.text.starts_with("Turn "));
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
            revision: ClientRevision {
                turn: session.turn(),
                snapshot_hash: session.snapshot().stable_hash(),
            },
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

#[test]
fn narrative_limit_counts_unicode_scalars_instead_of_bytes() {
    let accepted = request_narrative_with_timeout(
        Some(Arc::new(TextProvider("가".repeat(240)))),
        request(),
        Duration::from_millis(10),
    );
    let rejected = request_narrative_with_timeout(
        Some(Arc::new(TextProvider("가".repeat(241)))),
        request(),
        Duration::from_millis(10),
    );

    assert_eq!(accepted.source, NarrativeSource::Provider);
    assert_eq!(rejected.source, NarrativeSource::Fallback);
}

#[test]
fn narrative_rejects_c0_c1_and_ansi_controls() {
    for text in ["line\nfeed", "tab\ttext", "ansi\u{1b}[31m", "c1\u{85}text"] {
        let response = request_narrative_with_timeout(
            Some(Arc::new(TextProvider(text.to_string()))),
            request(),
            Duration::from_millis(10),
        );
        assert_eq!(response.source, NarrativeSource::Fallback);
    }
}
