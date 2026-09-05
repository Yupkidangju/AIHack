use aihack_runtime::GameSession;

#[test]
fn runtime_world_owns_bootstrap_and_invariants() {
    let session = GameSession::try_new_for_playing(42).unwrap();
    let world = session.world();

    assert!(world.player_alive());
    assert!(world.validate_invariants().is_valid());
}
