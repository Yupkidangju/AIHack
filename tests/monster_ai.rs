use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, LevelId, Pos, RunState},
    domain::monster::{monster_ai_kind, MonsterAiKind, MonsterKind},
};

fn kill_fixture_monsters(session: &mut GameSession) {
    session.world.entities.set_alive(EntityId(2), false);
    session.world.entities.set_alive(EntityId(3), false);
}

#[test]
fn monster_kind_maps_to_explicit_ai_policy() {
    assert_eq!(monster_ai_kind(MonsterKind::Jackal), MonsterAiKind::Wander);
    assert_eq!(
        monster_ai_kind(MonsterKind::Goblin),
        MonsterAiKind::ChaseVisiblePlayer
    );
    assert_eq!(
        monster_ai_kind(MonsterKind::FloatingEye),
        MonsterAiKind::Stationary
    );
}

#[test]
fn current_level_hostile_monsters_are_filtered_and_sorted() {
    let mut session = GameSession::new_for_playing(42);
    let level2_monster = session
        .world
        .entities
        .spawn_monster(MonsterKind::FloatingEye, Pos { x: 7, y: 7 });
    assert!(session.world.entities.set_actor_location(
        level2_monster,
        LevelId::main(2),
        Pos { x: 7, y: 7 }
    ));

    assert_eq!(
        session.world.current_level_hostile_monsters(),
        vec![EntityId(2), EntityId(3)]
    );

    session
        .world
        .set_player_location(LevelId::main(2), Pos { x: 5, y: 5 });

    assert_eq!(
        session.world.current_level_hostile_monsters(),
        vec![level2_monster]
    );
}

#[test]
fn monster_turn_runs_only_after_turn_advancing_commands() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(2), false);
    session.world.set_player_pos(Pos { x: 17, y: 12 });

    let show_inventory = session.submit(CommandIntent::ShowInventory);
    assert!(show_inventory.accepted);
    assert!(!show_inventory.turn_advanced);
    assert_eq!(
        session.world.entities.actor_location(EntityId(3)),
        Some((LevelId::main(1), Pos { x: 20, y: 12 }))
    );

    let rejected = session.submit(CommandIntent::Descend);
    assert!(!rejected.accepted);
    assert_eq!(
        session.world.entities.actor_location(EntityId(3)),
        Some((LevelId::main(1), Pos { x: 20, y: 12 }))
    );

    let waited = session.submit(CommandIntent::Wait);
    assert!(waited.accepted);
    assert!(waited.turn_advanced);
    assert!(waited.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity: EntityId(3),
            from: Pos { x: 20, y: 12 },
            to: Pos { x: 19, y: 12 },
        }
    )));
}

#[test]
fn monster_turn_order_is_entity_id_sorted() {
    let mut session = GameSession::new_for_playing(42);
    session.world.set_player_pos(Pos { x: 9, y: 5 });
    assert!(session.world.entities.set_actor_location(
        EntityId(2),
        LevelId::main(1),
        Pos { x: 8, y: 5 }
    ));
    assert!(session.world.entities.set_actor_location(
        EntityId(3),
        LevelId::main(1),
        Pos { x: 10, y: 5 }
    ));

    let outcome = session.submit(CommandIntent::Wait);
    let attackers = outcome
        .events
        .iter()
        .filter_map(|event| match event {
            GameEvent::AttackResolved { attacker, .. } if *attacker != session.world.player_id => {
                Some(*attacker)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(attackers, vec![EntityId(2), EntityId(3)]);
}

#[test]
fn goblin_chases_visible_player() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(2), false);
    session.world.set_player_pos(Pos { x: 17, y: 12 });
    let before = session
        .world
        .entities
        .actor_location(EntityId(3))
        .unwrap()
        .1;

    let outcome = session.submit(CommandIntent::Wait);
    let after = session
        .world
        .entities
        .actor_location(EntityId(3))
        .unwrap()
        .1;

    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity: EntityId(3),
            from: Pos { x: 20, y: 12 },
            to: Pos { x: 19, y: 12 },
        }
    )));
    assert!(
        after.chebyshev_distance(session.world.player_pos())
            < before.chebyshev_distance(session.world.player_pos())
    );
}

#[test]
fn goblin_waits_when_player_not_visible() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(2), false);

    let outcome = session.submit(CommandIntent::Wait);

    assert!(!outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity: EntityId(3),
            ..
        }
    )));
    assert_eq!(
        session.world.entities.actor_location(EntityId(3)),
        Some((LevelId::main(1), Pos { x: 20, y: 12 }))
    );
}

#[test]
fn jackal_wander_is_seed_stable() {
    fn run() -> Pos {
        let mut session = GameSession::new_for_playing(42);
        session.world.entities.set_alive(EntityId(3), false);
        session.world.set_player_pos(Pos { x: 1, y: 1 });
        assert!(session.world.entities.set_actor_location(
            EntityId(2),
            LevelId::main(1),
            Pos { x: 15, y: 10 }
        ));
        let before = session
            .world
            .entities
            .actor_location(EntityId(2))
            .unwrap()
            .1;

        let outcome = session.submit(CommandIntent::Wait);
        assert!(outcome.events.iter().any(|event| matches!(
            event,
            GameEvent::EntityMoved {
                entity: EntityId(2),
                from,
                ..
            } if *from == before
        )));
        session
            .world
            .entities
            .actor_location(EntityId(2))
            .unwrap()
            .1
    }

    assert_eq!(run(), run());
}

