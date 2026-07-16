use aihack_core::position::Direction;
use aihack_runtime::{
    systems::{doors, stairs, traps},
    world::GameWorld,
};

#[test]
fn runtime_exposes_environment_interaction_systems() {
    let mut world = GameWorld::fixture_without_monsters();

    let _ = traps::search(&mut world);
    assert_eq!(
        doors::door_state_in_direction(&world, Direction::East),
        None
    );
    assert!(stairs::descend(&mut world).is_err());
}
