use aihack_core::{action::CommandIntent, run_state::RunState};
use aihack_runtime::{observation, snapshot::GameSnapshot, GameSession};

#[test]
fn runtime_owns_observation_and_snapshot_projection() {
    let mut session = GameSession::new_for_playing(42);
    let world = session.world();

    let observation = observation::from_world(42, 0, RunState::Playing, &[], world);
    let snapshot = GameSnapshot::from_world(42, 0, RunState::Playing, &[], world);

    assert_eq!(observation.player_pos, world.player_pos());
    assert_eq!(snapshot.player_pos, world.player_pos());
    let before_hash = snapshot.stable_hash();
    let before_nutrition = snapshot.nutrition;

    let outcome = session.submit(CommandIntent::Wait);
    let after = session.snapshot();
    assert!(outcome.accepted && outcome.turn_advanced);
    assert_eq!(after.turn, 1);
    assert_eq!(after.nutrition, before_nutrition - 1);
    assert_ne!(after.stable_hash(), before_hash);
}
