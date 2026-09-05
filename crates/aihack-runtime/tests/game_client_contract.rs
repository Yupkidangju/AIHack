use aihack_core::action::CommandIntent;
use aihack_runtime::{GameClient, GameSession};

fn submit_wait_through_adapter<C: GameClient>(
    client: &mut C,
) -> (u64, aihack_core::turn::TurnOutcome, u64) {
    let before = client.observation().turn;
    assert_eq!(client.revision().turn, before);
    let outcome = client.submit(CommandIntent::Wait);
    let after = client.observation().turn;
    (before, outcome, after)
}

#[test]
fn game_client_is_an_adapter_facing_contract() {
    let mut client = GameSession::new_for_playing(42);
    let (before, outcome, after) = submit_wait_through_adapter(&mut client);
    assert!(outcome.accepted && outcome.turn_advanced);
    assert_eq!((before, after), (0, 1));
    assert_eq!(
        client.run_state(),
        aihack_core::run_state::RunState::Playing
    );
}
