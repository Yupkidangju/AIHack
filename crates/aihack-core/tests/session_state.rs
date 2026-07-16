use aihack_core::{
    event::GameEvent, meta::GameMeta, rng::GameRng, run_state::RunState, session::SessionState,
};

#[test]
fn session_state_keeps_runtime_fields_with_an_adapter_owned_world() {
    let state = SessionState {
        meta: GameMeta { seed: 42 },
        rng: GameRng::new(42),
        turn: 7,
        state: RunState::Playing,
        world: "adapter-world",
        event_log: vec![GameEvent::Waited { turn: 7 }],
    };

    assert_eq!(state.meta.seed, 42);
    assert_eq!(state.turn, 7);
    assert_eq!(state.state, RunState::Playing);
    assert_eq!(state.world, "adapter-world");
    assert_eq!(state.event_log.len(), 1);
}
