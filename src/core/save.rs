use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        action::CommandIntent,
        error::GameError,
        event::GameEvent,
        ids::{EntityId, LevelId},
        rng::RngStateV1,
        session::{GameMeta, GameSession, RunState},
        turn::{SnapshotHash, TurnOutcome},
        world::GameWorld,
    },
    domain::{entity::EntityStore, inventory::Inventory, item::ItemKind, level::LevelRegistry},
};

pub const SAVE_SCHEMA_VERSION_V1: u16 = 1;

/// [v0.1.0] Phase 9 explicit world persistence schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedWorldV1 {
    pub levels: LevelRegistry,
    pub current_level: LevelId,
    pub entities: EntityStore,
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

/// [v0.1.0] Phase 9 JSON save v1 schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDataV1 {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunState,
    pub rng_state: RngStateV1,
    pub world: SavedWorldV1,
    pub event_log: Vec<GameEvent>,
}

/// [v0.1.0] replay JSONL line schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLineV1 {
    pub turn_before: u64,
    pub command: CommandIntent,
    pub outcome: TurnOutcome,
    pub snapshot_hash_after: SnapshotHash,
}

pub fn save_session_to_path(session: &GameSession, path: &Path) -> Result<(), GameError> {
    ensure_parent_dir(path)?;
    let payload = serde_json::to_string_pretty(&session.to_save_data())
        .map_err(|error| GameError::Serialization(error.to_string()))?;
    fs::write(path, payload).map_err(|error| GameError::Io(error.to_string()))
}

pub fn load_session_from_path(path: &Path) -> Result<GameSession, GameError> {
    let payload = fs::read_to_string(path).map_err(|error| GameError::Io(error.to_string()))?;
    let save: SaveDataV1 = serde_json::from_str(&payload)
        .map_err(|error| GameError::Serialization(error.to_string()))?;
    GameSession::from_save_data(save)
}

pub fn append_replay_line(path: &Path, line: &ReplayLineV1) -> Result<(), GameError> {
    ensure_parent_dir(path)?;
    let file = File::options()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| GameError::Io(error.to_string()))?;
    let mut writer = BufWriter::new(file);
    let encoded =
        serde_json::to_string(line).map_err(|error| GameError::Serialization(error.to_string()))?;
    writer
        .write_all(encoded.as_bytes())
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|error| GameError::Io(error.to_string()))
}

pub fn read_replay_lines(path: &Path) -> Result<Vec<ReplayLineV1>, GameError> {
    let file = File::open(path).map_err(|error| GameError::Io(error.to_string()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|line| {
            let line = line.map_err(|error| GameError::Io(error.to_string()))?;
            serde_json::from_str(&line).map_err(|error| GameError::Serialization(error.to_string()))
        })
        .collect()
}

fn ensure_parent_dir(path: &Path) -> Result<(), GameError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| GameError::Io(error.to_string()))?;
    }
    Ok(())
}

impl GameSession {
    pub fn to_save_data(&self) -> SaveDataV1 {
        let world = self.world();
        SaveDataV1 {
            schema_version: SAVE_SCHEMA_VERSION_V1,
            seed: self.meta.seed,
            turn: self.turn,
            run_state: self.state,
            rng_state: self.rng.snapshot_state(),
            world: SavedWorldV1 {
                levels: world.levels.clone(),
                current_level: world.current_level(),
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
            },
            event_log: self.event_log.clone(),
        }
    }

    pub fn from_save_data(save: SaveDataV1) -> Result<Self, GameError> {
        if save.schema_version != SAVE_SCHEMA_VERSION_V1 {
            return Err(GameError::SaveSchemaVersionMismatch {
                expected: SAVE_SCHEMA_VERSION_V1,
                actual: save.schema_version,
            });
        }
        Ok(Self {
            meta: GameMeta { seed: save.seed },
            rng: crate::core::rng::GameRng::from_state(save.rng_state),
            turn: save.turn,
            state: save.run_state,
            world: GameWorld::from_saved_world(save.world),
            event_log: save.event_log,
        })
    }
}
