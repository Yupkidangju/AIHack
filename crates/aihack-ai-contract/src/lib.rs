//! AI adapter가 mutable runtime state 대신 전달받는 안정된 계약 타입이다.

use serde::{Deserialize, Serialize};

pub mod llm;
pub mod observation;
pub use aihack_core::campaign::Role;
pub use aihack_core::{
    action::{ActionIntent, CommandIntent, DirectionalAction, InventoryAction, NarrativeTopic},
    domain::{
        combat::DeathCause,
        entity::EntityKind,
        item::ItemKind,
        monster::MonsterKind,
        tile::{TileKind, TrapKind},
    },
    event::GameEvent,
    hash::SnapshotHash,
    ids::{BranchId, EntityId, LevelId},
    position::{Delta, Direction, Pos},
    run_state::RunState,
};
pub use observation::*;

/// action suggestion과 narrative 결과가 현재 session을 가리키는 최소 revision이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientRevision {
    pub turn: u64,
    pub snapshot_hash: SnapshotHash,
}
