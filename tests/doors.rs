use aihack::{
    core::{CommandIntent, Direction, GameEvent, GameSession, Pos},
    domain::tile::{DoorState, TileKind},
};

fn stand_west_of_fixture_door(session: &mut GameSession) {
    session.world.set_player_pos(Pos { x: 9, y: 5 });
}

#[test]
fn open_door_allows_movement_and_los() {
    let mut session = GameSession::new(42);
    stand_west_of_fixture_door(&mut session);

    let outcome = session.submit(CommandIntent::Open(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::DoorChanged {
            from: DoorState::Closed,
            to: DoorState::Open,
            ..
        }
    )));
    assert_eq!(
        session
            .world
            .current_map()
            .tile(Pos { x: 10, y: 5 })
            .unwrap(),
        TileKind::Door(DoorState::Open)
    );

    let movement = session.submit(CommandIntent::Move(Direction::East));
    assert!(movement.accepted);
    assert_eq!(session.world.player_pos(), Pos { x: 10, y: 5 });
}

#[test]
fn close_door_blocks_movement_and_los() {
    let mut session = GameSession::new(42);
    stand_west_of_fixture_door(&mut session);
    assert!(
        session
            .submit(CommandIntent::Open(Direction::East))
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Close(Direction::East))
            .accepted
    );

    assert_eq!(
        session
            .world
            .current_map()
            .tile(Pos { x: 10, y: 5 })
            .unwrap(),
        TileKind::Door(DoorState::Closed)
    );
    let movement = session.submit(CommandIntent::Move(Direction::East));
    assert!(!movement.accepted);
}

#[test]
fn invalid_open_close_are_rejected() {
    let mut session = GameSession::new(42);

    let open_floor = session.submit(CommandIntent::Open(Direction::East));
    assert!(!open_floor.accepted);
    assert_eq!(session.turn, 0);

    stand_west_of_fixture_door(&mut session);
    assert!(
        !session
            .submit(CommandIntent::Close(Direction::East))
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Open(Direction::East))
            .accepted
    );
    assert!(
        !session
            .submit(CommandIntent::Open(Direction::East))
            .accepted
    );
}
