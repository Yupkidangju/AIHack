use std::{env, fs};

use aihack::{
    core::GameSession,
    ui::tui::{compute_layout, runtime_smoke, TuiApp, UiCommandCandidate, UiRuntimeConfig},
};

#[test]
fn ui_runtime_smoke() {
    let map = runtime_smoke().unwrap();
    assert!(map.width >= 40);
    assert!(map.height >= 20);
}

#[test]
fn tui_app_save_load_bridge_uses_core_api() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let mut path = env::temp_dir();
    path.push(format!("aihack-tui-save-{}.json", std::process::id()));
    app.save_to_path(&path).unwrap();
    app.load_from_path(&path).unwrap();
    fs::remove_file(path).unwrap();
}

#[test]
fn viewport_roundtrip_matches_render_hit_contract() {
    let app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let layout = compute_layout(100, 32);
    let viewport = app.viewport_for_observation(layout);
    let world = aihack::core::Pos {
        x: app.session.world.player_pos().x + 1,
        y: app.session.world.player_pos().y,
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
    let mut path = env::temp_dir();
    path.push(format!("aihack-tui-handle-{}.json", std::process::id()));
    app.handle_candidate(UiCommandCandidate::Save, &path, &path)
        .unwrap();
    app.handle_candidate(UiCommandCandidate::Load, &path, &path)
        .unwrap();
    fs::remove_file(path).unwrap();
}

#[test]
fn narrative_consumer_smoke() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let response = aihack::llm::narrative::fallback_response(
        &aihack::llm::narrative::NarrativeRequest {
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
    let suggestion = aihack::llm::decision::fallback_suggestion(
        &aihack::llm::decision::DecisionRequest {
            observation: GameSession::new_for_playing(42).observation(),
            action_space: GameSession::new_for_playing(42).observation().action_space,
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
    app.handle_candidate(
        UiCommandCandidate::Inspect(aihack::core::Pos { x: 6, y: 5 }),
        std::path::Path::new("/tmp/unused-save.json"),
        std::path::Path::new("/tmp/unused-load.json"),
    )
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
