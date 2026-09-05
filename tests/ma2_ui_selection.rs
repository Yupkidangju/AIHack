use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use aihack::{
    core::{CommandIntent, Direction, GameSession},
    domain::{
        entity::EntityLocation,
        item::ItemKind,
        tile::{DoorState, TileKind},
    },
    testing::SessionBuilder,
    ui::tui::{
        render_frame, runtime_event_to_candidate, TuiApp, UiClock, UiCommandCandidate,
        UiRuntimeConfig,
    },
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{backend::TestBackend, Terminal};

#[derive(Default)]
struct TestClock(AtomicU64);
impl UiClock for TestClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}

fn app(session: GameSession) -> (TuiApp, Arc<TestClock>) {
    let clock = Arc::new(TestClock::default());
    (
        TuiApp::new_with_llm_enabled_and_clock(
            session,
            UiRuntimeConfig::default(),
            false,
            clock.clone(),
        ),
        clock,
    )
}

fn dispatch(app: &mut TuiApp, clock: &TestClock, event: Event) -> UiCommandCandidate {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    let candidate =
        runtime_event_to_candidate(event, 120, 36, app).expect("production input candidate");
    assert!(!app.handle_candidate_owned(candidate).unwrap());
    candidate
}

fn key(app: &mut TuiApp, clock: &TestClock, code: KeyCode) -> UiCommandCandidate {
    dispatch(
        app,
        clock,
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE)),
    )
}

fn rendered(app: &mut TuiApp) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
    terminal.draw(|frame| render_frame(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..36)
        .map(|y| (0..120).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn click_text(app: &mut TuiApp, clock: &TestClock, text: &str) -> UiCommandCandidate {
    let rows = rendered(app);
    let (row, column) = rows
        .iter()
        .enumerate()
        .find_map(|(y, line)| line.find(text).map(|x| (y, line[..x].chars().count())))
        .unwrap_or_else(|| panic!("visible text missing: {text}\n{}", rows.join("\n")));
    dispatch(
        app,
        clock,
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: column as u16,
            row: row as u16,
            modifiers: KeyModifiers::NONE,
        }),
    )
}

#[test]
fn door_selection_supports_all_eight_directions_and_cancellation() {
    let directions = [
        ('y', Direction::NorthWest),
        ('k', Direction::North),
        ('u', Direction::NorthEast),
        ('h', Direction::West),
        ('l', Direction::East),
        ('b', Direction::SouthWest),
        ('j', Direction::South),
        ('n', Direction::SouthEast),
    ];
    for action in ['o', 'c', 'K'] {
        for (direction_key, direction) in directions {
            let mut session = GameSession::new_for_playing(42);
            let pos = session.world().player_pos().offset(direction.delta());
            SessionBuilder::mutate(&mut session, |world| {
                world.saved().entities.clear_monsters();
                world
                    .current_map_mut()
                    .set_tile(
                        pos,
                        TileKind::Door(if action == 'c' {
                            DoorState::Open
                        } else {
                            DoorState::Closed
                        }),
                    )
                    .unwrap();
            });
            let (mut app, clock) = app(session);
            let before = app.revision();
            assert_eq!(
                key(&mut app, &clock, KeyCode::Char(action)),
                UiCommandCandidate::BeginAction(action)
            );
            assert!(rendered(&mut app)
                .join("\n")
                .contains(&format!("{direction:?}")));
            assert_eq!(app.revision(), before);
            assert_eq!(
                key(&mut app, &clock, KeyCode::Esc),
                UiCommandCandidate::CloseOverlay
            );
            assert_eq!(app.revision(), before);
            key(&mut app, &clock, KeyCode::Char(action));
            let command = match action {
                'o' => CommandIntent::Open(direction),
                'c' => CommandIntent::Close(direction),
                _ => CommandIntent::Kick(direction),
            };
            assert_eq!(
                key(&mut app, &clock, KeyCode::Char(direction_key)),
                UiCommandCandidate::Command(command)
            );
            assert_eq!(app.revision().turn, before.turn + 1);
            let expected = TileKind::Door(if action == 'c' {
                DoorState::Closed
            } else {
                DoorState::Open
            });
            assert_eq!(
                app.observation()
                    .visible_tiles
                    .iter()
                    .find(|tile| tile.pos == pos)
                    .unwrap()
                    .tile,
                expected
            );
        }
    }
}

fn twelve_item_session() -> GameSession {
    SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.clear_monsters();
            while world.inventory.entries.len() < 12 {
                let item = world
                    .entities
                    .spawn_item(
                        ItemKind::Rock,
                        EntityLocation::Inventory {
                            owner: world.player_id,
                        },
                    )
                    .unwrap();
                let letter = world.inventory.add_existing_with_next_letter(item).unwrap();
                assert!(world.entities.set_item_letter(item, letter));
            }
        })
        .build()
}

#[test]
fn inventory_next_button_and_last_item_mouse_drop_reach_concrete_command() {
    let session = twelve_item_session();
    let last = session.observation().inventory.last().unwrap().clone();
    let (mut app, clock) = app(session);
    let before = app.revision();
    key(&mut app, &clock, KeyCode::Char('i'));
    assert!(rendered(&mut app).join("\n").contains("Page 1/2"));
    assert_eq!(
        click_text(&mut app, &clock, "[>] Next"),
        UiCommandCandidate::MenuPage(true)
    );
    assert!(rendered(&mut app).join("\n").contains("Page 2/2"));
    assert_eq!(app.revision(), before);
    assert_eq!(
        click_text(&mut app, &clock, &format!("[{}]", last.letter.0)),
        UiCommandCandidate::ChooseItem {
            action: 'i',
            item: last.item
        }
    );
    assert_eq!(app.revision(), before);
    assert_eq!(
        click_text(&mut app, &clock, "[d] Drop"),
        UiCommandCandidate::Command(CommandIntent::Drop { item: last.item })
    );
    assert_eq!(app.revision().turn, before.turn + 1);
    assert!(!app
        .observation()
        .inventory
        .iter()
        .any(|item| item.item == last.item));
    assert!(app
        .observation()
        .visible_entities
        .iter()
        .any(|entity| entity.entity == last.item && entity.pos == app.observation().player_pos));
}

