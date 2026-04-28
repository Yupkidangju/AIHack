use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, Pos, RunState},
    domain::combat::DeathCause,
    systems::death,
};

#[test]
fn monster_death_creates_event_and_tombstone() {
    let mut session = GameSession::new(42);
    let jackal = EntityId(2);

    for _ in 0..50 {
        let outcome = session.submit(CommandIntent::Move(Direction::East));
        assert!(outcome.accepted);
        if !session
            .world
            .entities
            .get(jackal)
            .unwrap()
            .actor()
            .unwrap()
            .5
        {
            assert!(outcome.events.iter().any(|event| matches!(
                event,
                GameEvent::EntityDied {
                    entity,
                    cause: DeathCause::Combat { attacker }
                } if *entity == jackal && *attacker == session.world.player_id
            )));
            return;
        }
    }
    panic!("deterministic combat should kill the adjacent jackal within 50 bump attacks");
}

#[test]
fn dead_monster_no_longer_blocks_movement() {
    let mut session = GameSession::new(42);
    let jackal = EntityId(2);
    session.world.entities.set_alive(jackal, false);

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert_eq!(session.world.player_pos(), Pos { x: 6, y: 5 });
}

#[test]
fn player_death_enters_game_over() {
    let mut session = GameSession::new(42);
    let player = session.world.player_id;
    let attacker = EntityId(3);
    session.world.entities.actor_stats_mut(player).unwrap().hp = 0;

    let events = death::collect_death_events_after_attack(&mut session.world, attacker, player);
    session.state = death::state_after_deaths(&session.world);

    assert_eq!(session.state, RunState::GameOver);
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::EntityDied {
            entity,
            cause: DeathCause::Combat { attacker: cause_attacker }
        } if *entity == player && *cause_attacker == attacker
    )));
}

#[test]
fn snapshot_hash_changes_when_entity_state_changes() {
    let mut session = GameSession::new(42);
    let before = session.snapshot().stable_hash();
    session
        .world
        .entities
        .actor_stats_mut(EntityId(2))
        .unwrap()
        .hp -= 1;
    let after = session.snapshot().stable_hash();

    assert_ne!(before, after);
}
