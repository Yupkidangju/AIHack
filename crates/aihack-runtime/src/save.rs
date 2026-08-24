use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Component, Path, PathBuf},
};

use aihack_core::{
    domain::{
        combat::DeathCause,
        entity::{ActorKind, EntityLocation, EntityPayload, MAX_ABSOLUTE_ACTOR_STAT},
        item::ItemClass,
        player::adventurer_template,
    },
    error::{GameError, SaveValidationError},
    event::GameEvent,
    ids::LevelId,
    meta::GameMeta,
    rng::GameRng,
    run_state::RunState,
    session::SessionState,
};
use cap_fs_ext::{
    FollowSymlinks, MetadataExt as _, OpenOptionsFollowExt as _, OpenOptionsMaybeDirExt as _,
};
use cap_std::{
    ambient_authority,
    fs::{Dir, File, Metadata, OpenOptions},
};
use cap_tempfile::TempFile;

use crate::{domain::entity::EntityStore, session::GameSession, world::GameWorld};

pub use aihack_core::save::{ReplayLineV1, SAVE_SCHEMA_VERSION_V1};
pub type SavedWorldV1 = aihack_core::save::SavedWorldV1<EntityStore>;
pub type SaveDataV1 = aihack_core::save::SaveDataV1<EntityStore>;

pub const MAX_SAVE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_REPLAY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_REPLAY_LINES: usize = 100_000;
pub const MAX_REPLAY_LINE_BYTES: usize = 65_536;
pub const MAX_SAVE_EVENTS: usize = 100_000;
pub const MAX_SAVE_ENTITIES: usize = 100_000;
pub const MAX_RNG_DRAWS: u64 = 1_000_000;
pub const MAX_PERSISTED_TEXT_BYTES: usize = 512;
const MAX_SAVE_LEVELS: usize = 64;
const MAX_TOTAL_MAP_TILES: usize = 1_000_000;

/// runtime artifact의 모든 파일 작업을 열린 root directory 아래로 제한한다.
pub struct ArtifactStore {
    root: Dir,
}

impl ArtifactStore {
    pub fn open(root: &Path) -> Result<Self, GameError> {
        fs::create_dir_all(root).map_err(io_error)?;
        let root = open_root_directory(root)?;
        Ok(Self { root })
    }

    pub fn validate_path(&self, input: &Path) -> Result<PathBuf, GameError> {
        let relative = validate_relative_path(input)?;
        match self.root.symlink_metadata(&relative) {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(invalid_runtime_path(input)),
            Ok(_) => Ok(relative),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(relative),
            Err(error) => Err(io_error(error)),
        }
    }

    /// 두 상대 경로가 같은 runtime artifact를 가리키는지 lexical form과 열린 file identity로 판정한다.
    pub fn paths_refer_to_same_artifact(
        &self,
        left: &Path,
        right: &Path,
    ) -> Result<bool, GameError> {
        let left = validate_relative_path(left)?;
        let right = validate_relative_path(right)?;
        if relative_paths_equal(&left, &right) {
            return Ok(true);
        }

        let left_identity = self.file_identity_if_exists(&left)?;
        let right_identity = self.file_identity_if_exists(&right)?;
        Ok(matches!(
            (left_identity, right_identity),
            (Some(left), Some(right)) if left == right
        ))
    }

    pub fn append_replay_line(&self, path: &Path, line: &ReplayLineV1) -> Result<(), GameError> {
        self.append_replay_lines(path, std::slice::from_ref(line))
    }

    /// 여러 replay line을 한 번의 bounded read와 atomic rewrite로 추가한다.
    pub fn append_replay_lines(
        &self,
        path: &Path,
        appended: &[ReplayLineV1],
    ) -> Result<(), GameError> {
        let relative = self.validate_path(path)?;
        let mut lines = match self.root.symlink_metadata(&relative) {
            Ok(_) => self.read_replay_lines(&relative)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(io_error(error)),
        };
        for line in appended {
            validate_events(&line.outcome.events)?;
        }
        lines.extend_from_slice(appended);
        validate_count("replay lines", lines.len(), MAX_REPLAY_LINES)?;
        let payload = encode_replay_lines(&lines)?;
        self.write_atomic(&relative, &payload)
    }

