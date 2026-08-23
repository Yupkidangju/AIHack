use aihack_ai_contract::{CommandIntent, Direction, InventoryAction, RunState};
use aihack_llm::narrative::{NarrativeResponse, NarrativeSource};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    compute_layout, runtime_event_to_candidate, runtime_key_to_candidate, TuiApp,
    UiCommandCandidate, UiRuntimeConfig, UiTheme, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::process::Command;

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

#[test]
fn one_event_one_frame_harness_does_not_duplicate_title_or_creation_input() {
    let mut app = TuiApp::new(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
    );
    let first =
        runtime_key_to_candidate(KeyCode::Enter, &app.run_state(), &app.observation()).unwrap();
    app.handle_candidate_owned(first).unwrap();
    assert_eq!(app.run_state(), RunState::CharacterCreation);
    assert_eq!(app.observation().turn, 0);

    let cancel =
        runtime_key_to_candidate(KeyCode::Esc, &app.run_state(), &app.observation()).unwrap();
    app.handle_candidate_owned(cancel).unwrap();
    assert_eq!(app.run_state(), RunState::Title);
    assert_eq!(app.observation().turn, 0);
}

#[test]
fn high_contrast_theme_is_consumed_by_the_actual_panel_renderer() {
    let area = Rect::new(0, 0, 24, 4);
    let mut buffer = Buffer::empty(area);
    aihack_tui::tui::render_panels::ThemedTextPanel {
        title: "STATUS",
        lines: vec!["LLM: TIMEOUT".to_string()],
        theme: UiTheme::high_contrast(),
    }
    .render(area, &mut buffer);

    assert_eq!(buffer[(0, 0)].fg, Color::Yellow);
    assert_eq!(buffer[(0, 1)].fg, Color::White);
    assert_eq!(buffer[(0, 1)].bg, Color::Black);
}

#[test]
fn runtime_keys_are_state_aware_for_load_cancel_more_and_inventory_letters() {
    let title = GameSession::try_new(42).unwrap();
    assert_eq!(
        runtime_key_to_candidate(KeyCode::Char('L'), &title.run_state(), &title.observation()),
        Some(UiCommandCandidate::Load)
    );

    let playing = GameSession::new_for_playing(42);
    let direction = RunState::AwaitingDirection {
        action: aihack_ai_contract::DirectionalAction::Open,
    };
    assert_eq!(
        runtime_key_to_candidate(KeyCode::Char('h'), &direction, &playing.observation()),
        Some(UiCommandCandidate::Command(CommandIntent::Move(
            Direction::West
        )))
    );
    assert_eq!(
        runtime_key_to_candidate(KeyCode::Esc, &direction, &playing.observation()),
        Some(UiCommandCandidate::Command(CommandIntent::AcknowledgeMore))
    );

    let inventory = RunState::AwaitingInventorySelection {
        action: InventoryAction::Wield,
    };
    assert_eq!(
        runtime_key_to_candidate(KeyCode::Char('a'), &inventory, &playing.observation()),
        Some(UiCommandCandidate::InventoryLetter('a'))
    );
    assert_eq!(
        runtime_key_to_candidate(
            KeyCode::Char('x'),
            &RunState::MorePrompt,
            &playing.observation()
        ),
        Some(UiCommandCandidate::Command(CommandIntent::AcknowledgeMore))
    );
    for key in [KeyCode::Tab, KeyCode::BackTab, KeyCode::Char('N')] {
        assert_eq!(
            runtime_key_to_candidate(key, &RunState::MorePrompt, &playing.observation()),
            Some(UiCommandCandidate::Command(CommandIntent::AcknowledgeMore)),
            "key={key:?}"
        );
    }
}

#[test]
fn production_event_loop_uses_the_single_state_aware_dispatcher() {
    let source = include_str!("../src/tui/mod.rs");
    assert!(source.contains("runtime_event_to_candidate"));
    assert!(!source.contains("KeyCode::Char('N') if app.has_llm_result()"));
}

#[test]
fn production_dispatcher_prioritizes_game_over_new_run_over_llm_dismiss() {
    let mut session = GameSession::new_for_playing(42);
    assert!(session.submit(CommandIntent::Quit).accepted);
    let mut app = TuiApp::new_with_llm_enabled(session, UiRuntimeConfig::default(), true);
    app.set_narrative_response(NarrativeResponse {
        text: "presentation result".to_string(),
        source: NarrativeSource::Provider,
        timed_out: false,
    });

    let candidate = runtime_event_to_candidate(
        Event::Key(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE)),
        80,
        24,
        &app,
    );
    assert_eq!(candidate, Some(UiCommandCandidate::NewRun));
}

#[test]
fn terminal_lifecycle_routes_setup_and_loop_failures_through_one_restore_boundary() {
    let source = include_str!("../src/tui/mod.rs");
    assert!(source.contains("setup_terminal_state"));
    assert!(source.contains("run_with_terminal_restore"));
}

#[test]
fn tui_help_uses_current_product_description() {
    let output = Command::new(env!("CARGO_BIN_EXE_aihack"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("NetHack 3.6.7 호환 Rust 로그라이크"));
    assert!(!stdout.contains("v0.1.0"));
    assert!(!stdout.contains("Phase"));
}