#[test]
fn inventory_paged_keyboard_selection_can_cancel_without_core_mutation() {
    let session = twelve_item_session();
    let last = session.observation().inventory.last().unwrap().clone();
    let (mut app, clock) = app(session);
    let before = app.revision();
    key(&mut app, &clock, KeyCode::Char('i'));
    key(&mut app, &clock, KeyCode::PageDown);
    assert!(rendered(&mut app)
        .join("\n")
        .contains(&format!("[{}]", last.letter.0)));
    assert_eq!(
        key(&mut app, &clock, KeyCode::Char(last.letter.0)),
        UiCommandCandidate::ChooseItem {
            action: 'i',
            item: last.item
        }
    );
    key(&mut app, &clock, KeyCode::Esc);
    assert_eq!(app.revision(), before);
    assert_eq!(app.observation().inventory.len(), 12);
}

#[test]
fn inventory_last_page_item_mouse_selection_and_drop_work_after_page_down() {
    let session = twelve_item_session();
    let last = session.observation().inventory.last().unwrap().clone();
    let (mut app, clock) = app(session);
    let before = app.revision();
    key(&mut app, &clock, KeyCode::Char('i'));
    key(&mut app, &clock, KeyCode::PageDown);
    assert_eq!(
        click_text(&mut app, &clock, &format!("[{}]", last.letter.0)),
        UiCommandCandidate::ChooseItem {
            action: 'i',
            item: last.item
        }
    );
    assert_eq!(app.revision(), before);
    assert_eq!(
        click_text(&mut app, &clock, "[d] Drop"),
        UiCommandCandidate::Command(CommandIntent::Drop { item: last.item })
    );
    assert_eq!(app.revision().turn, before.turn + 1);
    assert!(!app
        .observation()
        .inventory
        .iter()
        .any(|item| item.item == last.item));
}

#[test]
fn playing_q_then_potion_letter_quaffs_without_exiting_and_heals() {
    let session = SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.clear_monsters();
            world.entities.actor_stats_mut(world.player_id).unwrap().hp = 1;
            let potion = world
                .entities
                .spawn_item(
                    ItemKind::PotionHealing,
                    EntityLocation::Inventory {
                        owner: world.player_id,
                    },
                )
                .unwrap();
            let letter = world
                .inventory
                .add_existing_with_next_letter(potion)
                .unwrap();
            assert!(world.entities.set_item_letter(potion, letter));
        })
        .build();
    let potion = session
        .observation()
        .inventory
        .into_iter()
        .find(|item| item.kind == ItemKind::PotionHealing)
        .unwrap();
    let (mut app, clock) = app(session);
    let before = app.revision();
    let hp_before = app.observation().player.hp;
    assert_eq!(
        key(&mut app, &clock, KeyCode::Char('q')),
        UiCommandCandidate::BeginAction('q')
    );
    assert_eq!(app.revision(), before);
    assert!(rendered(&mut app)
        .join("\n")
        .contains(&format!("[{}]", potion.letter.0)));
    assert_eq!(
        key(&mut app, &clock, KeyCode::Char(potion.letter.0)),
        UiCommandCandidate::Command(CommandIntent::Quaff { item: potion.item })
    );
    assert_eq!(app.revision().turn, before.turn + 1);
    assert_eq!(app.run_state(), aihack::core::RunState::Playing);
    assert!(app.observation().player.hp > hp_before);
    assert!(!app
        .observation()
        .inventory
        .iter()
        .any(|item| item.item == potion.item));
}

#[test]
fn game_over_new_run_and_title_creation_actions_use_rendered_mouse_ctas() {
    let mut session = SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.actor_stats_mut(world.player_id).unwrap().hp = 1;
        })
        .build();
    for _ in 0..20 {
        if matches!(session.run_state(), aihack::core::RunState::GameOver { .. }) {
            break;
        }
        session.submit(CommandIntent::Wait);
    }
    assert!(matches!(
        session.run_state(),
        aihack::core::RunState::GameOver { .. }
    ));
    let (mut app, clock) = app(session);
    assert_eq!(
        click_text(&mut app, &clock, "[N] New Run"),
        UiCommandCandidate::NewRun
    );
    assert_eq!(app.run_state(), aihack::core::RunState::Title);
    assert_eq!(app.observation().seed, 43);
    assert_eq!(
        click_text(&mut app, &clock, "Press Enter to Start"),
        UiCommandCandidate::Command(CommandIntent::Wait)
    );
    assert_eq!(app.run_state(), aihack::core::RunState::CharacterCreation);
    assert_eq!(
        click_text(&mut app, &clock, "Esc - Back to Title"),
        UiCommandCandidate::BackToTitle
    );
    assert_eq!(app.run_state(), aihack::core::RunState::Title);
    click_text(&mut app, &clock, "Press Enter to Start");
    assert_eq!(
        click_text(&mut app, &clock, "Press Enter to confirm"),
        UiCommandCandidate::Command(CommandIntent::Wait)
    );
    assert_eq!(app.run_state(), aihack::core::RunState::Playing);
    assert_eq!(app.revision().turn, 0);
}
