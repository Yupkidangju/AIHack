use aihack::{
    core::{GameEvent, GameSession},
    ui::tui::{project_event, UiEffectKind, UiRuntimeConfig, UiTheme},
};

#[test]
fn ui_effect_projection_does_not_alter_core_hash() {
    let session = GameSession::new_for_playing(42);
    let before = session.snapshot().stable_hash();
    let effect = project_event(&GameEvent::Waited { turn: 1 }, 1).unwrap();
    assert_eq!(effect.kind, UiEffectKind::Info);
    let after = session.snapshot().stable_hash();
    assert_eq!(before, after);
}

#[test]
fn reduced_motion_shortens_effect_without_touching_hash() {
    let session = GameSession::new_for_playing(42);
    let before = session.snapshot().stable_hash();
    let standard = aihack::ui::tui::effects::project_event_with_config(
        &GameEvent::AttackResolved {
            attacker: session.world.player_id,
            defender: session.world.player_id,
            attack_roll: 12,
            defense: 10,
            hit: true,
            damage: 2,
        },
        1,
        &UiRuntimeConfig::default(),
    )
    .unwrap();
    let reduced = aihack::ui::tui::effects::project_event_with_config(
        &GameEvent::AttackResolved {
            attacker: session.world.player_id,
            defender: session.world.player_id,
            attack_roll: 12,
            defense: 10,
            hit: true,
            damage: 2,
        },
        1,
        &UiRuntimeConfig {
            reduced_motion: true,
            ..UiRuntimeConfig::default()
        },
    )
    .unwrap();
    assert!(reduced.ttl_ms < standard.ttl_ms);
    let after = session.snapshot().stable_hash();
    assert_eq!(before, after);
}

#[test]
fn high_contrast_theme_toggle_is_presentation_only() {
    let session = GameSession::new_for_playing(42);
    let before = session.snapshot().stable_hash();
    let standard = UiTheme::from_high_contrast(false);
    let contrast = UiTheme::from_high_contrast(true);
    assert_ne!(standard.accent, contrast.accent);
    let after = session.snapshot().stable_hash();
    assert_eq!(before, after);
}
