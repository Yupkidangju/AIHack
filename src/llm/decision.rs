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
