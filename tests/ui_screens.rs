use aihack::core::{
    action::CommandIntent,
    session::{GameSession, RunState},
};
use aihack::domain::combat::DeathCause;

mod support;

use support::session_builder::SessionBuilder;

/// Title 상태의 Enter는 CharacterCreation으로 전환해야 한다.
#[test]
fn title_screen_wait_transitions_to_character_creation() {
    let mut session = GameSession::new(42);
    assert!(matches!(session.run_state(), RunState::Title));

    let outcome = session.submit(CommandIntent::Wait);
    assert!(outcome.accepted);
    assert!(matches!(session.run_state(), RunState::CharacterCreation));
}

/// CharacterCreation 상태의 Enter는 Playing으로 전환해야 한다.
#[test]
fn character_creation_wait_transitions_to_playing() {
    let mut session = GameSession::new(42);
    session.submit(CommandIntent::Wait); // Title -> CharacterCreation
    assert!(matches!(session.run_state(), RunState::CharacterCreation));

    let outcome = session.submit(CommandIntent::Wait);
    assert!(outcome.accepted);
    assert!(matches!(session.run_state(), RunState::Playing));
}

/// Title 상태에서 Quit 입력은 종료 상태로 전환해야 한다.
#[test]
fn title_quit_transitions_to_game_over() {
    let mut session = GameSession::new(42);
    let outcome = session.submit(CommandIntent::Quit);
    assert!(outcome.accepted);
    assert!(matches!(session.run_state(), RunState::GameOver { .. }));
}

/// Playing 상태에서 사망하면 cause와 final_score를 가진 GameOver로 전환해야 한다.
#[test]
fn player_death_transitions_to_game_over_with_cause_and_score() {
    let mut session = GameSession::new_for_playing(42);
    // 플레이어를 goblin 위치로 이동하여 전투 유발
    let goblin_pos = aihack::core::position::Pos { x: 20, y: 12 };
    let level = session.world().current_level();
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_location(level, goblin_pos);
    });

    // 여러 턴 기다리며 전투가 끝날 때까지
    for _ in 0..50 {
        if matches!(session.run_state(), RunState::GameOver { .. }) {
            break;
        }
        session.submit(CommandIntent::Wait);
    }

    assert!(
        matches!(session.run_state(), RunState::GameOver { cause, .. } if matches!(cause, DeathCause::Combat { .. })),
        "사망 시 GameOver {{ cause: Combat, final_score }} 상태가 되어야 한다. 현재 상태: {:?}",
        session.run_state()
    );
}

/// GameOver 상태에서는 동결된 종료 행동만 허용해야 한다.
#[test]
fn game_over_rejects_non_quit_commands() {
    let mut session = SessionBuilder::playing(42)
        .run_state(RunState::GameOver {
            cause: DeathCause::Combat {
                attacker: aihack::core::ids::EntityId(0),
            },
            final_score: 100,
        })
        .build();

    let outcome = session.submit(CommandIntent::Wait);
    assert!(!outcome.accepted);
}

/// MorePrompt 상태에서는 AcknowledgeMore만 허용해야 한다.
#[test]
fn more_prompt_allows_acknowledge_more() {
    let mut session = SessionBuilder::playing(42)
        .run_state(RunState::MorePrompt)
        .build();

    let outcome = session.submit(CommandIntent::AcknowledgeMore);
    assert!(outcome.accepted);
    assert!(matches!(session.run_state(), RunState::Playing));
}

/// AwaitingDirection 상태의 방향 입력은 Playing으로 복귀해야 한다.
#[test]
fn awaiting_direction_returns_to_playing() {
    let mut session = SessionBuilder::playing(42)
        .run_state(RunState::AwaitingDirection {
            action: aihack::core::action::DirectionalAction::Open,
        })
        .build();

    // 동쪽으로 이동 (플레이어 인접 타일)
    let _outcome = session.submit(CommandIntent::Move(aihack::core::position::Direction::East));
    // 인접한 동쪽에 문이 없으면 reject될 수 있지만, 상태는 Playing으로 복귀해야 한다
    assert!(matches!(session.run_state(), RunState::Playing));
}

#[test]
fn awaiting_direction_and_inventory_support_non_turn_cancel_and_typed_selection() {
    let mut direction = SessionBuilder::playing(42)
        .run_state(RunState::AwaitingDirection {
            action: aihack::core::action::DirectionalAction::Open,
        })
        .build();
    let before = direction.snapshot().stable_hash();
    let cancelled = direction.submit(CommandIntent::AcknowledgeMore);
    assert!(cancelled.accepted);
    assert!(!cancelled.turn_advanced);
    assert!(matches!(direction.run_state(), RunState::Playing));
    assert_ne!(direction.snapshot().stable_hash(), before);

    let mut inventory = SessionBuilder::playing(42)
        .run_state(RunState::AwaitingInventorySelection {
            action: aihack::core::action::InventoryAction::Wield,
        })
        .build();
    let wielded = inventory.submit(CommandIntent::Wield {
        item: aihack::core::EntityId(5),
    });
    assert!(wielded.accepted);
    assert!(matches!(inventory.run_state(), RunState::Playing));
    assert_eq!(
        inventory.world().inventory().equipped_melee,
        Some(aihack::core::EntityId(5))
    );
}

/// render_panels의 화면별 line 함수는 실제 안내 내용을 제공해야 한다.
#[test]
fn screen_lines_are_not_empty() {
    use aihack::ui::tui::render_panels;

    assert!(!render_panels::title_lines().is_empty());
    assert!(!render_panels::character_creation_lines().is_empty());
    assert!(!render_panels::game_over_lines("test", 1, 1, 0, 0, 42).is_empty());
    assert!(!render_panels::awaiting_direction_lines("open").is_empty());
    assert!(!render_panels::awaiting_inventory_lines("drop").is_empty());
    assert!(!render_panels::more_prompt_lines().is_empty());
}
