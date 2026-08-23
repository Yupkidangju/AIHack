use thiserror::Error;

use crate::{
    domain::tile::{DoorState, TileKind},
    position::Pos,
};

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SaveValidationError {
    #[error("resource limit exceeded for {resource}: limit {limit}, actual {actual}")]
    ResourceLimit {
        resource: String,
        limit: u64,
        actual: u64,
    },
    #[error("save seed {save_seed} does not match RNG seed {rng_seed}")]
    RngSeedMismatch { save_seed: u64, rng_seed: u64 },
    #[error("invalid persisted text at event {event_index}")]
    InvalidText { event_index: usize },
    #[error("invalid persisted world: {reason}")]
    InvalidWorld { reason: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameError {
    #[error(transparent)]
    Content(#[from] ContentError),
    #[error("command rejected: {0}")]
    CommandRejected(String),
    #[error("position out of bounds: {pos:?}")]
    OutOfBounds { pos: Pos },
    #[error("movement blocked at {pos:?} by {tile:?}")]
    BlockedMovement { pos: Pos, tile: TileKind },
    #[error("no door at {pos:?}; found {tile:?}")]
    NoDoor { pos: Pos, tile: TileKind },
    #[error("invalid door state at {pos:?}; expected {expected:?}, actual {actual:?}")]
    InvalidDoorState {
        pos: Pos,
        expected: DoorState,
        actual: DoorState,
    },
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("save schema mismatch: expected {expected}, actual {actual}")]
    SaveSchemaVersionMismatch { expected: u16, actual: u16 },
    #[error("invalid save: {0}")]
    InvalidSave(#[from] SaveValidationError),
    #[error("invalid CLI option: {0}")]
    InvalidCliOption(String),
    #[error("invalid runtime path: {0}")]
    InvalidRuntimePath(String),
}

pub type GameResult<T> = Result<T, GameError>;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ContentError {
    #[error("content parse error in {file}: {message}")]
    Parse { file: String, message: String },
    #[error("duplicate content id: {id}")]
    DuplicateId { id: String },
    #[error("unknown content reference from {owner} to {target}")]
    UnknownReference { owner: String, target: String },
    #[error("invalid damage dice: {value}")]
    InvalidDice { value: String },
    #[error("invalid coordinate in {level}: ({x}, {y})")]
    InvalidCoordinate { level: String, x: i16, y: i16 },
    #[error("missing paired stairs for level: {level}")]
    MissingStairsPair { level: String },
}
