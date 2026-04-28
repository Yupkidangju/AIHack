use aihack::{
    core::{CommandIntent, EntityId, GameEvent, GameSession, LevelId, Pos},
    domain::level::{
        PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID, PHASE5_LEVEL2_STAIRS_UP_POS,
    },
};

#[test]
fn descend_requires_stairs_down() {
    let mut session = GameSession::new(42);
    let before_hash = session.snapshot().stable_hash();

    let outcome = session.submit(CommandIntent::Descend);

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.turn, 0);
    assert_eq!(session.world.current_level(), PHASE5_LEVEL1_ID);
    assert_eq!(outcome.snapshot_hash, before_hash);
    assert!(
        matches!(&outcome.events[..], [GameEvent::CommandRejected { reason }] if reason.contains("stairs down"))
    );
}

#[test]
fn descend_lands_on_level2_stairs_up() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);

    let outcome = session.submit(CommandIntent::Descend);

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(session.world.current_level(), PHASE5_LEVEL2_ID);
    assert_eq!(session.world.player_pos(), PHASE5_LEVEL2_STAIRS_UP_POS);
    assert_eq!(
        session
            .world
            .entities
            .actor_location(session.world.player_id),
        Some((PHASE5_LEVEL2_ID, PHASE5_LEVEL2_STAIRS_UP_POS))
    );
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
}

#[test]
fn ascend_requires_stairs_up() {
    let mut session = GameSession::new(42);
    session
        .world
        .set_player_location(PHASE5_LEVEL2_ID, Pos { x: 6, y: 5 });
    let before_hash = session.snapshot().stable_hash();

    let outcome = session.submit(CommandIntent::Ascend);

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.world.current_level(), PHASE5_LEVEL2_ID);
    assert_eq!(outcome.snapshot_hash, before_hash);
    assert!(
        matches!(&outcome.events[..], [GameEvent::CommandRejected { reason }] if reason.contains("stairs up"))
    );
}

#[test]
fn ascend_returns_to_level1_stairs_down() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
    assert!(session.submit(CommandIntent::Descend).accepted);

    let outcome = session.submit(CommandIntent::Ascend);

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(session.world.current_level(), PHASE5_LEVEL1_ID);
    assert_eq!(session.world.player_pos(), PHASE5_LEVEL1_STAIRS_DOWN);
    assert_eq!(
        session
            .world
            .entities
            .actor_location(session.world.player_id),
        Some((PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN))
    );
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
}

#[test]
fn level_change_event_order_is_stable() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);

    let outcome = session.submit(CommandIntent::Descend);

    assert!(matches!(
        outcome.events.first(),
        Some(GameEvent::TurnStarted { turn: 1 })
    ));
    assert!(matches!(
        outcome.events.get(1),
        Some(GameEvent::LevelChanged { entity: EntityId(1), from, to })
            if *from == PHASE5_LEVEL1_ID && *to == PHASE5_LEVEL2_ID
    ));
    assert_eq!(outcome.events.len(), 2);
}

#[test]
fn landing_positions_are_stable() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
    assert!(session.submit(CommandIntent::Descend).accepted);
    assert_eq!(session.world.player_pos(), PHASE5_LEVEL2_STAIRS_UP_POS);
    assert!(session.submit(CommandIntent::Ascend).accepted);
    assert_eq!(session.world.player_pos(), PHASE5_LEVEL1_STAIRS_DOWN);
}

#[test]
fn observation_includes_stairs_actions() {
    let mut session = GameSession::new(42);
    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
    let level1_stairs = session.observation();
    assert_eq!(level1_stairs.current_level, PHASE5_LEVEL1_ID);
    assert!(level1_stairs
        .legal_actions
        .contains(&CommandIntent::Descend));

    session.world.set_player_pos(Pos { x: 5, y: 5 });
    let level1_floor = session.observation();
    assert!(!level1_floor.legal_actions.contains(&CommandIntent::Descend));

    session
        .world
        .set_player_location(PHASE5_LEVEL2_ID, PHASE5_LEVEL2_STAIRS_UP_POS);
    let level2_stairs = session.observation();
    assert_eq!(level2_stairs.current_level, PHASE5_LEVEL2_ID);
    assert!(level2_stairs.legal_actions.contains(&CommandIntent::Ascend));
    assert!(level2_stairs
        .visible_tiles
        .iter()
        .all(|tile| { session.world.map(LevelId::main(2)).tile(tile.pos).is_ok() }));
}
