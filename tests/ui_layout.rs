use aihack::{
    core::GameSession,
    ui::tui::{compute_layout, render_panels, terminal_size_supported, LayoutTier, UiPanel},
};

#[test]
fn layout_80x28_has_no_overlap() {
    let layout = compute_layout(80, 28);
    assert_eq!(layout.tier, LayoutTier::Standard);
    layout.validate().unwrap();
    assert_eq!(layout.map.width, 52);
    assert_eq!(layout.status.width, 28);
    assert!(layout.map.height >= 20);
}

#[test]
fn manual_matrix_sizes_follow_the_60x24_runtime_contract() {
    for (width, height) in [(120, 36), (80, 24), (60, 24)] {
        assert!(terminal_size_supported(width, height));
        compute_layout(width, height).validate().unwrap();
    }

    for (width, height) in [(59, 24), (60, 23), (59, 23)] {
        assert!(!terminal_size_supported(width, height));
    }
}

#[test]
fn larger_layout_tiers_preserve_panel_contract() {
    let standard = compute_layout(100, 32);
    let roomy = compute_layout(120, 36);
    standard.validate().unwrap();
    roomy.validate().unwrap();
    assert_eq!(standard.tier, LayoutTier::Standard);
    assert_eq!(roomy.tier, LayoutTier::Roomy);
    assert_eq!(standard.map.width, 65);
    assert_eq!(roomy.map.width, 84);
    assert!(roomy.debug.is_some());
}

#[test]
fn compact_60x24_keeps_every_panel_in_bounds_without_overlap() {
    let compact = compute_layout(60, 24);
    assert_eq!(compact.tier, LayoutTier::Degraded);
    compact.validate().unwrap();
    assert_eq!((compact.map.x, compact.map.y), (0, 0));
    assert_eq!((compact.map.width, compact.map.height), (40, 20));
    assert_eq!(compact.command.height, 3);
    assert_eq!(compact.log.height, 1);
}

#[test]
fn priority_message_and_command_hint_render_have_accessible_text() {
    let mut session = GameSession::new_for_playing(42);
    let player = session.world().player_id();
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.actor_stats_mut(player).unwrap().hp = 3;
    });
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
