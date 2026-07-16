use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use aihack_core::{error::GameError, meta::GameMeta, rng::GameRng, session::SessionState};

use crate::{domain::entity::EntityStore, session::GameSession, world::GameWorld};

pub use aihack_core::save::{ReplayLineV1, SAVE_SCHEMA_VERSION_V1};
pub type SavedWorldV1 = aihack_core::save::SavedWorldV1<EntityStore>;
pub type SaveDataV1 = aihack_core::save::SaveDataV1<EntityStore>;

/// CLI artifact path를 지정 root 아래의 relative path로 제한한다.
pub fn resolve_path_in_root(root: &Path, input: &Path) -> Result<PathBuf, GameError> {
    if input.is_absolute()
        || input.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(GameError::InvalidRuntimePath(input.display().to_string()));
    }
    let root = fs::canonicalize(root).map_err(|error| GameError::Io(error.to_string()))?;
    let candidate = root.join(input);
    match fs::symlink_metadata(&candidate) {
        Ok(_) => {
            let existing = fs::canonicalize(&candidate)
                .map_err(|_| GameError::InvalidRuntimePath(input.display().to_string()))?;
            if !existing.starts_with(&root) {
                return Err(GameError::InvalidRuntimePath(input.display().to_string()));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(GameError::Io(error.to_string())),
    }
    if let Some(parent) = candidate.parent().filter(|parent| parent.exists()) {
        let parent = fs::canonicalize(parent).map_err(|error| GameError::Io(error.to_string()))?;
        if !parent.starts_with(&root) {
            return Err(GameError::InvalidRuntimePath(input.display().to_string()));
        }
    }
    Ok(candidate)
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

pub fn save_session_to_path(session: &GameSession, path: &Path) -> Result<(), GameError> {
    ensure_parent_dir(path)?;
    let payload = serde_json::to_string_pretty(&session.to_save_data())
        .map_err(|error| GameError::Serialization(error.to_string()))?;
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path).map_err(|error| GameError::Io(error.to_string()))?;
    file.write_all(payload.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| GameError::Io(error.to_string()))?;
    fs::rename(&temp_path, path).map_err(|error| GameError::Io(error.to_string()))
}

pub fn load_session_from_path(path: &Path) -> Result<GameSession, GameError> {
    let payload = fs::read_to_string(path).map_err(|error| GameError::Io(error.to_string()))?;
    let save: SaveDataV1 = serde_json::from_str(&payload)
        .map_err(|error| GameError::Serialization(error.to_string()))?;
    GameSession::from_save_data(save)
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
            world: SavedWorldV1::from(world.state()),
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
            inner: SessionState {
                meta: GameMeta { seed: save.seed },
                rng: GameRng::from_state(save.rng_state),
                turn: save.turn,
                state: save.run_state,
                world: GameWorld::from_saved_world(save.world),
                event_log: save.event_log,
            },
        })
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), GameError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| GameError::Io(error.to_string()))?;
    }
    Ok(())
}
