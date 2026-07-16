use aihack_runtime::world::GameWorld;

#[test]
fn runtime_world_owns_bootstrap_and_invariants() {
    let world = GameWorld::try_fixture_phase5().unwrap();

    assert!(world.player_alive());
    assert!(world.validate_invariants().is_valid());
}
