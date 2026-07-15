use crate::core::{
    action::CommandIntent,
    invariant::{InvariantReport, WORLD_INVARIANT_COUNT},
    session::GameSession,
    turn::TurnOutcome,
};

/// 한 명령의 working copy를 보관한다. validate가 끝나기 전에는 원본 session을 변경하지 않는다.
pub(crate) struct TurnTransaction {
    working: GameSession,
}

impl TurnTransaction {
    pub(crate) fn prepare(session: &GameSession) -> Self {
        Self {
            working: session.clone(),
        }
    }

    pub(crate) fn apply(&mut self, intent: CommandIntent) -> TurnOutcome {
        self.working.submit_uncommitted(intent)
    }

    pub(crate) fn validate(&self) -> InvariantReport {
        self.working.world().validate_invariants()
    }

    pub(crate) fn commit(self) -> GameSession {
        self.working
    }

    pub(crate) fn invariant_reason(report: &InvariantReport) -> String {
        debug_assert_eq!(report.checked, WORLD_INVARIANT_COUNT);
        format!("world invariant violation: {:?}", report.errors)
    }
}
