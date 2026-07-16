//! Content bootstrap과 게임 실행을 조합하고 adapter에는 안정된 client 계약만 제공한다.

pub mod bootstrap;
pub mod client;
pub mod domain;
pub mod observation;
pub mod save;
pub mod session;
pub mod snapshot;
pub mod systems;
mod transaction;
pub mod world;

pub use aihack_core::error::{ContentError, GameError};
pub use client::GameClient;
pub use session::GameSession;