    pub fn read_replay_lines(&self, path: &Path) -> Result<Vec<ReplayLineV1>, GameError> {
        let file = self.open_read_file(path)?;
        let metadata_len = file.metadata().map_err(io_error)?.len();
        if metadata_len > MAX_REPLAY_BYTES as u64 {
            return Err(resource_limit(
                "replay bytes",
                MAX_REPLAY_BYTES as u64,
                metadata_len,
            ));
        }
        let mut reader = BufReader::new(file).take(MAX_REPLAY_BYTES as u64 + 1);
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let mut total_bytes = 0usize;
        loop {
            buffer.clear();
            let bytes = reader.read_line(&mut buffer).map_err(io_error)?;
            if bytes == 0 {
                break;
            }
            total_bytes = total_bytes
                .checked_add(bytes)
                .ok_or_else(|| resource_limit("replay bytes", MAX_REPLAY_BYTES as u64, u64::MAX))?;
            validate_count("replay bytes", total_bytes, MAX_REPLAY_BYTES)?;
            let encoded = buffer.trim_end_matches(['\r', '\n']);
            validate_count("replay line bytes", encoded.len(), MAX_REPLAY_LINE_BYTES)?;
            validate_count("replay lines", lines.len() + 1, MAX_REPLAY_LINES)?;
            let line: ReplayLineV1 = serde_json::from_str(encoded)
                .map_err(|error| GameError::Serialization(error.to_string()))?;
            validate_events(&line.outcome.events)?;
            lines.push(line);
        }
        Ok(lines)
    }

    pub fn save_session(&self, session: &GameSession, path: &Path) -> Result<(), GameError> {
        let save = session.to_save_data();
        validate_save_data_with_registry(&save, session.world().content_registry())?;
        let payload = encode_save_data(&save)?;
        self.write_atomic(path, &payload)
    }

    pub fn load_session(&self, path: &Path) -> Result<GameSession, GameError> {
        self.load_session_with_registry(path, aihack_content::registry()?)
    }

    pub fn load_session_with_registry(
        &self,
        path: &Path,
        registry: &aihack_content::ContentRegistry,
    ) -> Result<GameSession, GameError> {
        let payload = self.read_bounded(path, MAX_SAVE_BYTES, "save bytes")?;
        let save: SaveDataV1 = serde_json::from_str(&payload)
            .map_err(|error| GameError::Serialization(error.to_string()))?;
        GameSession::from_save_data_with_registry(save, registry)
    }

    pub fn write_atomic(&self, path: &Path, payload: &[u8]) -> Result<(), GameError> {
        let relative = self.validate_path(path)?;
        let (parent, file_name) = self.open_parent(&relative, true)?;
        validate_destination(&parent, &file_name, path)?;

        // 임시 파일은 capability가 가리키는 동일 directory 안에서 원자적으로 생성된다.
        let mut temporary = TempFile::new(&parent).map_err(io_error)?;
        set_platform_writable_permissions(temporary.as_file())?;
        validate_new_temporary_file(temporary.as_file(), path)?;
        temporary.write_all(payload).map_err(io_error)?;
        temporary.flush().map_err(io_error)?;
        temporary.as_file().sync_all().map_err(io_error)?;
        validate_new_temporary_file(temporary.as_file(), path)?;

        // 공격자가 검증 뒤 destination을 link로 바꾼 경우 replace 직전에도 중단한다.
        validate_destination(&parent, &file_name, path)?;
        temporary.replace(file_name).map_err(io_error)?;
        sync_parent_directory(&parent)
    }

    fn open_read_file(&self, path: &Path) -> Result<File, GameError> {
        let relative = self.validate_path(path)?;
        let (parent, file_name) = self.open_parent(&relative, false)?;
        let mut options = OpenOptions::new();
        options.read(true);
        configure_no_follow(&mut options);
        let file = parent.open_with(file_name, &options).map_err(io_error)?;
        validate_open_file(&file, path)?;
        Ok(file)
    }

