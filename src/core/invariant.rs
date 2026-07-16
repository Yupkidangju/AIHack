pub use aihack_core::invariant::{
    validate_world, InvariantReport, WorldInvariantError, WorldInvariantView, WORLD_INVARIANT_COUNT,
};

pub fn validate(world: &crate::core::world::GameWorld) -> InvariantReport {
    world.validate_invariants()
}
