use crate::core::{ActionIntent, GameSession, TurnOutcome};

pub use aihack_llm::decision::*;

/// 게임 세션에 명령을 적용하는 것은 앱 어댑터의 책임으로 남긴다.
pub fn execute_suggestion(
    session: &mut GameSession,
    request: &DecisionRequest,
    suggestion: &SuggestedAction,
) -> Option<TurnOutcome> {
    if !is_legal_suggestion(request, suggestion) {
        return None;
    }
    match suggestion.action {
        ActionIntent::Command(command) => Some(session.submit(command)),
        ActionIntent::NarrativeRequest { .. } | ActionIntent::Noop => None,
    }
}

/// correlation과 current ActionSpace 검증을 마친 명령만 기존 submit 경로로 전달한다.
pub fn execute_validated_decision(
    session: &mut GameSession,
    decision: &ValidatedDecision,
) -> Option<TurnOutcome> {
    if session.turn() != decision.revision().turn
        || session.snapshot().stable_hash() != decision.revision().snapshot_hash
    {
        return None;
    }
    match decision.action() {
        ActionIntent::Command(command) => Some(session.submit(command)),
        ActionIntent::NarrativeRequest { .. } | ActionIntent::Noop => None,
    }
}
