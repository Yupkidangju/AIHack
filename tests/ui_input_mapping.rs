use aihack::{
    core::{CommandIntent, Direction, GameSession, Pos},
    ui::tui::{
        compute_layout, key_to_candidate, keyboard_baseline, map_mouse_event, UiCommandCandidate,
        UiInputEvent, UiPanel, Viewport,
    },
};

#[test]
fn keyboard_baseline_maps_commands() {
    let baseline = keyboard_baseline();
    assert!(baseline.contains(&('h', UiInputEvent::Key(CommandIntent::Move(Direction::West)))));
    assert!(baseline.contains(&(
        'y',
        UiInputEvent::Key(CommandIntent::Move(Direction::NorthWest))
    )));
    assert!(baseline.contains(&(
        'u',
        UiInputEvent::Key(CommandIntent::Move(Direction::NorthEast))
    )));
    assert!(baseline.contains(&(
        'b',
        UiInputEvent::Key(CommandIntent::Move(Direction::SouthWest))
    )));
    assert!(baseline.contains(&(
        'n',
        UiInputEvent::Key(CommandIntent::Move(Direction::SouthEast))
    )));
    assert!(baseline.contains(&('s', UiInputEvent::Key(CommandIntent::Search))));
    assert!(baseline.contains(&('K', UiInputEvent::Key(CommandIntent::Kick(Direction::East)))));
    assert!(baseline.contains(&('p', UiInputEvent::Key(CommandIntent::Pray))));
    assert!(baseline.contains(&('S', UiInputEvent::SaveRequest)));
    assert!(baseline.contains(&('L', UiInputEvent::LoadRequest)));

    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    assert!(matches!(
        key_to_candidate('w', &observation),
        Some(UiCommandCandidate::Command(CommandIntent::Wield { .. }))
    ));
    assert!(matches!(
        key_to_candidate('t', &observation),
        Some(UiCommandCandidate::Command(CommandIntent::Throw { .. }))
    ));
    assert!(matches!(
        key_to_candidate('z', &observation),
        Some(UiCommandCandidate::Command(CommandIntent::Zap { .. }))
    ));
    assert!(matches!(
        key_to_candidate('r', &observation),
        Some(UiCommandCandidate::Command(CommandIntent::Read { .. }))
    ));
    assert_eq!(key_to_candidate('o', &observation), None);
}

#[test]
fn mouse_mapping_matches_layout_contract() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let layout = compute_layout(100, 32);
    let viewport = Viewport::from_rect(Pos { x: 0, y: 0 }, Pos { x: 30, y: 12 }, layout.map);
    let candidate = map_mouse_event(
        UiInputEvent::MouseHover {
            column: layout.map.x + 5,
            row: layout.map.y + 7,
        },
        layout,
        viewport,
        &observation,
    )
    .unwrap();
    assert_eq!(candidate, UiCommandCandidate::Inspect(Pos { x: 5, y: 7 }));

    let move_candidate = map_mouse_event(
        UiInputEvent::MouseClick {
            column: layout.map.x + layout.map.width / 2 + 1,
            row: layout.map.y + layout.map.height / 2,
        },
        layout,
        viewport,
        &observation,
    )
    .unwrap();
    assert_eq!(
        move_candidate,
        UiCommandCandidate::Command(CommandIntent::Move(Direction::East))
    );

    let focus = map_mouse_event(
        UiInputEvent::MouseClick {
            column: layout.status.x,
            row: layout.status.y,
        },
        layout,
        viewport,
        &observation,
    )
    .unwrap();
    assert_eq!(focus, UiCommandCandidate::Focus(UiPanel::Status));
}

#[test]
fn save_load_request_bridges_core_api() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let layout = compute_layout(100, 32);
    let viewport = Viewport::from_rect(Pos { x: 0, y: 0 }, Pos { x: 0, y: 0 }, layout.map);
    assert_eq!(
        map_mouse_event(UiInputEvent::SaveRequest, layout, viewport, &observation),
        Some(UiCommandCandidate::Save)
    );
    assert_eq!(
        map_mouse_event(UiInputEvent::LoadRequest, layout, viewport, &observation),
        Some(UiCommandCandidate::Load)
    );
}

#[test]
fn hover_inspect_is_non_turn() {
    let mut app = aihack::ui::tui::TuiApp::new(
        GameSession::new_for_playing(42),
        aihack::ui::tui::UiRuntimeConfig::default(),
    );
    let before_turn = app.observation().turn;
    let before_hash = app.session.snapshot().stable_hash();
    app.handle_candidate(
        UiCommandCandidate::Inspect(Pos { x: 6, y: 5 }),
        std::path::Path::new("/tmp/unused-save.json"),
        std::path::Path::new("/tmp/unused-load.json"),
    )
    .unwrap();
    assert_eq!(app.observation().turn, before_turn);
    assert_eq!(app.session.snapshot().stable_hash(), before_hash);
    assert_eq!(app.hovered_pos(), Some(Pos { x: 6, y: 5 }));
}

#[test]
fn inventory_click_selection_matches_keyboard_flow() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let layout = compute_layout(100, 32);
    let viewport = Viewport::from_rect(Pos { x: 0, y: 0 }, observation.player_pos, layout.map);
    let clicked = map_mouse_event(
        UiInputEvent::MouseClick {
            column: layout.inspect.x + 1,
            row: layout.inspect.y + 1,
        },
        layout,
        viewport,
        &observation,
    )
    .unwrap();
    assert_eq!(clicked, key_to_candidate('w', &observation).unwrap());
}