    fn file_identity_if_exists(&self, path: &Path) -> Result<Option<(u64, u64)>, GameError> {
        match self.root.symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(invalid_runtime_path(path)),
            Ok(_) => {
                let metadata = self.open_read_file(path)?.metadata().map_err(io_error)?;
                Ok(Some((metadata.dev(), metadata.ino())))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_error(error)),
        }
    }

    fn read_bounded(&self, path: &Path, limit: usize, resource: &str) -> Result<String, GameError> {
        let file = self.open_read_file(path)?;
        let metadata_len = file.metadata().map_err(io_error)?.len();
        if metadata_len > limit as u64 {
            return Err(resource_limit(resource, limit as u64, metadata_len));
        }
        let mut reader = BufReader::new(file).take(limit as u64 + 1);
        let mut payload = Vec::with_capacity(metadata_len as usize);
        reader.read_to_end(&mut payload).map_err(io_error)?;
        if payload.len() > limit {
            return Err(resource_limit(resource, limit as u64, payload.len() as u64));
        }
        String::from_utf8(payload)
            .map_err(|error| GameError::Serialization(format!("artifact is not UTF-8: {error}")))
    }

    fn open_parent(&self, path: &Path, create: bool) -> Result<(Dir, PathBuf), GameError> {
        let file_name = path.file_name().ok_or_else(|| invalid_runtime_path(path))?;
        let parent_path = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if create {
            self.root.create_dir_all(parent_path).map_err(io_error)?;
        }
        let parent = self.root.open_dir(parent_path).map_err(io_error)?;
        Ok((parent, PathBuf::from(file_name)))
    }
}

fn encode_replay_lines(lines: &[ReplayLineV1]) -> Result<Vec<u8>, GameError> {
    let mut payload = Vec::new();
    for replay_line in lines {
        let encoded = serde_json::to_vec(replay_line)
            .map_err(|error| GameError::Serialization(error.to_string()))?;
        validate_count("replay line bytes", encoded.len(), MAX_REPLAY_LINE_BYTES)?;
        let next_len = payload
            .len()
            .checked_add(encoded.len() + 1)
            .ok_or_else(|| resource_limit("replay bytes", MAX_REPLAY_BYTES as u64, u64::MAX))?;
        if next_len > MAX_REPLAY_BYTES {
            return Err(resource_limit(
                "replay bytes",
                MAX_REPLAY_BYTES as u64,
                next_len as u64,
            ));
        }
        payload.extend_from_slice(&encoded);
        payload.push(b'\n');
    }
    Ok(payload)
}

fn encode_save_data(save: &SaveDataV1) -> Result<Vec<u8>, GameError> {
    let mut writer = CappedBuffer::new(MAX_SAVE_BYTES);
    if let Err(error) = serde_json::to_writer_pretty(&mut writer, save) {
        if let Some(actual) = writer.limit_exceeded_at {
            return Err(resource_limit("save bytes", MAX_SAVE_BYTES as u64, actual));
        }
        return Err(GameError::Serialization(error.to_string()));
    }
    Ok(writer.bytes)
}

struct CappedBuffer {
    bytes: Vec<u8>,
    limit: usize,
    limit_exceeded_at: Option<u64>,
}

impl CappedBuffer {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            limit_exceeded_at: None,
        }
    }
}

