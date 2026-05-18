use aihack::core::{ActionIntent, CommandIntent, Direction, GameSession, RunStateSummary};

#[test]
fn observation_updates_after_movement() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    let observation = session.observation();

    assert_eq!(observation.turn, 1);
    assert_eq!(observation.run_state, RunStateSummary::Playing);
    assert_eq!(observation.player_pos.x, 6);
    assert_eq!(observation.player_pos.y, 5);
    assert!(observation
        .visible_tiles
        .iter()
        .any(|tile| tile.rel.dx == 0 && tile.rel.dy == 0));
    assert_eq!(
        observation.action_space.commands.first(),
        Some(&ActionIntent::Command(CommandIntent::Wait))
    );
}

#[test]
fn observation_includes_phase7_legal_actions_and_hidden_tile_projection() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    let observation = session.observation();

    assert!(observation.legal_actions.contains(&CommandIntent::Search));
    assert!(observation
        .action_space
        .commands
        .contains(&ActionIntent::Command(CommandIntent::Search)));
    assert!(observation.legal_actions.contains(&CommandIntent::Read {
        item: aihack::core::EntityId(8)
    }));

    session
        .world
        .set_player_pos(aihack::core::Pos { x: 11, y: 5 });
    let door_observation = session.observation();
    assert!(door_observation.visible_tiles.iter().any(|tile| {
        tile.pos == aihack::core::Pos { x: 12, y: 5 }
            && tile.tile == aihack::domain::tile::TileKind::Wall
    }));

    session
        .world
        .set_player_pos(aihack::core::Pos { x: 15, y: 5 });
    let trap_observation = session.observation();
    assert!(trap_observation.visible_tiles.iter().any(|tile| {
        tile.pos == aihack::core::Pos { x: 16, y: 5 }
            && tile.tile == aihack::domain::tile::TileKind::Floor
    }));
}
