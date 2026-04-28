use serde::{Deserialize, Serialize};

use crate::core::{event::GameEvent, session::RunState};

/// [v0.1.0] 플랫폼/러스트 버전에 흔들리지 않도록 문자열로 노출하는 고정 hash 값이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotHash(pub String);

/// [v0.1.0] `GameSession::submit()`이 반환하는 턴 처리 결과다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnOutcome {
    pub accepted: bool,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
    pub snapshot_hash: SnapshotHash,
    pub next_state: RunState,
}