impl Write for CappedBuffer {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let next = self.bytes.len().checked_add(buffer.len()).ok_or_else(|| {
            self.limit_exceeded_at = Some(u64::MAX);
            std::io::Error::other("save byte budget overflow")
        })?;
        if next > self.limit {
            self.limit_exceeded_at = Some(next as u64);
            return Err(std::io::Error::other("save byte budget exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn open_root_directory(path: &Path) -> Result<Dir, GameError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invalid_runtime_path(path));
    }

    let Some(file_name) = path.file_name().filter(|name| !name.is_empty()) else {
        return Dir::open_ambient_dir(path, ambient_authority()).map_err(io_error);
    };
    let parent_path = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = Dir::open_ambient_dir(parent_path, ambient_authority()).map_err(io_error)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .maybe_dir(true)
        .follow(FollowSymlinks::No);
    let file = parent.open_with(file_name, &options).map_err(io_error)?;
    let opened = file.metadata().map_err(io_error)?;
    if opened.file_type().is_symlink() || !opened.is_dir() {
        return Err(invalid_runtime_path(path));
    }
    Ok(Dir::from_std_file(file.into_std()))
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
        Self::from_save_data_with_registry(save, aihack_content::registry()?)
    }

    pub fn from_save_data_with_registry(
        save: SaveDataV1,
        registry: &aihack_content::ContentRegistry,
    ) -> Result<Self, GameError> {
        if save.schema_version != SAVE_SCHEMA_VERSION_V1 {
            return Err(GameError::SaveSchemaVersionMismatch {
                expected: SAVE_SCHEMA_VERSION_V1,
                actual: save.schema_version,
            });
        }
        validate_save_data_with_registry(&save, registry)?;
        Ok(Self {
            inner: SessionState {
                meta: GameMeta { seed: save.seed },
                rng: GameRng::from_state(save.rng_state),
                turn: save.turn,
                state: save.run_state,
                world: GameWorld::from_saved_world_with_registry(save.world, registry)?,
                event_log: save.event_log,
            },
            transaction_aborted: false,
        })
    }
}

pub(crate) fn validate_save_data_with_registry(
    save: &SaveDataV1,
    registry: &aihack_content::ContentRegistry,
) -> Result<(), GameError> {
    validate_count("event log", save.event_log.len(), MAX_SAVE_EVENTS)?;
    validate_count(
        "entity store",
        save.world.entities.entities().len(),
        MAX_SAVE_ENTITIES,
    )?;
    validate_count(
        "level registry",
        save.world.levels.levels.len(),
        MAX_SAVE_LEVELS,
    )?;
    if save.rng_state.draws > MAX_RNG_DRAWS {
        return Err(resource_limit(
            "RNG draws",
            MAX_RNG_DRAWS,
            save.rng_state.draws,
        ));
    }
    if save.seed != save.rng_state.seed {
        return Err(SaveValidationError::RngSeedMismatch {
            save_seed: save.seed,
            rng_seed: save.rng_state.seed,
        }
        .into());
    }
    validate_events(&save.event_log)?;
    validate_saved_world_for_run_state(&save.world, save.run_state, registry)?;
    validate_consumer_safe_arithmetic(save)
}

fn validate_consumer_safe_arithmetic(save: &SaveDataV1) -> Result<(), GameError> {
    if save.turn == u64::MAX {
        return invalid_world("turn cannot be incremented safely".to_string());
    }
    let depth = i128::from(save.world.current_level.depth);
    let inventory_value =
        save.world
            .inventory
            .entries
            .iter()
            .try_fold(0_i128, |total, entry| {
                let price = save
                    .world
                    .entities
                    .item_data(entry.item)
                    .map(|data| i128::from(data.base_price))
                    .ok_or_else(|| invalid_world_error("inventory score item is missing"))?;
                total
                    .checked_add(price)
                    .ok_or_else(|| invalid_world_error("inventory score overflows"))
            })?;
    let score = i128::from(save.world.gold)
        + i128::from(save.world.kill_count) * 10
        + depth * 100
        + inventory_value
        - i128::from(save.turn / 10);
    if !(i128::from(i32::MIN)..=i128::from(i32::MAX)).contains(&score) {
        return invalid_world("persisted score inputs exceed the i32 result range".to_string());
    }
    Ok(())
}

fn validate_events(events: &[GameEvent]) -> Result<(), GameError> {
    for (event_index, event) in events.iter().enumerate() {
        let text = match event {
            GameEvent::CommandRejected { reason } => Some(reason.as_str()),
            GameEvent::Message { text, .. } => Some(text.as_str()),
            _ => None,
        };
        if text.is_some_and(|value| {
            value.len() > MAX_PERSISTED_TEXT_BYTES || value.chars().any(char::is_control)
        }) {
            return Err(SaveValidationError::InvalidText { event_index }.into());
        }
    }
    Ok(())
}

pub(crate) fn validate_saved_world(world: &SavedWorldV1) -> Result<(), GameError> {
    validate_saved_world_inner(world, false, None, aihack_content::registry()?)
}

pub(crate) fn validate_depleted_saved_world(world: &SavedWorldV1) -> Result<(), GameError> {
    validate_saved_world_inner(world, true, None, aihack_content::registry()?)
}

fn validate_saved_world_for_run_state(
    world: &SavedWorldV1,
    run_state: RunState,
    registry: &aihack_content::ContentRegistry,
) -> Result<(), GameError> {
    validate_saved_world_inner(world, false, Some(run_state), registry)
}

fn validate_saved_world_inner(
    world: &SavedWorldV1,
    allow_depleted_actor: bool,
    run_state: Option<RunState>,
    registry: &aihack_content::ContentRegistry,
) -> Result<(), GameError> {
    let mut level_ids = HashSet::new();
    let mut total_tiles = 0usize;
    for level in &world.levels.levels {
        if !level_ids.insert(level.id) {
            return invalid_world(format!("duplicate level id: {:?}", level.id));
        }
        if level.map.width <= 0 || level.map.height <= 0 {
            return invalid_world(format!("level {:?} has non-positive dimensions", level.id));
        }
        let expected = usize::try_from(i32::from(level.map.width) * i32::from(level.map.height))
            .map_err(|_| invalid_world_error("map dimensions overflow"))?;
        if level.map.tile_count() != expected {
            return invalid_world(format!(
                "level {:?} tile count mismatch: expected {expected}, actual {}",
                level.id,
                level.map.tile_count()
            ));
        }
        total_tiles = total_tiles
            .checked_add(expected)
            .ok_or_else(|| invalid_world_error("total map tile count overflow"))?;
    }
    let expected_level_ids = registry
        .levels()
        .map(|level| LevelId::main(level.depth))
        .collect::<HashSet<_>>();
    if level_ids != expected_level_ids {
        return invalid_world("persisted level IDs do not match the active registry".to_string());
    }
    validate_count("map tiles", total_tiles, MAX_TOTAL_MAP_TILES)?;
    if !world.levels.contains(world.current_level) {
        return invalid_world("current level is missing".to_string());
    }

    let entities = world.entities.entities();
    let mut entity_ids = HashSet::new();
    let mut max_id = 0u32;
    let mut player_count = 0usize;
    for entity in entities {
        if entity.id.0 == 0 || !entity_ids.insert(entity.id) {
            return invalid_world(format!("invalid or duplicate entity id: {}", entity.id.0));
        }
        max_id = max_id.max(entity.id.0);
        match &entity.payload {
            EntityPayload::Actor {
                kind,
                location,
                stats,
                alive,
                ..
            } => {
                if *kind == ActorKind::Player {
                    player_count += 1;
                }
                validate_actor_stats(entity.id.0, stats, *alive, allow_depleted_actor)?;
                validate_map_location(world, entity.id.0, *location)?;
            }
            EntityPayload::Item {
                kind,
                data,
                location,
                charges,
                ..
            } => {
                if data.kind != *kind {
                    return invalid_world(format!("item {} kind/data mismatch", entity.id.0));
                }
                let expected = crate::domain::item::try_item_data_from_registry(*kind, registry)?;
                if !item_data_matches_registry(data, &expected) {
                    return invalid_world(format!(
                        "item {} data does not match the active content registry",
                        entity.id.0
                    ));
                }
                if charges.is_some() != data.max_charges.is_some()
                    || charges
                        .zip(data.max_charges)
                        .is_some_and(|(value, max)| value > max)
                {
                    return invalid_world(format!("item {} has invalid charges", entity.id.0));
                }
                if matches!(location, EntityLocation::OnMap { .. }) {
                    validate_map_location(world, entity.id.0, *location)?;
                }
            }
        }
    }
    let expected_next_id = max_id
        .checked_add(1)
        .ok_or_else(|| invalid_world_error("entity allocator has no successor ID"))?;
    if world.entities.next_id() != expected_next_id {
        return invalid_world(
            "entity allocator next_id is not the exact persisted ID successor".to_string(),
        );
    }
    if player_count != 1 {
        return invalid_world(format!("expected exactly one player, found {player_count}"));
    }
    let Some(player) = world.entities.get(world.player_id) else {
        return invalid_world(format!("player {} is missing", world.player_id.0));
    };
    if player.actor_kind() != Some(ActorKind::Player) {
        return invalid_world(format!("entity {} is not the player", world.player_id.0));
    }
    let player_alive = player.is_alive_actor();
    if let Some(run_state) = run_state {
        match run_state {
            RunState::GameOver {
                cause: DeathCause::Combat { attacker },
                ..
            } if player_alive && attacker.0 == 0 => {}
            RunState::GameOver { .. } if player_alive => {
                return invalid_world("living player has a non-quit GameOver state".to_string());
            }
            RunState::GameOver { .. } => {}
            _ if !player_alive => {
                return invalid_world("dead player is not in GameOver state".to_string());
            }
            _ => {}
        }
    }
    let Some((player_level, player_pos)) = world.entities.actor_location(world.player_id) else {
        return invalid_world("player has no map location".to_string());
    };
    if player_level != world.current_level
        || !world
            .levels
            .map(player_level)
            .is_some_and(|map| map.contains(player_pos))
    {
        return invalid_world("player location/current level mismatch".to_string());
    }
    validate_inventory(world)
}

fn item_data_matches_registry(
    persisted: &aihack_core::domain::item::ItemData,
    expected: &aihack_core::domain::item::ItemData,
) -> bool {
    let attack_matches = match (persisted.attack_profile, expected.attack_profile) {
        (Some(persisted), Some(expected)) => {
            persisted.hit_bonus == expected.hit_bonus && persisted.damage == expected.damage
        }
        (None, None) => true,
        _ => false,
    };
    persisted.kind == expected.kind
        && persisted.class == expected.class
        && persisted.glyph == expected.glyph
        && persisted.weight == expected.weight
        && persisted.base_price == expected.base_price
        && persisted.ac_bonus == expected.ac_bonus
        && attack_matches
        && persisted.consumable_effect == expected.consumable_effect
        && persisted.wand_effect == expected.wand_effect
        && persisted.max_charges == expected.max_charges
        && persisted.nutrition == expected.nutrition
}

fn validate_actor_stats(
    entity_id: u32,
    stats: &aihack_core::domain::entity::ActorStats,
    alive: bool,
    allow_depleted_actor: bool,
) -> Result<(), GameError> {
    let bounded = [
        stats.hp,
        stats.max_hp,
        stats.ac,
        stats.hit_bonus,
        stats.damage_bonus,
        stats.damage_reduction,
        stats.weapon_hit_bonus,
        stats.speed,
    ]
    .into_iter()
    .all(|value| i32::from(value).abs() <= MAX_ABSOLUTE_ACTOR_STAT);
    let dice_valid = (stats.damage.dice == 0 && stats.damage.sides == 0)
        || (stats.damage.dice > 0
            && stats.damage.sides > 0
            && i32::from(stats.damage.dice) <= MAX_ABSOLUTE_ACTOR_STAT
            && i32::from(stats.damage.sides) <= MAX_ABSOLUTE_ACTOR_STAT);
    let hp_relation_valid = stats.hp <= stats.max_hp
        && if allow_depleted_actor && alive && stats.hp <= 0 {
            true
        } else {
            alive == (stats.hp > 0)
        };
    if !bounded || stats.max_hp <= 0 || !hp_relation_valid || !dice_valid {
        return invalid_world(format!(
            "actor {entity_id} has invalid stats: hp={} max_hp={} alive={alive} bounded={bounded} hp_relation={hp_relation_valid} dice={dice_valid}",
            stats.hp, stats.max_hp
        ));
    }
    Ok(())
}

fn validate_map_location(
    world: &SavedWorldV1,
    entity_id: u32,
    location: EntityLocation,
) -> Result<(), GameError> {
    let EntityLocation::OnMap { level, pos } = location else {
        return invalid_world(format!(
            "actor/item {entity_id} has an invalid map location"
        ));
    };
    if !world.levels.map(level).is_some_and(|map| map.contains(pos)) {
        return invalid_world(format!("entity {entity_id} is outside a persisted map"));
    }
    Ok(())
}

fn validate_inventory(world: &SavedWorldV1) -> Result<(), GameError> {
    if world.inventory.owner != world.player_id || world.inventory.next_letter_index > 26 {
        return invalid_world("inventory owner or next letter is invalid".to_string());
    }
    let mut items = HashSet::new();
    let mut letters = HashSet::new();
    for entry in &world.inventory.entries {
        if !items.insert(entry.item)
            || !letters.insert(entry.letter)
            || !entry.letter.0.is_ascii_lowercase()
        {
            return invalid_world("inventory contains a duplicate or invalid entry".to_string());
        }
        let Some((_, _, location, assigned_letter, _)) = world
            .entities
            .get(entry.item)
            .and_then(|entity| entity.item())
        else {
            return invalid_world(format!("inventory item {} is missing", entry.item.0));
        };
        if location
            != (EntityLocation::Inventory {
                owner: world.player_id,
            })
            || assigned_letter != Some(entry.letter)
        {
            return invalid_world(format!("inventory item {} relation mismatch", entry.item.0));
        }
    }
    if usize::from(world.inventory.next_letter_index) < world.inventory.entries.len() {
        return invalid_world("inventory next letter precedes active entries".to_string());
    }
    for entity in world.entities.entities() {
        if let Some((_, _, EntityLocation::Inventory { owner }, _, _)) = entity.item() {
            if owner != world.player_id {
                return invalid_world(format!(
                    "inventory item {} has unsupported owner {}",
                    entity.id.0, owner.0
                ));
            }
            if !items.contains(&entity.id) {
                return invalid_world(format!("inventory item {} is not indexed", entity.id.0));
            }
        }
    }
    validate_equipped_item(world, world.inventory.equipped_melee, ItemClass::Weapon)?;
    validate_equipped_item(world, world.inventory.equipped_body, ItemClass::Armor)?;
    let bonus = if let Some(body) = world.inventory.equipped_body {
        world
            .entities
            .item_data(body)
            .map(|data| data.ac_bonus)
            .ok_or_else(|| invalid_world_error("equipped body item is missing"))?
    } else {
        0
    };
    let player_ac = world
        .entities
        .actor_stats(world.player_id)
        .map(|stats| stats.ac)
        .ok_or_else(|| invalid_world_error("player stats are missing"))?;
    let derived_ac = i32::from(adventurer_template().ac)
        .checked_sub(i32::from(bonus))
        .and_then(|value| i16::try_from(value).ok())
        .ok_or_else(|| invalid_world_error("equipped body armor AC overflows"))?;
    if player_ac != derived_ac {
        return invalid_world("equipped body armor and player AC disagree".to_string());
    }
    Ok(())
}

fn validate_equipped_item(
    world: &SavedWorldV1,
    item: Option<aihack_core::ids::EntityId>,
    expected_class: ItemClass,
) -> Result<(), GameError> {
    let Some(item) = item else {
        return Ok(());
    };
    if !world.inventory.contains(item)
        || world
            .entities
            .item_data(item)
            .is_none_or(|data| data.class != expected_class)
    {
        return invalid_world(format!("equipped item {} is invalid", item.0));
    }
    Ok(())
}

fn validate_count(resource: &str, actual: usize, limit: usize) -> Result<(), GameError> {
    if actual > limit {
        return Err(resource_limit(resource, limit as u64, actual as u64));
    }
    Ok(())
}

fn resource_limit(resource: &str, limit: u64, actual: u64) -> GameError {
    SaveValidationError::ResourceLimit {
        resource: resource.to_string(),
        limit,
        actual,
    }
    .into()
}

fn invalid_world(reason: String) -> Result<(), GameError> {
    Err(invalid_world_error(&reason))
}

fn invalid_world_error(reason: &str) -> GameError {
    SaveValidationError::InvalidWorld {
        reason: reason.to_string(),
    }
    .into()
}

fn validate_relative_path(input: &Path) -> Result<PathBuf, GameError> {
    if input.as_os_str().is_empty() || input.is_absolute() {
        return Err(invalid_runtime_path(input));
    }
    let mut normalized = PathBuf::new();
    for component in input.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => {
                validate_platform_path_component(part, input)?;
                normalized.push(part);
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(invalid_runtime_path(input));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(invalid_runtime_path(input));
    }
    Ok(normalized)
}

#[cfg(windows)]
fn validate_platform_path_component(
    component: &std::ffi::OsStr,
    display: &Path,
) -> Result<(), GameError> {
    let Some(value) = component.to_str() else {
        return Err(invalid_runtime_path(display));
    };
    if value.ends_with(['.', ' '])
        || value.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
        })
    {
        return Err(invalid_runtime_path(display));
    }
    let device_stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    let reserved = matches!(
        device_stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || device_stem
        .strip_prefix("COM")
        .or_else(|| device_stem.strip_prefix("LPT"))
        .is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        });
    if reserved {
        return Err(invalid_runtime_path(display));
    }
    Ok(())
}

