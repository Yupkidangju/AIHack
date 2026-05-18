use std::{sync::Arc, time::Duration};

use crate::core::{ActionIntent, ActionSpace, CommandIntent, GameSession, Observation};

pub const DECISION_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRequest {
    pub observation: Observation,
    pub action_space: ActionSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSource {
    Provider,
    Fallback,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedAction {
    pub action: ActionIntent,
    pub rationale: String,
    pub source: DecisionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionError {
    Provider(String),
    Timeout,
    Invalid(String),
}

pub trait DecisionProvider: Send + Sync {
    fn suggest(
        &self,
        request: &DecisionRequest,
        timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError>;
}

pub fn request_decision(
    provider: Option<Arc<dyn DecisionProvider>>,
    request: DecisionRequest,
) -> SuggestedAction {
    request_decision_with_timeout(
        provider,
        request,
        Duration::from_millis(DECISION_TIMEOUT_MS),
    )
}

pub fn request_decision_with_timeout(
    provider: Option<Arc<dyn DecisionProvider>>,
    request: DecisionRequest,
    timeout: Duration,
) -> SuggestedAction {
    let Some(provider) = provider else {
        return fallback_suggestion(&request, DecisionSource::Disabled);
    };
    match provider.suggest(&request, timeout) {
        Ok(suggestion) if is_legal_suggestion(&request, &suggestion) => suggestion,
        Ok(_) => fallback_suggestion(&request, DecisionSource::Fallback),
        Err(DecisionError::Timeout) => fallback_suggestion(&request, DecisionSource::Fallback),
        Err(_) => fallback_suggestion(&request, DecisionSource::Fallback),
    }
}

pub fn is_legal_suggestion(request: &DecisionRequest, suggestion: &SuggestedAction) -> bool {
    request.action_space.commands.contains(&suggestion.action)
}

pub fn execute_suggestion(
    session: &mut GameSession,
    request: &DecisionRequest,
    suggestion: &SuggestedAction,
) -> Option<crate::core::TurnOutcome> {
    if !is_legal_suggestion(request, suggestion) {
        return None;
    }
    match suggestion.action {
        ActionIntent::Command(command) => Some(session.submit(command)),
        ActionIntent::NarrativeRequest { .. } | ActionIntent::Noop => None,
    }
}

pub fn fallback_suggestion(request: &DecisionRequest, source: DecisionSource) -> SuggestedAction {
    let action = request
        .action_space
        .commands
        .iter()
        .copied()
        .find(|action| {
            matches!(
                action,
                ActionIntent::Command(CommandIntent::Wait) | ActionIntent::Noop
            )
        })
        .or_else(|| request.action_space.commands.first().copied())
        .unwrap_or(ActionIntent::Noop);
    SuggestedAction {
        action,
        rationale: "deterministic fallback suggestion".to_string(),
        source,
    }
}

pub fn decision_log_lines(suggestion: &SuggestedAction, accepted: Option<bool>) -> Vec<String> {
    let status = match accepted {
        Some(true) => "accepted",
        Some(false) => "rejected",
        None => "suggested",
    };
    vec![
        format!("Decision({:?}, {status})", suggestion.source),
        format!("{:?} :: {}", suggestion.action, suggestion.rationale),
    ]
}
