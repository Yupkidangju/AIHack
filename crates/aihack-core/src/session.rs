use crate::{event::GameEvent, meta::GameMeta, rng::GameRng, run_state::RunState};

/// command 처리 adapter가 소유하는 world를 제외한 결정론적 session 상태다.
#[derive(Debug, Clone)]
pub struct SessionState<W> {
    pub meta: GameMeta,
    pub rng: GameRng,
    pub turn: u64,
    pub state: RunState,
    pub world: W,
    pub event_log: Vec<GameEvent>,
}
