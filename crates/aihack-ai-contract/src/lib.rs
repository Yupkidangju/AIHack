//! AI adapter가 mutable runtime state 대신 전달받는 안정된 계약 타입이다.

use serde::{Deserialize, Serialize};

pub mod llm;
pub mod observation;
pub use aihack_core::campaign::{
    Alignment, Attributes, LearnedSpell, Race, Role, SkillCategory, SkillRank, SpellKind,
    SpellSchool,
};
pub use aihack_core::{
    action::{
        ActionIntent, CommandIntent, DipTarget, DirectionalAction, EngravingMethod, EngravingText,
        InventoryAction, NarrativeTopic, WishChoice,
    },
    domain::{
        combat::DeathCause,
        entity::{EntityKind, Faction},
        inventory::InventoryLetter,
        item::{
            AmuletEffect, BucStatus, EquipmentSlot, ItemClass, ItemKind, RingEffect, ToolAction,
        },
        monster::MonsterKind,
        status::{Resistances, StatusTimers},
        tile::{DoorState, TileKind, TrapKind},
    },
    event::{GameEvent, MessagePriority},
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
