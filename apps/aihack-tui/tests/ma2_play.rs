use aihack_runtime::{GameClient, GameSession};
use aihack_tui::tui::{compute_layout, render_map::MapWidget, render_panels, UiTheme, Viewport};
use aihack_tui::tui::{
    render_frame, runtime_event_to_candidate, TuiApp, UiClock, UiCommandCandidate as U,
    UiRuntimeConfig,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Default)]
struct Clock(AtomicU64);
impl UiClock for Clock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}
fn press(app: &mut TuiApp, clock: &Clock, key: KeyCode) -> U {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    runtime_event_to_candidate(
        Event::Key(KeyEvent::new(key, KeyModifiers::NONE)),
        80,
        24,
        app,
    )
    .unwrap()
}
fn click_text(app: &mut TuiApp, text: &str) -> U {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| render_frame(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    for y in 0..24 {
        let line: String = (0..80).map(|x| buffer[(x, y)].symbol()).collect();
        if let Some(x) = line.find(text) {
            return runtime_event_to_candidate(
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: line[..x].chars().count() as u16,
                    row: y,
                    modifiers: KeyModifiers::NONE,
                }),
                80,
                24,
                app,
            )
            .unwrap();
        }
    }
    panic!("rendered CTA missing: {text}");
}

#[test]
fn all_projectile_directions_use_the_selected_item_and_match_direct_submit() {
    use aihack_ai_contract::{CommandIntent as C, Direction};
    for (key, direction) in [
        ('h', Direction::West),
        ('j', Direction::South),
        ('k', Direction::North),
        ('l', Direction::East),
        ('y', Direction::NorthWest),
        ('u', Direction::NorthEast),
        ('b', Direction::SouthWest),
        ('n', Direction::SouthEast),
    ] {
        for (action, letter) in [('t', 'e'), ('z', 'c')] {
            let session = GameSession::new_for_playing(42);
            let mut expected = session.clone();
            let item = session
                .observation()
                .inventory
                .iter()
                .find(|i| i.letter.0 == letter)
                .unwrap()
                .item;
            let clock = Arc::new(Clock::default());
            let mut app = TuiApp::new_with_llm_enabled_and_clock(
                session,
                UiRuntimeConfig::default(),
                false,
                clock.clone(),
            );
            let before = app.revision();
            for key in [action, letter] {
                let candidate = press(&mut app, &clock, KeyCode::Char(key));
                assert!(!app.handle_candidate_owned(candidate).unwrap());
                assert_eq!(app.revision(), before);
            }
            let candidate = press(&mut app, &clock, KeyCode::Char(key));
            let intent = if action == 't' {
                C::Throw { item, direction }
            } else {
                C::Zap { item, direction }
            };
            assert_eq!(candidate, U::Command(intent));
            app.handle_candidate_owned(candidate).unwrap();
            expected.submit(intent);
            assert_eq!(app.revision(), expected.revision());
        }
    }
}

#[test]
fn mouse_starts_game_and_selects_last_inventory_item_then_drop() {
    let mut app = TuiApp::new(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
    );
    for text in [
        "Press Enter to Start",
        "Press Enter to confirm",
        "[i] Inventory",
        "[e] rock",
        "[d] Drop",
    ] {
        let candidate = click_text(&mut app, text);
        assert!(!app.handle_candidate_owned(candidate).unwrap());
    }
    assert_eq!(app.observation().turn, 1);
    assert!(!app
        .observation()
        .inventory
        .iter()
        .any(|i| i.letter.0 == 'e'));
}

