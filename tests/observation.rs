use aihack::core::{CommandIntent, Direction, GameSession};

#[test]
fn observation_updates_after_movement() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    let observation = session.observation();

    assert_eq!(observation.turn, 1);
    assert_eq!(observation.player_pos.x, 6);
    assert_eq!(observation.player_pos.y, 5);
    assert!(observation
        .visible_tiles
        .iter()
        .any(|tile| tile.rel.dx == 0 && tile.rel.dy == 0));
}
