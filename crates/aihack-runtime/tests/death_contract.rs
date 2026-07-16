use aihack_core::run_state::RunState;
use aihack_runtime::{systems::death, world::GameWorld};

#[test]
fn runtime_death_keeps_a_live_player_in_the_current_state() {
    let world = GameWorld::fixture_phase5();

    assert_eq!(death::state_after_deaths(&world), RunState::Playing);
}
