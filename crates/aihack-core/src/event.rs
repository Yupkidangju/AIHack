use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        combat::DeathCause,
        inventory::InventoryLetter,
        item::EquipmentSlot,
        tile::{DoorState, TileKind, TrapKind},
    },
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    TurnStarted {
        turn: u64,
    },
    Waited {
        turn: u64,
    },
    EntityMoved {
        entity: EntityId,
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
    TileRevealed {
        pos: Pos,
        tile: TileKind,
    },
    ItemPickedUp {
        entity: EntityId,
        item: EntityId,
        letter: InventoryLetter,
    },
    ItemDropped {
        entity: EntityId,
        item: EntityId,
        pos: Pos,
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
    TrapTriggered {
        entity: EntityId,
        trap: TrapKind,
        pos: Pos,
        damage: i16,
    },
    ItemThrown {
        entity: EntityId,
        item: EntityId,
        from: Pos,
        to: Pos,
    },
    WandZapped {
        entity: EntityId,
        item: EntityId,
        direction: Direction,
        charges_after: u8,
    },
    ScrollRead {
        entity: EntityId,
        item: EntityId,
    },
    ItemIdentified {
        entity: EntityId,
        item: EntityId,
    },
    DoorKicked {
        pos: Pos,
    },
    PassiveAttackTriggered {
        source: EntityId,
        target: EntityId,
    },
    PrayerOffered {
        entity: EntityId,
        cooldown_after: u16,
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
    Message {
        priority: MessagePriority,
        text: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Info,
    Warning,
    Danger,
}
