use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, Pos},
    domain::{
        entity::EntityLocation,
        tile::{TileKind, TrapKind},
    },
};

#[test]
fn search_reveals_hidden_door() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 11, y: 5 });

    let outcome = session.submit(CommandIntent::Search);

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos: Pos { x: 12, y: 5 },
            tile: TileKind::Door(_),
        }
    )));
    assert_eq!(
        session.world.current_map().tile(Pos { x: 12, y: 5 }),
        Ok(TileKind::Door(aihack::domain::tile::DoorState::Closed))
    );
}

#[test]
fn search_reveals_hidden_trap() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 15, y: 5 });

    let outcome = session.submit(CommandIntent::Search);

    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos: Pos { x: 16, y: 5 },
            tile: TileKind::Trap(TrapKind::Pit),
        }
    )));
}

#[test]
fn hidden_trap_triggers_on_entry() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 15, y: 5 });
    let before_hp = session
        .world
        .entities
        .actor_stats(session.world.player_id)
        .unwrap()
        .hp;

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::TrapTriggered {
            trap: TrapKind::Pit,
            damage: 3,
            pos: Pos { x: 16, y: 5 },
            ..
        }
    )));
    assert_eq!(
        session
            .world
            .entities
            .actor_stats(session.world.player_id)
            .unwrap()
            .hp,
        before_hp - 3
    );
}

#[test]
fn waiting_on_trap_tile_does_not_retrigger() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 15, y: 5 });
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );

    let outcome = session.submit(CommandIntent::Wait);

    assert!(!outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::TrapTriggered { .. })));
}

#[test]
fn phase7_state_affects_snapshot_hash() {
    let mut a = GameSession::new_for_playing(42);
    let mut b = GameSession::new_for_playing(42);
    a.world.entities.clear_monsters();
    b.world.entities.clear_monsters();
    b.world.set_player_pos(Pos { x: 11, y: 5 });
    assert!(b.submit(CommandIntent::Search).accepted);

    assert_ne!(a.snapshot().stable_hash(), b.snapshot().stable_hash());
}

#[test]
fn scroll_reveal_reveals_all_hidden_tiles() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();

    let outcome = session.submit(CommandIntent::Read { item: EntityId(8) });

    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::ScrollRead {
            item: EntityId(8),
            ..
        }
    )));
    assert_eq!(
        session.world.current_map().tile(Pos { x: 12, y: 5 }),
        Ok(TileKind::Door(aihack::domain::tile::DoorState::Closed))
    );
    assert_eq!(
        session.world.current_map().tile(Pos { x: 16, y: 5 }),
        Ok(TileKind::Trap(TrapKind::Pit))
    );
    assert_eq!(
        session.world.entities.item_location(EntityId(8)),
        Some(EntityLocation::Consumed)
    );
}
