use aihack::{
    core::GameSession,
    ui::tui::{compute_layout, render_panels, LayoutTier, UiPanel},
};

#[test]
fn layout_80x28_has_no_overlap() {
    let layout = compute_layout(80, 28);
    assert_eq!(layout.tier, LayoutTier::Degraded);
    layout.validate().unwrap();
    assert_eq!(layout.map.width, 40);
    assert_eq!(layout.map.height, 20);
}

#[test]
fn larger_layout_tiers_preserve_panel_contract() {
    let standard = compute_layout(100, 32);
    let roomy = compute_layout(120, 36);
    standard.validate().unwrap();
    roomy.validate().unwrap();
    assert_eq!(standard.tier, LayoutTier::Standard);
    assert_eq!(roomy.tier, LayoutTier::Roomy);
    assert!(roomy.debug.is_some());
}

#[test]
fn priority_message_and_command_hint_render_have_accessible_text() {
    let mut session = GameSession::new_for_playing(42);
    let player = session.world.player_id;
    let stats = session.world.entities.actor_stats_mut(player).unwrap();
    stats.hp = 3;
    let observation = session.observation();
    let log_lines = render_panels::log_lines(&observation, &["Narrative(idle)".to_string()]);
    let command_lines = render_panels::command_lines(&observation, UiPanel::Inspect);
    let status_lines = render_panels::status_lines(&observation);
    assert!(log_lines.iter().any(|line| line.contains("hp critical")));
    assert!(command_lines
        .iter()
        .any(|line| line.contains("[hover] Inspect")));
    assert!(status_lines.iter().any(|line| line.contains("ALERT")));
}
