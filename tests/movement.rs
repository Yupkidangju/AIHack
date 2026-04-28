use aihack::{
    core::{CommandIntent, Direction, GameSession, Pos},
    domain::tile::TileKind,
};

#[test]
fn movement_to_floor_advances_turn() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(session.world.player_pos(), Pos { x: 6, y: 5 });
    assert_eq!(session.turn, 1);
}

#[test]
fn movement_blockers_do_not_advance_turn() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 9, y: 5 });
    let before_hash = session.snapshot().stable_hash();
    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.world.player_pos(), Pos { x: 9, y: 5 });
    assert_eq!(session.turn, 0);
    assert_eq!(outcome.snapshot_hash, before_hash);
}

#[test]
fn wall_and_out_of_bounds_movement_are_rejected() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 1, y: 1 });

    let wall = session.submit(CommandIntent::Move(Direction::North));
    assert!(!wall.accepted);
    assert_eq!(session.world.player_pos(), Pos { x: 1, y: 1 });

    session.world.set_player_pos(Pos { x: 0, y: 1 });
    let out_of_bounds = session.submit(CommandIntent::Move(Direction::West));
    assert!(!out_of_bounds.accepted);
    assert_eq!(session.turn, 0);
}

#[test]
fn diagonal_corner_cutting_is_rejected() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 5, y: 5 });
    session
        .world
        .current_map_mut()
        .set_tile(Pos { x: 5, y: 4 }, TileKind::Wall)
        .unwrap();
    session
        .world
        .current_map_mut()
        .set_tile(Pos { x: 6, y: 4 }, TileKind::Floor)
        .unwrap();

    let outcome = session.submit(CommandIntent::Move(Direction::NorthEast));

    assert!(!outcome.accepted);
    assert_eq!(session.world.player_pos(), Pos { x: 5, y: 5 });
    assert_eq!(session.turn, 0);
}
