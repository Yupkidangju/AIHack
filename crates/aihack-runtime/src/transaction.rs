use aihack_core::{
    action::CommandIntent,
    invariant::{InvariantReport, WORLD_INVARIANT_COUNT},
    turn::TurnOutcome,
};

use crate::session::GameSession;

/// 한 명령의 working copy를 보관한다. validate가 끝나기 전에는 원본 session을 변경하지 않는다.
pub(crate) struct TurnTransaction {
    inner: aihack_core::transaction::TurnTransaction<GameSession>,
}

impl TurnTransaction {
    pub(crate) fn prepare(session: &GameSession) -> Self {
        Self {
            inner: aihack_core::transaction::TurnTransaction::prepare(session),
        }
    }

    pub(crate) fn apply(&mut self, intent: CommandIntent) -> TurnOutcome {
        self.inner.working_mut().submit_uncommitted(intent)
    }

    pub(crate) fn validate(&self) -> InvariantReport {
        self.inner.working().world().validate_invariants()
    }

    pub(crate) fn commit(self) -> GameSession {
        self.inner.commit()
    }

    pub(crate) fn invariant_reason(report: &InvariantReport) -> String {
        debug_assert_eq!(report.checked, WORLD_INVARIANT_COUNT);
        format!("world invariant violation: {:?}", report.errors)
    }
}
