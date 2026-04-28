use aihack::{
    core::{CommandIntent, Direction, GameSession, Pos},
    domain::tile::{DoorState, TileKind},
    systems::vision::{has_line_of_sight, visible_positions, DEFAULT_VISION_RADIUS},
};

#[test]
fn vision_respects_radius_8() {
    let session = GameSession::new(42);
    let visible = visible_positions(&session.world);

    assert!(visible
        .iter()
        .all(|pos| session.world.player_pos().chebyshev_distance(*pos) <= DEFAULT_VISION_RADIUS));
    assert!(!visible.contains(&Pos { x: 14, y: 14 }));
}

#[test]
fn vision_respects_door_blockers() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(Pos { x: 9, y: 5 });
    let behind_door = Pos { x: 11, y: 5 };

    assert!(!has_line_of_sight(
        &session.world,
        session.world.player_pos(),
        behind_door
    ));

    assert!(
        session
            .submit(CommandIntent::Open(Direction::East))
            .accepted
    );
    assert!(has_line_of_sight(
        &session.world,
        session.world.player_pos(),
        behind_door
    ));
}

#[test]
fn wall_blocks_los_but_wall_tile_can_be_seen() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(Pos { x: 11, y: 5 });
    let wall = Pos { x: 12, y: 5 };
    let behind_wall = Pos { x: 13, y: 5 };

    assert!(has_line_of_sight(
        &session.world,
        session.world.player_pos(),
        wall
    ));
    assert!(!has_line_of_sight(
        &session.world,
        session.world.player_pos(),
        behind_wall
    ));
}

#[test]
fn observation_contains_visible_tiles() {
    let session = GameSession::new(42);
    let observation = session.observation();

    assert_eq!(observation.schema_version, 1);
    assert_eq!(observation.seed, 42);
    assert_eq!(observation.turn, 0);
    assert_eq!(observation.player_pos, Pos { x: 5, y: 5 });
    assert!(observation.visible_tiles.iter().any(|tile| {
        tile.pos == Pos { x: 5, y: 5 }
            && tile.rel.dx == 0
            && tile.rel.dy == 0
            && tile.visible
            && tile.tile == TileKind::Floor
    }));
}

#[test]
fn observation_legal_actions_include_wait_move_and_door_actions() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    let observation = session.observation();
    assert!(observation.legal_actions.contains(&CommandIntent::Wait));
    assert!(observation
        .legal_actions
        .contains(&CommandIntent::Move(Direction::East)));

    session.world.set_player_pos(Pos { x: 9, y: 5 });
    let closed = session.observation();
    assert!(closed
        .legal_actions
        .contains(&CommandIntent::Open(Direction::East)));

    session
        .world
        .current_map_mut()
        .set_tile(Pos { x: 10, y: 5 }, TileKind::Door(DoorState::Open))
        .unwrap();
    let open = session.observation();
    assert!(open
        .legal_actions
        .contains(&CommandIntent::Close(Direction::East)));
}