#[cfg(not(windows))]
fn validate_platform_path_component(
    _component: &std::ffi::OsStr,
    _display: &Path,
) -> Result<(), GameError> {
    Ok(())
}

fn relative_paths_equal(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn validate_destination(parent: &Dir, file_name: &Path, display: &Path) -> Result<(), GameError> {
    match parent.symlink_metadata(file_name) {
        Ok(metadata) => validate_metadata(&metadata, display),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

fn validate_open_file(file: &File, display: &Path) -> Result<(), GameError> {
    let metadata = file.metadata().map_err(io_error)?;
    validate_metadata(&metadata, display)
}

fn validate_new_temporary_file(file: &File, display: &Path) -> Result<(), GameError> {
    let metadata = file.metadata().map_err(io_error)?;
    // Linux O_TMPFILE은 directory entry가 생기기 전까지 link count가 0이다.
    if !metadata.is_file() || metadata.nlink() > 1 {
        return Err(invalid_runtime_path(display));
    }
    Ok(())
}

fn validate_metadata(metadata: &Metadata, display: &Path) -> Result<(), GameError> {
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(invalid_runtime_path(display));
    }
    Ok(())
}

fn configure_no_follow(options: &mut OpenOptions) {
    options.follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
}

fn set_platform_writable_permissions(file: &File) -> Result<(), GameError> {
    let mut permissions = file.metadata().map_err(io_error)?.permissions();
    // Windows는 parent directory DACL을 상속하므로 read-only 속성만 해제한다.
    permissions.set_readonly(false);
    #[cfg(unix)]
    {
        use cap_std::fs::PermissionsExt as _;
        // Unix에서는 save payload가 다른 계정에 노출되지 않도록 mode 0600을 강제한다.
        permissions.set_mode(0o600);
    }
    file.set_permissions(permissions).map_err(io_error)
}

fn sync_parent_directory(parent: &Dir) -> Result<(), GameError> {
    #[cfg(unix)]
    {
        // cap_std::Dir는 Linux에서 O_PATH handle일 수 있어 직접 fsync하면 EBADF가 된다.
        // 같은 capability 아래의 `.`을 read-only File로 다시 열어 sync 가능한 descriptor를 얻는다.
        let mut options = OpenOptions::new();
        options.read(true).maybe_dir(true);
        let directory = parent
            .open_with(Path::new("."), &options)
            .map_err(io_error)?;
        if !directory.metadata().map_err(io_error)?.is_dir() {
            return Err(invalid_runtime_path(Path::new(".")));
        }
        directory.sync_all().map_err(io_error)?;
    }
    #[cfg(windows)]
    {
        let _ = parent;
    }
    Ok(())
}

fn invalid_runtime_path(path: &Path) -> GameError {
    GameError::InvalidRuntimePath(path.display().to_string())
}

fn io_error(error: std::io::Error) -> GameError {
    GameError::Io(error.to_string())
}

#[cfg(test)]
mod budget_tests {
    use super::{
        validate_count, CappedBuffer, MAX_REPLAY_BYTES, MAX_REPLAY_LINES, MAX_REPLAY_LINE_BYTES,
        MAX_SAVE_ENTITIES, MAX_SAVE_EVENTS,
    };
    use std::io::Write as _;

    #[test]
    fn cardinality_helpers_accept_each_limit_and_reject_limit_plus_one() {
        for (resource, limit) in [
            ("save events", MAX_SAVE_EVENTS),
            ("save entities", MAX_SAVE_ENTITIES),
            ("replay bytes", MAX_REPLAY_BYTES),
            ("replay lines", MAX_REPLAY_LINES),
            ("replay line bytes", MAX_REPLAY_LINE_BYTES),
        ] {
            assert!(validate_count(resource, limit.saturating_sub(1), limit).is_ok());
            assert!(validate_count(resource, limit, limit).is_ok());
            assert!(validate_count(resource, limit + 1, limit).is_err());
        }
    }

    #[test]
    fn save_serialization_buffer_accepts_exact_bytes_and_rejects_limit_plus_one() {
        let mut exact = CappedBuffer::new(4);
        assert!(exact.write_all(b"1234").is_ok());
        assert_eq!(exact.bytes, b"1234");

        let mut plus_one = CappedBuffer::new(4);
        assert!(plus_one.write_all(b"12345").is_err());
        assert_eq!(plus_one.limit_exceeded_at, Some(5));
        assert!(plus_one.bytes.is_empty());
    }
}
