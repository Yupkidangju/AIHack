use serde::{Deserialize, Serialize};

use crate::{
    action::CommandIntent,
    domain::{inventory::Inventory, item::ItemKind, level::LevelRegistry},
    event::GameEvent,
    hash::SnapshotHash,
    ids::{EntityId, LevelId},
    rng::RngStateV1,
    run_state::RunState,
    turn::TurnOutcome,
    world::WorldState,
};

pub const SAVE_SCHEMA_VERSION_V1: u16 = 1;

/// v1 JSON save에 포함되는 world 상태다. runtime-only 사망 원인은 저장하지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedWorldV1<E> {
    pub levels: LevelRegistry,
    pub current_level: LevelId,
    pub entities: E,
    pub player_id: EntityId,
    pub inventory: Inventory,
    pub nutrition: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
    pub kill_count: u32,
    pub gold: u32,
    pub identified_items: Vec<ItemKind>,
}

impl<E: Clone> From<&WorldState<E>> for SavedWorldV1<E> {
    fn from(world: &WorldState<E>) -> Self {
        Self {
            levels: world.levels.clone(),
            current_level: world.current_level,
            entities: world.entities.clone(),
            player_id: world.player_id,
            inventory: world.inventory.clone(),
            nutrition: world.nutrition,
            luck: world.luck,
            prayer_cooldown: world.prayer_cooldown,
            paralysis_turns: world.paralysis_turns,
            hallucinating: world.hallucinating,
            kill_count: world.kill_count,
            gold: world.gold,
            identified_items: world.identified_items.clone(),
        }
    }
}

impl<E> From<SavedWorldV1<E>> for WorldState<E> {
    fn from(saved: SavedWorldV1<E>) -> Self {
        Self {
            levels: saved.levels,
            current_level: saved.current_level,
            entities: saved.entities,
            player_id: saved.player_id,
            inventory: saved.inventory,
            nutrition: saved.nutrition,
            luck: saved.luck,
            prayer_cooldown: saved.prayer_cooldown,
            paralysis_turns: saved.paralysis_turns,
            hallucinating: saved.hallucinating,
            kill_count: saved.kill_count,
            gold: saved.gold,
            identified_items: saved.identified_items,
            last_death_cause: None,
        }
    }
}

/// v1 JSON save envelope다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDataV1<E> {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunState,
    pub rng_state: RngStateV1,
    pub world: SavedWorldV1<E>,
    pub event_log: Vec<GameEvent>,
}

/// replay JSONL line schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLineV1 {
    pub turn_before: u64,
    pub command: CommandIntent,
    pub outcome: TurnOutcome,
    pub snapshot_hash_after: SnapshotHash,
}
