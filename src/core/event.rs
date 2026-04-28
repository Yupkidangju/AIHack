use serde::{Deserialize, Serialize};

use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
    },
    domain::{
        combat::DeathCause, inventory::InventoryLetter, item::EquipmentSlot, tile::DoorState,
    },
};

/// [v0.1.0] Phase 5 replay/hash 입력으로 level transition event를 추가한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    TurnStarted {
        turn: u64,
    },
    Waited {
        turn: u64,
    },
    EntityMoved {
        from: Pos,
        to: Pos,
    },
    LevelChanged {
        entity: EntityId,
        from: LevelId,
        to: LevelId,
    },
    DoorChanged {
        pos: Pos,
        from: DoorState,
        to: DoorState,
    },
    ItemPickedUp {
        entity: EntityId,
        item: EntityId,
        letter: InventoryLetter,
    },
    ItemEquipped {
        entity: EntityId,
        item: EntityId,
        slot: EquipmentSlot,
    },
    ItemConsumed {
        entity: EntityId,
        item: EntityId,
    },
    EntityHealed {
        entity: EntityId,
        amount: i16,
        hp_after: i16,
    },
    AttackResolved {
        attacker: EntityId,
        defender: EntityId,
        attack_roll: i16,
        defense: i16,
        hit: bool,
        damage: i16,
    },
    EntityDied {
        entity: EntityId,
        cause: DeathCause,
    },
    CommandRejected {
        reason: String,
    },
}
