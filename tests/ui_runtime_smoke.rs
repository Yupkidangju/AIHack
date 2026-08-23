use aihack::{
    core::{CommandIntent, GameSession},
    ui::tui::{
        compute_layout, runtime_smoke, StorageOperation, TuiApp, UiCommandCandidate, UiOverlay,
        UiPanel, UiRuntimeConfig,
    },
};

#[test]
fn ui_runtime_smoke() {
    let map = runtime_smoke().unwrap();
    assert!(map.width >= 40);
    assert!(map.height >= 20);
}

#[test]
fn tui_app_owned_quick_save_load_bridge_uses_core_api() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    app.quick_save().unwrap();
    app.handle_candidate_owned(UiCommandCandidate::Command(CommandIntent::Wait))
        .unwrap();
    assert_eq!(app.observation().turn, 1);
    app.quick_load().unwrap();
    assert_eq!(app.observation().turn, 0);
}

#[test]
fn viewport_roundtrip_matches_render_hit_contract() {
    let app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let layout = compute_layout(100, 32);
    let viewport = app.viewport_for_observation(layout);
    let observation = app.observation();
    let world = aihack::core::Pos {
        x: observation.player_pos.x + 1,
        y: observation.player_pos.y,
    };
    let term = viewport.world_to_terminal(world, layout.map).unwrap();
    let roundtrip = viewport
        .terminal_to_world(term.0, term.1, layout.map)
        .unwrap();
    assert_eq!(world, roundtrip);
}

#[test]
fn handle_candidate_bridges_save_and_load() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    app.handle_candidate_owned(UiCommandCandidate::Save)
        .unwrap();
    app.handle_candidate_owned(UiCommandCandidate::Load)
        .unwrap();
}

#[test]
fn narrative_consumer_smoke() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let response = aihack::llm::narrative::fallback_response(
        &aihack::llm::narrative::NarrativeRequest {
            revision: app.revision(),
            topic: aihack::core::NarrativeTopic::SituationSummary,
            observation: app.observation(),
        },
        false,
    );
    app.set_narrative_response(response);
    let lines = app.narrative_lines();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Narrative"));
}

#[test]
fn decision_support_consumer_smoke() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let observation = app.observation();
    let suggestion = aihack::llm::decision::fallback_suggestion(
        &aihack::llm::decision::DecisionRequest {
            revision: app.revision(),
            action_space: observation.action_space.clone(),
            observation,
        },
        aihack::llm::decision::DecisionSource::Fallback,
    );
    app.set_decision_suggestion(suggestion, Some(false));
    let lines = app.decision_lines();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Decision"));
}

#[test]
fn inspect_panel_prefers_hovered_read_only_lines() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before_turn = app.observation().turn;
    app.handle_candidate_owned(UiCommandCandidate::Inspect(aihack::core::Pos {
        x: 6,
        y: 5,
    }))
    .unwrap();
    let observation = app.observation();
    let lines = aihack::ui::tui::render_panels::inspect_lines(
        &observation,
        app.hovered_pos(),
        app.focused_panel(),
        &app.decision_lines(),
    );
    assert_eq!(observation.turn, before_turn);
    assert!(lines.iter().any(|line| line.contains("read-only inspect")));
}

#[test]
fn inventory_overlay_and_recoverable_storage_error_do_not_mutate_core() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before = app.revision();

    app.handle_candidate_owned(UiCommandCandidate::Command(CommandIntent::ShowInventory))
        .unwrap();
    assert_eq!(app.ui_overlay(), &UiOverlay::Inventory);
    assert_eq!(app.revision(), before);
    app.handle_candidate_owned(UiCommandCandidate::CloseOverlay)
        .unwrap();
    assert_eq!(app.ui_overlay(), &UiOverlay::None);

    app.handle_candidate_owned(UiCommandCandidate::Load)
        .unwrap();
    assert_eq!(app.storage_error(), Some(StorageOperation::Load));
    assert_eq!(app.revision(), before);
}

#[test]
fn new_run_clears_every_transient_ui_state_but_preserves_config() {
    let config = UiRuntimeConfig {
        high_contrast: true,
        reduced_motion: true,
        ..Default::default()
    };
    let mut app = TuiApp::new(GameSession::new_for_playing(42), config);
    app.handle_candidate_owned(UiCommandCandidate::Inspect(aihack::core::Pos {
        x: 6,
        y: 5,
    }))
    .unwrap();
    app.handle_candidate_owned(UiCommandCandidate::Command(CommandIntent::ShowInventory))
        .unwrap();
    app.debug_observation_visible = true;

    app.handle_candidate_owned(UiCommandCandidate::NewRun)
        .unwrap();

    assert_eq!(app.ui_overlay(), &UiOverlay::None);
    assert_eq!(app.hovered_pos(), None);
    assert_eq!(app.focused_panel(), UiPanel::Map);
    assert!(!app.debug_observation_visible);
    assert!(app.config.high_contrast);
    assert!(app.config.reduced_motion);
}
