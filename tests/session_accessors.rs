use aihack::{
    core::{CommandIntent, GameSession, Pos, RunState},
    testing::SessionBuilder,
};

#[test]
fn session_exposes_read_only_runtime_state_through_accessors() {
    let mut session = GameSession::new_for_playing(42);

    assert_eq!(session.seed(), 42);
    assert_eq!(session.turn(), 0);
    assert_eq!(session.run_state(), RunState::Playing);
    assert!(session.event_log().is_empty());

    let outcome = session.submit(CommandIntent::Wait);

    assert!(outcome.accepted);
    assert_eq!(session.turn(), 1);
    assert_eq!(session.snapshot().turn, session.turn());
    assert_eq!(session.observation().turn, session.turn());
    assert_eq!(session.event_log().len(), outcome.events.len());
}

#[test]
fn fixture_builder_rebuilds_a_session_from_saved_world_configuration() {
    let session = SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.clear_monsters();
            assert!(world.entities.set_actor_location(
                world.player_id,
                world.current_level,
                Pos { x: 8, y: 5 },
            ));
        })
        .build();

    assert_eq!(session.world().player_pos(), Pos { x: 8, y: 5 });
    assert!(session.world().current_level_hostile_monsters().is_empty());
}
