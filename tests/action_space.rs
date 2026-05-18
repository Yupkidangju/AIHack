use aihack::core::{ActionIntent, GameSession};

#[test]
fn action_space_matches_legacy_legal_actions() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let expected = observation
        .legal_actions
        .iter()
        .copied()
        .map(ActionIntent::Command)
        .collect::<Vec<_>>();
    assert_eq!(observation.action_space.commands, expected);
}