#[test]
fn cancelling_item_or_direction_selection_preserves_revision() {
    let clock = Arc::new(Clock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        GameSession::new_for_playing(42),
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    let before = app.revision();
    for key in [
        KeyCode::Char('t'),
        KeyCode::Esc,
        KeyCode::Char('t'),
        KeyCode::Char('e'),
        KeyCode::Esc,
    ] {
        let c = press(&mut app, &clock, key);
        app.handle_candidate_owned(c).unwrap();
        assert_eq!(app.revision(), before);
    }
}

#[test]
fn durable_save_survives_two_app_lifetimes_and_rng_continuation() {
    use aihack_ai_contract::CommandIntent as C;
    let directory = tempfile::tempdir().unwrap();
    let mut expected = GameSession::new_for_playing(42);
    expected.submit(C::Wait);
    {
        let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default())
            .with_save_directory(directory.path())
            .unwrap();
        app.handle_candidate_owned(U::Command(C::Wait)).unwrap();
        app.quick_save().unwrap();
    }
    let mut restored = TuiApp::new(GameSession::try_new(7).unwrap(), UiRuntimeConfig::default())
        .with_save_directory(directory.path())
        .unwrap();
    restored.quick_load().unwrap();
    assert_eq!(restored.revision(), expected.revision());
    restored
        .handle_candidate_owned(U::Command(C::Wait))
        .unwrap();
    expected.submit(C::Wait);
    assert_eq!(restored.revision(), expected.revision());
}

#[test]
fn quaff_shortcut_never_exits_and_quit_requires_confirmation() {
    use aihack_tui::tui::{key_to_candidate, TuiApp, UiCommandCandidate, UiRuntimeConfig};
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let before = app.revision();
    let quaff = key_to_candidate('q', &app.observation()).unwrap();
    assert!(!app.handle_candidate_owned(quaff).unwrap());
    assert_eq!(before, app.revision());
    app.handle_candidate_owned(UiCommandCandidate::CloseOverlay)
        .unwrap();
    assert!(!app
        .handle_candidate_owned(UiCommandCandidate::Quit)
        .unwrap());
    assert_eq!(before, app.revision());
}

#[test]
fn visible_starting_monster_and_floor_item_reach_map_and_hover() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let area = Rect::new(0, 0, 40, 20);
    let viewport = Viewport::from_rect(
        aihack_ai_contract::Pos { x: 0, y: 0 },
        observation.player_pos,
        area,
    );
    let mut buffer = Buffer::empty(area);
    MapWidget {
        observation: &observation,
        viewport,
        labels: &[],
        theme: UiTheme::standard(),
    }
    .render(area, &mut buffer);
    assert_eq!(buffer[(6, 5)].symbol(), "d");
    assert_eq!(buffer[(8, 5)].symbol(), "!");
    let lines = render_panels::inspect_lines(
        &observation,
        Some(aihack_ai_contract::Pos { x: 8, y: 5 }),
        aihack_tui::tui::UiPanel::Inspect,
        &[],
    );
    assert!(lines.iter().any(|line| line.contains("potion")));
}

#[test]
fn undersized_terminal_can_exit_without_an_invisible_confirmation() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());
    let candidate = runtime_event_to_candidate(
        Event::Key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE)),
        59,
        23,
        &mut app,
    )
    .unwrap();
    assert!(app.handle_candidate_owned(candidate).unwrap());
}

#[test]
fn full_inventory_contains_all_starting_items() {
    let observation = GameSession::new_for_playing(42).observation();
    let lines = render_panels::inventory_overlay_lines(&observation);
    for item in &observation.inventory {
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with(&format!("{} ", item.letter.0))),
            "missing {}",
            item.letter.0
        );
    }
}

#[test]
fn minimum_log_shows_message_body_and_status_shows_hunger() {
    for (w, h) in [(60, 24), (80, 24), (120, 36)] {
        let layout = compute_layout(w, h);
        let mut buffer = Buffer::empty(layout.root);
        render_panels::TextPanel {
            title: "LOG",
            lines: vec!["! hit for 3".into()],
        }
        .render(layout.log, &mut buffer);
        let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
        assert!(text.contains("! hit for 3"));
    }
    assert!(
        render_panels::status_lines(&GameSession::new_for_playing(42).observation())
            .iter()
            .any(|line| line.contains("food"))
    );
}
