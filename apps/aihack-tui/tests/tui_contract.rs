use aihack_runtime::GameSession;
use aihack_tui::tui::{
    compute_layout, runtime_key_to_candidate, TuiApp, UiCommandCandidate, UiRuntimeConfig,
    MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
};
use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[test]
fn tui_package_owns_the_runtime_app_and_layout() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());

    assert_eq!(app.observation().turn, 0);
    assert!(compute_layout(80, 28).validate().is_ok());
    assert!(app.run_single_frame(80, 28).is_ok());
}

#[test]
fn runtime_enter_key_reaches_title_and_character_creation_state_mapping() {
    let title = GameSession::try_new(42).unwrap();
    let candidate =
        runtime_key_to_candidate(KeyCode::Enter, &title.run_state(), &title.observation());
    assert!(matches!(candidate, Some(UiCommandCandidate::Command(_))));
}

#[test]
fn default_runtime_config_matches_the_60x24_terminal_contract() {
    let config = UiRuntimeConfig::default();
    assert_eq!(config.min_terminal_width, MIN_TERMINAL_WIDTH);
    assert_eq!(config.min_terminal_height, MIN_TERMINAL_HEIGHT);
}

#[test]
fn runtime_honors_an_explicit_terminal_minimum_config() {
    let config = UiRuntimeConfig {
        min_terminal_width: 100,
        min_terminal_height: 32,
        ..Default::default()
    };
    let app = TuiApp::new(GameSession::new_for_playing(42), config);

    assert!(!app.supports_terminal_size(80, 24));
    assert!(app.supports_terminal_size(100, 32));
}

#[test]
fn text_panel_clears_underlying_content_for_modal_blank_lines() {
    let area = Rect::new(0, 0, 24, 4);
    let mut buffer = Buffer::empty(area);
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buffer[(x, y)].set_char('x');
        }
    }

    aihack_tui::tui::render_panels::TextPanel {
        title: "MODAL",
        lines: vec![String::new()],
    }
    .render(area, &mut buffer);

    assert_eq!(buffer[(5, 1)].symbol(), " ");
}
