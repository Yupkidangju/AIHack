use aihack_ai_contract::{CommandIntent, Direction, InventoryAction, RunState};
use aihack_llm::narrative::{NarrativeResponse, NarrativeSource};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    compute_layout, runtime_event_to_candidate, runtime_key_to_candidate, TuiApp,
    UiCommandCandidate, UiRuntimeConfig, UiTheme, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
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

fn command_wait_click(width: u16, height: u16, app: &TuiApp) -> Event {
    let layout = compute_layout(width, height);
    let line =
        aihack_tui::tui::render_panels::command_lines(&app.observation(), app.focused_panel())
            .remove(0);
    let offset = line.find("[. ] Wait").expect("Wait CTA must be rendered") as u16;
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: layout.command.x + offset,
        row: layout.command.y + 1,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn modal_and_overlay_mouse_clicks_never_submit_underlying_core_commands() {
    let mut inventory = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    inventory
        .handle_candidate_owned(UiCommandCandidate::Command(CommandIntent::ShowInventory))
        .unwrap();

    let mut storage = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    storage
        .handle_candidate_owned(UiCommandCandidate::Load)
        .unwrap();

    let mut soft = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    soft.handle_candidate_owned(UiCommandCandidate::LlmJudge)
        .unwrap();

    for (name, app) in [
        ("inventory", &mut inventory),
        ("storage", &mut storage),
        ("soft-input", &mut soft),
    ] {
        let before = app.revision();
        let candidate = runtime_event_to_candidate(command_wait_click(80, 24, app), 80, 24, app);
        if let Some(candidate) = candidate {
            app.handle_candidate_owned(candidate).unwrap();
        }
        assert_eq!(app.revision(), before, "modal={name}");
    }
}

#[test]
fn inspect_hover_and_decision_presentations_do_not_expose_hidden_inventory_commands() {
    let layout = compute_layout(80, 24);
    let click = || {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: layout.inspect.x,
            row: layout.inspect.y + 1,
            modifiers: KeyModifiers::NONE,
        })
    };

    let mut hover = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    hover
        .handle_candidate_owned(UiCommandCandidate::Inspect(aihack_ai_contract::Pos {
            x: 6,
            y: 5,
        }))
        .unwrap();
    assert!(
        !matches!(
            runtime_event_to_candidate(click(), 80, 24, &hover),
            Some(UiCommandCandidate::Command(_))
        ),
        "hover presentation must not expose an inventory command"
    );

    let mut decision = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let observation = decision.observation();
    let suggestion = aihack_llm::decision::fallback_suggestion(
        &aihack_llm::decision::DecisionRequest {
            revision: decision.revision(),
            action_space: observation.action_space.clone(),
            observation,
        },
        aihack_llm::decision::DecisionSource::Fallback,
    );
    decision.set_decision_suggestion(suggestion, Some(false));
    assert!(
        !matches!(
            runtime_event_to_candidate(click(), 80, 24, &decision),
            Some(UiCommandCandidate::Command(_))
        ),
        "decision presentation must not expose an inventory command"
    );
}

#[test]
fn llm_request_key_repeat_never_creates_a_new_candidate() {
    let app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    for key in ['G', 'A', 'J', 'R'] {
        let repeat =
            KeyEvent::new_with_kind(KeyCode::Char(key), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(
            runtime_event_to_candidate(Event::Key(repeat), 80, 24, &app),
            None,
            "key={key}"
        );
    }
}

#[test]
fn judge_editor_accepts_character_repeat_but_release_never_adds_text() {
    let mut app = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    app.handle_candidate_owned(UiCommandCandidate::LlmJudge)
        .unwrap();

    for character in ['G', 'A', 'J', 'R'] {
        for kind in [
            KeyEventKind::Press,
            KeyEventKind::Repeat,
            KeyEventKind::Release,
        ] {
            let event = Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char(character),
                KeyModifiers::NONE,
                kind,
            ));
            if let Some(candidate) = runtime_event_to_candidate(event, 80, 24, &app) {
                app.handle_candidate_owned(candidate).unwrap();
            }
        }
    }

    assert_eq!(app.soft_input(), Some("GGAAJJRR"));
}

