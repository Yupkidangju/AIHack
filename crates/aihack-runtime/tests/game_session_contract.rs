use aihack_ai_contract::ClientRevision;
use aihack_core::{action::CommandIntent, run_state::RunState};
use aihack_runtime::{GameClient, GameSession};

#[test]
fn runtime_game_session_implements_the_adapter_contract() {
    let mut session = GameSession::new_for_playing(42);
    let before = session.revision();

    let outcome = GameClient::submit(&mut session, CommandIntent::Wait);

    assert_eq!(before.turn, 0);
    assert!(outcome.accepted);
    assert_eq!(session.run_state(), RunState::Playing);
    assert_eq!(
        session.revision(),
        ClientRevision {
            turn: 1,
            snapshot_hash: outcome.snapshot_hash,
        }
    );
}
