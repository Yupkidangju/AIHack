use aihack_core::{action::CommandIntent, position::Direction};
use aihack_runtime::GameSession;

#[test]
fn runtime_movement_updates_the_player_location() {
    let mut session = GameSession::new_for_playing(42);
    let before = session.world().player_pos();

    let outcome = session.submit(CommandIntent::Move(Direction::West));

    assert!(outcome.accepted);
    assert_eq!(
        session.world().player_pos(),
        before.offset(Direction::West.delta())
    );
}
