use crate::world::GameWorld;

pub use aihack_core::score::{apply_luck, hallucination_message};

pub fn death_score(world: &GameWorld, turn: u64) -> i32 {
    aihack_core::score::death_score(world, turn)
}
