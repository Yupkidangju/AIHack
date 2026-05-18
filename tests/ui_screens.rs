use aihack::core::{
    action::CommandIntent,
    session::{GameSession, RunState},
};
use aihack::domain::combat::DeathCause;

/// [v0.2.0] Phase 17: Title 상태에서 Enter 입력 시 CharacterCreation으로 전환한다.
#[test]
fn title_screen_wait_transitions_to_character_creation() {
    let mut session = GameSession::new(42);
    assert!(matches!(session.state, RunState::Title));

    let outcome = session.submit(CommandIntent::Wait);
    assert!(outcome.accepted);
    assert!(matches!(session.state, RunState::CharacterCreation));
}

/// [v0.2.0] Phase 17: CharacterCreation 상태에서 Enter 입력 시 Playing으로 전환한다.
#[test]
fn character_creation_wait_transitions_to_playing() {
    let mut session = GameSession::new(42);
    session.submit(CommandIntent::Wait); // Title -> CharacterCreation
    assert!(matches!(session.state, RunState::CharacterCreation));

    let outcome = session.submit(CommandIntent::Wait);
    assert!(outcome.accepted);
    assert!(matches!(session.state, RunState::Playing));
}

/// [v0.2.0] Phase 17: Title 상태에서 Quit 입력 시 GameOver로 전환한다.
#[test]
fn title_quit_transitions_to_game_over() {
    let mut session = GameSession::new(42);
    let outcome = session.submit(CommandIntent::Quit);
    assert!(outcome.accepted);
    assert!(matches!(session.state, RunState::GameOver { .. }));
}

/// [v0.2.0] Phase 17: Playing 상태에서 사망 시 GameOver { cause, final_score }로 전환한다.
#[test]
fn player_death_transitions_to_game_over_with_cause_and_score() {
    let mut session = GameSession::new_for_playing(42);
    // 플레이어를 goblin 위치로 이동하여 전투 유발
    let goblin_pos = aihack::core::position::Pos { x: 20, y: 12 };
    session
        .world
        .set_player_location(session.world.current_level(), goblin_pos);

    // 여러 턴 기다리며 전투가 끝날 때까지
    for _ in 0..50 {
        if matches!(session.state, RunState::GameOver { .. }) {
            break;
        }
        session.submit(CommandIntent::Wait);
    }

    assert!(
        matches!(session.state, RunState::GameOver { cause, .. } if matches!(cause, DeathCause::Combat { .. })),
        "사망 시 GameOver {{ cause: Combat, final_score }} 상태가 되어야 한다. 현재 상태: {:?}",
        session.state
    );
}

/// [v0.2.0] Phase 17: GameOver 상태에서는 Quit만 허용된다.
#[test]
fn game_over_rejects_non_quit_commands() {
    let mut session = GameSession::new_for_playing(42);
    // 강제로 GameOver 상태로 설정
    session.state = RunState::GameOver {
        cause: DeathCause::Combat {
            attacker: aihack::core::ids::EntityId(0),
        },
        final_score: 100,
    };

    let outcome = session.submit(CommandIntent::Wait);
    assert!(!outcome.accepted);
}

/// [v0.2.0] Phase 17: MorePrompt 상태에서는 AcknowledgeMore만 허용된다.
#[test]
fn more_prompt_allows_acknowledge_more() {
    let mut session = GameSession::new_for_playing(42);
    session.state = RunState::MorePrompt;

    let outcome = session.submit(CommandIntent::AcknowledgeMore);
    assert!(outcome.accepted);
    assert!(matches!(session.state, RunState::Playing));
}

/// [v0.2.0] Phase 17: AwaitingDirection 상태에서 방향 입력 시 Playing으로 복귀한다.
#[test]
fn awaiting_direction_returns_to_playing() {
    let mut session = GameSession::new_for_playing(42);
    session.state = RunState::AwaitingDirection {
        action: aihack::core::action::DirectionalAction::Open,
    };

    // 동쪽으로 이동 (플레이어 인접 타일)
    let _outcome = session.submit(CommandIntent::Move(aihack::core::position::Direction::East));
    // 인접한 동쪽에 문이 없으면 reject될 수 있지만, 상태는 Playing으로 복귀해야 한다
    assert!(matches!(session.state, RunState::Playing));
}

/// [v0.2.0] Phase 17: render_panels의 화면별 lines 함수들이 비어있지 않다.
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
