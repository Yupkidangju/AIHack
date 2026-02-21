use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Failed to save or load game: {0}")]
    SaveLoadError(String),

    #[error("Failed to initialize game: {0}")]
    InitError(String),

    #[error("Missing required resource: {0}")]
    ResourceMissing(String),

    #[error("Invalid game state: {0}")]
    InvalidState(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type GameResult<T> = Result<T, GameError>;
