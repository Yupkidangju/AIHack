use std::{
    fs,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use aihack_core::{error::GameError, meta::GameMeta, rng::GameRng, session::SessionState};
use cap_fs_ext::{FollowSymlinks, MetadataExt as _, OpenOptionsFollowExt as _};
use cap_std::{
    ambient_authority,
    fs::{Dir, File, Metadata, OpenOptions},
};
use cap_tempfile::TempFile;

use crate::{domain::entity::EntityStore, session::GameSession, world::GameWorld};

pub use aihack_core::save::{ReplayLineV1, SAVE_SCHEMA_VERSION_V1};
pub type SavedWorldV1 = aihack_core::save::SavedWorldV1<EntityStore>;
pub type SaveDataV1 = aihack_core::save::SaveDataV1<EntityStore>;

/// runtime artifact의 모든 파일 작업을 열린 root directory 아래로 제한한다.
pub struct ArtifactStore {
    root: Dir,
}

impl ArtifactStore {
    pub fn open(root: &Path) -> Result<Self, GameError> {
        fs::create_dir_all(root).map_err(io_error)?;
        let root = Dir::open_ambient_dir(root, ambient_authority()).map_err(io_error)?;
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

    pub fn append_replay_line(&self, path: &Path, line: &ReplayLineV1) -> Result<(), GameError> {
        let relative = self.validate_path(path)?;
        let (parent, file_name) = self.open_parent(&relative, true)?;
        let mut options = OpenOptions::new();
        options.append(true).create(true);
        configure_no_follow(&mut options);
        let file = parent.open_with(&file_name, &options).map_err(io_error)?;
        validate_open_file(&file, path)?;

        let mut writer = BufWriter::new(file);
        let encoded = serde_json::to_string(line)
            .map_err(|error| GameError::Serialization(error.to_string()))?;
        writer
            .write_all(encoded.as_bytes())
            .and_then(|_| writer.write_all(b"\n"))
            .and_then(|_| writer.flush())
            .map_err(io_error)
    }

    pub fn read_replay_lines(&self, path: &Path) -> Result<Vec<ReplayLineV1>, GameError> {
        let file = self.open_read_file(path)?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .map(|line| {
                let line = line.map_err(io_error)?;
                serde_json::from_str(&line)
                    .map_err(|error| GameError::Serialization(error.to_string()))
            })
            .collect()
    }

    pub fn save_session(&self, session: &GameSession, path: &Path) -> Result<(), GameError> {
        let payload = serde_json::to_string_pretty(&session.to_save_data())
            .map_err(|error| GameError::Serialization(error.to_string()))?;
        self.write_atomic(path, payload.as_bytes())
    }

    pub fn load_session(&self, path: &Path) -> Result<GameSession, GameError> {
        let mut reader = BufReader::new(self.open_read_file(path)?);
        let mut payload = String::new();
        std::io::Read::read_to_string(&mut reader, &mut payload).map_err(io_error)?;
        let save: SaveDataV1 = serde_json::from_str(&payload)
            .map_err(|error| GameError::Serialization(error.to_string()))?;
        GameSession::from_save_data(save)
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
        temporary.replace(file_name).map_err(io_error)
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

/// CLI artifact path를 지정 root 아래의 relative path로 제한한다.
pub fn resolve_path_in_root(root: &Path, input: &Path) -> Result<PathBuf, GameError> {
    validate_relative_path(input)?;
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
    let (store, relative) = store_for_path(path)?;
    store.append_replay_line(&relative, line)
}

pub fn read_replay_lines(path: &Path) -> Result<Vec<ReplayLineV1>, GameError> {
    let (store, relative) = store_for_path(path)?;
    store.read_replay_lines(&relative)
}

pub fn save_session_to_path(session: &GameSession, path: &Path) -> Result<(), GameError> {
    let (store, relative) = store_for_path(path)?;
    store.save_session(session, &relative)
}

pub fn load_session_from_path(path: &Path) -> Result<GameSession, GameError> {
    let (store, relative) = store_for_path(path)?;
    store.load_session(&relative)
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

fn validate_relative_path(input: &Path) -> Result<PathBuf, GameError> {
    if input.as_os_str().is_empty()
        || input.is_absolute()
        || input.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid_runtime_path(input));
    }
    Ok(input.to_path_buf())
}

fn store_for_path(path: &Path) -> Result<(ArtifactStore, PathBuf), GameError> {
    let file_name = path.file_name().ok_or_else(|| invalid_runtime_path(path))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok((ArtifactStore::open(parent)?, PathBuf::from(file_name)))
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

fn invalid_runtime_path(path: &Path) -> GameError {
    GameError::InvalidRuntimePath(path.display().to_string())
}

fn io_error(error: std::io::Error) -> GameError {
    GameError::Io(error.to_string())
}
