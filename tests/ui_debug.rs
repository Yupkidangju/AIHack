use aihack::core::session::GameSession;
use aihack::ui::tui::render_panels;

/// debug_observation_lines가 현재 observation 내용을 제공해야 한다.
#[test]
fn debug_observation_lines_are_not_empty() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let lines = render_panels::debug_observation_lines(&observation);
    assert!(!lines.is_empty());
}

/// debug_observation_lines에 필수 진단 항목이 포함되어야 한다.
#[test]
fn debug_observation_lines_include_required_fields() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let lines = render_panels::debug_observation_lines(&observation);
    let text = lines.join("\n");

    assert!(text.contains("schema_version"));
    assert!(text.contains("seed"));
    assert!(text.contains("turn"));
    assert!(text.contains("run_state"));
    assert!(text.contains("player_pos"));
    assert!(text.contains("player_hp"));
    assert!(text.contains("hunger"));
    assert!(text.contains("luck"));
    assert!(text.contains("visible_tiles"));
    assert!(text.contains("visible_entities"));
    assert!(text.contains("inventory"));
    assert!(text.contains("action_space"));
    assert!(text.contains("last_events"));
    assert!(text.contains("legal_actions"));
}
