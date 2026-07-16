use aihack_core::run_state::RunState;
use aihack_runtime::{observation, snapshot::GameSnapshot, world::GameWorld};

#[test]
fn runtime_owns_observation_and_snapshot_projection() {
    let world = GameWorld::fixture_without_monsters();

    let observation = observation::from_world(42, 0, RunState::Playing, &[], &world);
    let snapshot = GameSnapshot::from_world(42, 0, RunState::Playing, &[], &world);

    assert_eq!(observation.player_pos, world.player_pos());
    assert_eq!(snapshot.player_pos, world.player_pos());
    assert_eq!(snapshot.stable_hash(), snapshot.clone().stable_hash());
}