#[test]
fn control_key_repeat_does_not_cross_state_boundaries() {
    let key = |code, kind| Event::Key(KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind));
    let dispatch =
        |app: &TuiApp, code, kind| runtime_event_to_candidate(key(code, kind), 80, 24, app);

    let mut judge = TuiApp::new_with_llm_enabled(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        true,
    );
    judge
        .handle_candidate_owned(UiCommandCandidate::LlmJudge)
        .unwrap();
    let cancel = dispatch(&judge, KeyCode::Esc, KeyEventKind::Press).unwrap();
    assert!(!judge.handle_candidate_owned(cancel).unwrap());
    assert_eq!(judge.run_state(), RunState::Playing);
    assert_eq!(dispatch(&judge, KeyCode::Esc, KeyEventKind::Repeat), None);
    assert_eq!(dispatch(&judge, KeyCode::Esc, KeyEventKind::Release), None);

    let mut inventory = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    inventory
        .handle_candidate_owned(UiCommandCandidate::Command(CommandIntent::ShowInventory))
        .unwrap();
    let close = dispatch(&inventory, KeyCode::Esc, KeyEventKind::Press).unwrap();
    assert!(!inventory.handle_candidate_owned(close).unwrap());
    assert_eq!(
        dispatch(&inventory, KeyCode::Esc, KeyEventKind::Repeat),
        None
    );

    let mut storage = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    storage
        .handle_candidate_owned(UiCommandCandidate::Load)
        .unwrap();
    let close = dispatch(&storage, KeyCode::Esc, KeyEventKind::Press).unwrap();
    assert!(!storage.handle_candidate_owned(close).unwrap());
    assert_eq!(dispatch(&storage, KeyCode::Esc, KeyEventKind::Repeat), None);

    let mut save = GameSession::new_for_playing(42).to_save_data();
    save.run_state = RunState::MorePrompt;
    let mut more = TuiApp::new(
        GameSession::from_save_data(save).unwrap(),
        UiRuntimeConfig::default(),
    );
    let acknowledge = dispatch(&more, KeyCode::Esc, KeyEventKind::Press).unwrap();
    assert!(!more.handle_candidate_owned(acknowledge).unwrap());
    assert_eq!(more.run_state(), RunState::Playing);
    assert_eq!(dispatch(&more, KeyCode::Esc, KeyEventKind::Repeat), None);

    let mut creation = TuiApp::new(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
    );
    let enter = dispatch(&creation, KeyCode::Enter, KeyEventKind::Press).unwrap();
    assert!(!creation.handle_candidate_owned(enter).unwrap());
    assert_eq!(creation.run_state(), RunState::CharacterCreation);
    assert_eq!(
        dispatch(&creation, KeyCode::Enter, KeyEventKind::Repeat),
        None
    );
    let back = dispatch(&creation, KeyCode::Esc, KeyEventKind::Press).unwrap();
    assert!(!creation.handle_candidate_owned(back).unwrap());
    assert_eq!(creation.run_state(), RunState::Title);
    assert_eq!(
        dispatch(&creation, KeyCode::Esc, KeyEventKind::Repeat),
        None
    );

    let undersized = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    for code in [KeyCode::Esc, KeyCode::Char('q'), KeyCode::Char('Q')] {
        assert_eq!(
            runtime_event_to_candidate(key(code, KeyEventKind::Repeat), 40, 10, &undersized),
            None
        );
    }
}

#[test]
fn f9_press_uses_the_actual_dispatch_and_handler_without_changing_core_revision() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before = app.revision();
    let f9 = |kind| {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::F(9),
            KeyModifiers::NONE,
            kind,
        ))
    };

    let candidate = runtime_event_to_candidate(f9(KeyEventKind::Press), 80, 24, &app);
    assert_eq!(candidate, Some(UiCommandCandidate::ToggleDebug));
    assert!(!app.handle_candidate_owned(candidate.unwrap()).unwrap());
    assert!(app.debug_observation_visible);
    assert_eq!(app.revision(), before);
    assert_eq!(
        runtime_event_to_candidate(f9(KeyEventKind::Repeat), 80, 24, &app),
        None
    );
    assert_eq!(
        runtime_event_to_candidate(f9(KeyEventKind::Release), 80, 24, &app),
        None
    );

    let candidate = runtime_event_to_candidate(f9(KeyEventKind::Press), 80, 24, &app).unwrap();
    assert!(!app.handle_candidate_owned(candidate).unwrap());
    assert!(!app.debug_observation_visible);
    assert_eq!(app.revision(), before);
}

#[test]
fn visible_debug_panel_consumes_mouse_while_hidden_panel_preserves_map_hit_testing() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let layout = compute_layout(80, 24);
    let event = || {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: layout.map.x + layout.map.width / 2 + 1,
            row: layout.map.y + layout.map.height / 2,
            modifiers: KeyModifiers::NONE,
        })
    };

    assert!(matches!(
        runtime_event_to_candidate(event(), 80, 24, &app),
        Some(UiCommandCandidate::Command(CommandIntent::Move(
            Direction::East
        )))
    ));
    let revision = app.revision();
    app.debug_observation_visible = true;
    assert_eq!(runtime_event_to_candidate(event(), 80, 24, &app), None);
    assert_eq!(app.revision(), revision);
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
