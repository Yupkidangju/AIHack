use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, Pos},
    domain::entity::EntityLocation,
};

#[test]
fn throw_item_hits_or_lands_deterministically() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();

    let outcome = session.submit(CommandIntent::Throw {
        item: EntityId(9),
        direction: Direction::East,
    });

    assert!(outcome.accepted);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::ItemThrown {
            item: EntityId(9),
            to: Pos { x: 9, y: 5 },
            ..
        }
    )));
    assert_eq!(
        session.world.entities.item_location(EntityId(9)),
        Some(EntityLocation::OnMap {
            level: session.world.current_level(),
            pos: Pos { x: 9, y: 5 },
        })
    );
}

#[test]
fn throw_non_throwable_is_rejected() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();

    let outcome = session.submit(CommandIntent::Throw {
        item: EntityId(6),
        direction: Direction::East,
    });

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
}

#[test]
fn throw_hits_monster_and_leaves_item_on_map() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(2), false);
    session.world.set_player_pos(Pos { x: 17, y: 12 });

    let outcome = session.submit(CommandIntent::Throw {
        item: EntityId(9),
        direction: Direction::East,
    });

    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved {
            defender: EntityId(3),
            ..
        }
    )));
    assert_eq!(
        session.world.entities.item_location(EntityId(9)),
        Some(EntityLocation::OnMap {
            level: session.world.current_level(),
            pos: Pos { x: 20, y: 12 },
        })
    );
}

#[test]
fn wand_zap_hits_first_target_and_spends_charge() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(2), false);
    session.world.set_player_pos(Pos { x: 17, y: 12 });

    let outcome = session.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });

    assert!(outcome.accepted);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::WandZapped {
            item: EntityId(7),
            charges_after: 2,
            ..
        }
    )));
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved {
            defender: EntityId(3),
            ..
        }
    )));
}

#[test]
fn empty_wand_is_rejected() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    assert!(session
        .world
        .entities
        .set_item_charges(EntityId(7), Some(0)));

    let outcome = session.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
}

#[test]
fn wand_zap_stops_at_wall() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();
    let eye = session.world.entities.spawn_monster(
        aihack::domain::monster::MonsterKind::FloatingEye,
        Pos { x: 11, y: 5 },
    );
    session.world.set_player_pos(Pos { x: 9, y: 5 });

    let outcome = session.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });

    assert!(outcome.accepted);
    assert!(!outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved { defender, .. } if *defender == eye
    )));
}
