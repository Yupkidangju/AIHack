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
}

pub type GameResult<T> = Result<T, GameError>;