#[test]
fn floating_eye_remains_stationary() {
    let mut session = GameSession::new_for_playing(42);
    kill_fixture_monsters(&mut session);
    let eye = session
        .world
        .entities
        .spawn_monster(MonsterKind::FloatingEye, Pos { x: 8, y: 8 });
    let before = session.world.entities.actor_location(eye);

    let outcome = session.submit(CommandIntent::Wait);

    assert_eq!(session.world.entities.actor_location(eye), before);
    assert!(!outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved { entity, .. } if *entity == eye
    )));
}

#[test]
fn floating_eye_does_not_attack_when_adjacent() {
    let mut session = GameSession::new_for_playing(42);
    kill_fixture_monsters(&mut session);
    session.world.set_player_pos(Pos { x: 8, y: 8 });
    let eye = session
        .world
        .entities
        .spawn_monster(MonsterKind::FloatingEye, Pos { x: 9, y: 8 });

    let outcome = session.submit(CommandIntent::Wait);

    assert_eq!(
        session.world.entities.actor_location(eye).unwrap().1,
        Pos { x: 9, y: 8 }
    );
    assert!(!outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved { attacker, .. } if *attacker == eye
    )));
}

#[test]
fn adjacent_monster_attacks_instead_of_moving() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.set_alive(EntityId(3), false);
    session.world.set_player_pos(Pos { x: 9, y: 5 });
    assert!(session.world.entities.set_actor_location(
        EntityId(2),
        LevelId::main(1),
        Pos { x: 8, y: 5 }
    ));

    let outcome = session.submit(CommandIntent::Wait);

    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved {
            attacker: EntityId(2),
            defender,
            ..
        } if *defender == session.world.player_id
    )));
    assert!(!outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity: EntityId(2),
            ..
        }
    )));
}

#[test]
fn player_death_stops_remaining_monster_actions() {
    let mut session = GameSession::new_for_playing(42);
    session.world.set_player_pos(Pos { x: 9, y: 5 });
    session
        .world
        .entities
        .actor_stats_mut(session.world.player_id)
        .unwrap()
        .hp = 1;
    session
        .world
        .entities
        .actor_stats_mut(session.world.player_id)
        .unwrap()
        .ac = -20;
    assert!(session.world.entities.set_actor_location(
        EntityId(2),
        LevelId::main(1),
        Pos { x: 8, y: 5 }
    ));
    assert!(session.world.entities.set_actor_location(
        EntityId(3),
        LevelId::main(1),
        Pos { x: 10, y: 5 }
    ));

    let outcome = session.submit(CommandIntent::Wait);
    let attackers = outcome
        .events
        .iter()
        .filter_map(|event| match event {
            GameEvent::AttackResolved { attacker, .. } if *attacker != session.world.player_id => {
                Some(*attacker)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(matches!(session.state, RunState::GameOver { .. }));
    assert_eq!(attackers, vec![EntityId(2)]);
}

#[test]
fn off_level_monsters_do_not_act() {
    let mut session = GameSession::new_for_playing(42);
    session
        .world
        .set_player_location(LevelId::main(2), Pos { x: 5, y: 5 });
    let jackal_before = session.world.entities.actor_location(EntityId(2));
    let goblin_before = session.world.entities.actor_location(EntityId(3));

    let outcome = session.submit(CommandIntent::Wait);

    assert_eq!(
        session.world.entities.actor_location(EntityId(2)),
        jackal_before
    );
    assert_eq!(
        session.world.entities.actor_location(EntityId(3)),
        goblin_before
    );
    assert_eq!(
        outcome.events,
        vec![
            GameEvent::TurnStarted { turn: 1 },
            GameEvent::Waited { turn: 1 }
        ]
    );
}

#[test]
fn movement_events_include_actor_identity() {
    let mut player_session = GameSession::new_for_playing(42);
    player_session.world.entities.clear_monsters();
    let player_outcome = player_session.submit(CommandIntent::Move(Direction::East));
    assert!(player_outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity,
            from: Pos { x: 5, y: 5 },
            to: Pos { x: 6, y: 5 },
        } if *entity == player_session.world.player_id
    )));

    let mut monster_session = GameSession::new_for_playing(42);
    monster_session.world.entities.set_alive(EntityId(2), false);
    monster_session.world.set_player_pos(Pos { x: 17, y: 12 });
    let monster_outcome = monster_session.submit(CommandIntent::Wait);
    assert!(monster_outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::EntityMoved {
            entity: EntityId(3),
            from: Pos { x: 20, y: 12 },
            to: Pos { x: 19, y: 12 },
        }
    )));
}
