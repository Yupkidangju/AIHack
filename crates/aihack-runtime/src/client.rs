use aihack_ai_contract::{ClientRevision, Observation};
use aihack_core::{action::CommandIntent, run_state::RunState, turn::TurnOutcome};

/// TUI, headless, 향후 adapter가 게임 실행에 사용하는 최소 경계다.
pub trait GameClient {
    fn observation(&self) -> Observation;
    fn revision(&self) -> ClientRevision;
    fn run_state(&self) -> RunState;
    fn submit(&mut self, intent: CommandIntent) -> TurnOutcome;
}
