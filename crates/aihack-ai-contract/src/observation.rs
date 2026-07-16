use aihack_core::{
    action::{ActionIntent, CommandIntent},
    domain::{
        entity::EntityKind,
        inventory::InventoryLetter,
        item::{EquipmentSlot, ItemKind},
        tile::TileKind,
    },
    event::GameEvent,
    ids::{EntityId, LevelId},
    position::{Delta, Pos},
};
use serde::{Deserialize, Serialize};

pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStateSummary {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection,
    AwaitingInventorySelection,
    MorePrompt,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerObservation {
    pub entity: EntityId,
    pub pos: Pos,
    pub hp: i16,
    pub max_hp: i16,
    pub current_level: LevelId,
    pub hunger: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityObservation {
    pub entity: EntityId,
    pub kind: EntityKind,
    pub pos: Pos,
    pub hp: Option<i16>,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionSpace {
    pub commands: Vec<ActionIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub current_level: LevelId,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub player_pos: Pos,
    pub visible_tiles: Vec<TileObservation>,
    pub visible_entities: Vec<EntityObservation>,
    pub inventory: Vec<ItemObservation>,
    pub last_events: Vec<GameEvent>,
    pub action_space: ActionSpace,
    pub legal_actions: Vec<CommandIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemObservation {
    pub item: EntityId,
    pub kind: ItemKind,
    pub letter: InventoryLetter,
    pub equipped_slot: Option<EquipmentSlot>,
    pub identified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileObservation {
    pub pos: Pos,
    pub rel: Delta,
    pub tile: TileKind,
    pub visible: bool,
}
