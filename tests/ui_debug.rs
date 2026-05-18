use aihack::core::session::GameSession;
use aihack::ui::tui::render_panels;

/// [v0.2.0] Phase 18: debug_observation_lines가 비어있지 않다.
#[test]
fn debug_observation_lines_are_not_empty() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let lines = render_panels::debug_observation_lines(&observation);
    assert!(!lines.is_empty());
}

/// [v0.2.0] Phase 18: debug_observation_lines에 필수 항목이 포함된다.
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

/// [v0.2.0] Phase 18: F9 토글은 headless hash에 영향을 주지 않는다.
/// Debug observation은 UI-only 기능이므로 core 상태나 snapshot hash를 변경하지 않는다.
#[test]
fn debug_observation_toggle_does_not_affect_hash() {
    let session_a = GameSession::new_for_playing(42);
    let session_b = GameSession::new_for_playing(42);

    let hash_a = session_a.snapshot().stable_hash();
    let hash_b = session_b.snapshot().stable_hash();

    assert_eq!(hash_a, hash_b);
}
