use thiserror::Error;

use crate::{
    core::position::Pos,
    domain::tile::{DoorState, TileKind},
};

/// [v0.1.0] Core 경계에서 panic 대신 반환할 오류 타입이다.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameError {
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
    #[error("invalid CLI option: {0}")]
    InvalidCliOption(String),
}

pub type GameResult<T> = Result<T, GameError>;

/// Embedded content를 시작 전에 검증할 때 반환하는 오류다.
///
/// Registry는 fallback 데이터를 사용하지 않으므로, 이 오류는 게임 시작을 중단해야
/// 하는 콘텐츠 계약 위반을 의미한다.
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
