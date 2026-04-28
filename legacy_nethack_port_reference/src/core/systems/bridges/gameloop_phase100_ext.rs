// ============================================================================
// [v2.36.0 Phase 100-2] 전체 게임 루프 통합 (gameloop_phase100_ext.rs)
// 원본: NetHack 3.6.7 src/allmain.c + main.c 핵심 게임 루프 통합
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 게임 상태 머신 — game_state_machine
// =============================================================================

/// [v2.36.0 100-2] 게임 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    CharacterCreation,
    Playing,
    Inventory,
    Shopping,
    Dialog,
    LevelTransition,
    GameOver,
    Ascension,
    Paused,
    Saving,
    Loading,
}

/// [v2.36.0 100-2] 턴 페이즈
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    PlayerInput,
    PlayerAction,
    MonsterAction,
    EnvironmentUpdate,
    StatusUpdate,
    VisionUpdate,
    MessageFlush,
    TurnEnd,
}

/// [v2.36.0 100-2] 턴 결과
#[derive(Debug, Clone)]
pub struct TurnResult {
    pub phase_completed: Vec<TurnPhase>,
    pub messages: Vec<String>,
    pub state_change: Option<GameState>,
    pub turn_number: i32,
    pub player_acted: bool,
    pub monsters_acted: i32,
    pub events: Vec<String>,
}

/// [v2.36.0 100-2] 턴 실행
pub fn execute_turn(
    current_state: GameState,
    turn_number: i32,
    player_action: &str,
    monster_count: i32,
) -> TurnResult {
    if current_state != GameState::Playing {
        return TurnResult {
            phase_completed: vec![],
            messages: vec!["게임이 진행 중이 아닙니다.".to_string()],
            state_change: None,
            turn_number,
            player_acted: false,
            monsters_acted: 0,
            events: vec![],
        };
    }

    let mut phases = Vec::new();
    let mut messages = Vec::new();
    let mut events = Vec::new();
    let mut state_change = None;

    // 1. 플레이어 입력 처리
    phases.push(TurnPhase::PlayerInput);

    // 2. 플레이어 행동
    phases.push(TurnPhase::PlayerAction);
    match player_action {
        "이동" => messages.push("이동했다.".to_string()),
        "공격" => {
            messages.push("공격했다!".to_string());
            events.push("전투 발생".to_string());
        }
        "사용" => messages.push("아이템을 사용했다.".to_string()),
        "계단" => {
            messages.push("계단을 이용한다.".to_string());
            state_change = Some(GameState::LevelTransition);
            events.push("레벨 전환".to_string());
        }
        "저장" => {
            state_change = Some(GameState::Saving);
            events.push("저장 요청".to_string());
        }
        "종료" => {
            state_change = Some(GameState::GameOver);
            events.push("게임 종료".to_string());
        }
        _ => messages.push(format!("{} 행동 수행.", player_action)),
    }

    // 3. 몬스터 행동
    phases.push(TurnPhase::MonsterAction);
    let monsters_acted = monster_count;

    // 4. 환경 갱신
    phases.push(TurnPhase::EnvironmentUpdate);

    // 5. 상태이상 갱신
    phases.push(TurnPhase::StatusUpdate);

    // 6. 시야 갱신
    phases.push(TurnPhase::VisionUpdate);

    // 7. 메시지 플러시
    phases.push(TurnPhase::MessageFlush);

    // 8. 턴 종료
    phases.push(TurnPhase::TurnEnd);

    TurnResult {
        phase_completed: phases,
        messages,
        state_change,
        turn_number: turn_number + 1,
        player_acted: true,
        monsters_acted,
        events,
    }
}

// =============================================================================
// [2] 게임 초기화 — game_init
// =============================================================================

/// [v2.36.0 100-2] 초기화 설정
#[derive(Debug, Clone)]
pub struct GameInitConfig {
    pub player_name: String,
    pub player_race: String,
    pub player_role: String,
    pub player_gender: String,
    pub player_alignment: String,
    pub difficulty: String,
    pub seed: Option<u64>,
    pub tutorial_mode: bool,
}

/// [v2.36.0 100-2] 초기화 결과
#[derive(Debug, Clone)]
pub struct GameInitResult {
    pub success: bool,
    pub starting_level: i32,
    pub starting_hp: i32,
    pub starting_mp: i32,
    pub starting_items: Vec<String>,
    pub welcome_message: String,
}

/// [v2.36.0 100-2] 게임 초기화
pub fn initialize_game(config: &GameInitConfig) -> GameInitResult {
    let (hp, mp, items) = match config.player_role.as_str() {
        "전사" | "barbarian" => (
            16,
            2,
            vec![
                "양손검".to_string(),
                "사슬 갑옷".to_string(),
                "식량".to_string(),
            ],
        ),
        "마법사" | "wizard" => (
            10,
            15,
            vec![
                "아셈".to_string(),
                "로브".to_string(),
                "마법서 3권".to_string(),
            ],
        ),
        "도적" | "rogue" => (
            12,
            4,
            vec![
                "단검".to_string(),
                "가죽 갑옷".to_string(),
                "도구 세트".to_string(),
            ],
        ),
        "기사" | "knight" => (
            14,
            4,
            vec![
                "장검".to_string(),
                "판금 갑옷".to_string(),
                "방패".to_string(),
            ],
        ),
        "성직자" | "priest" => (
            12,
            10,
            vec!["철퇴".to_string(), "로브".to_string(), "성수".to_string()],
        ),
        "레인저" | "ranger" => (
            13,
            4,
            vec![
                "활".to_string(),
                "화살 50개".to_string(),
                "가죽 갑옷".to_string(),
            ],
        ),
        "발키리" | "valkyrie" => (
            14,
            2,
            vec![
                "장검".to_string(),
                "방패".to_string(),
                "모피 갑옷".to_string(),
            ],
        ),
        "수도승" | "monk" => (
            12,
            8,
            vec![
                "가죽 장갑".to_string(),
                "로브".to_string(),
                "식량".to_string(),
            ],
        ),
        _ => (
            12,
            4,
            vec![
                "단검".to_string(),
                "가죽 갑옷".to_string(),
                "식량".to_string(),
            ],
        ),
    };

    let welcome = format!(
        "{}! 당신은 {} {} {}입니다. 운명의 동굴에 오신 것을 환영합니다!",
        config.player_name,
        config.player_race,
        config.player_role,
        if config.tutorial_mode {
            "(튜토리얼 모드)"
        } else {
            ""
        },
    );

    GameInitResult {
        success: true,
        starting_level: 1,
        starting_hp: hp,
        starting_mp: mp,
        starting_items: items,
        welcome_message: welcome,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_move() {
        let result = execute_turn(GameState::Playing, 1, "이동", 5);
        assert!(result.player_acted);
        assert_eq!(result.phase_completed.len(), 8);
    }

    #[test]
    fn test_turn_attack() {
        let result = execute_turn(GameState::Playing, 1, "공격", 3);
        assert!(result.events.iter().any(|e| e.contains("전투")));
    }

    #[test]
    fn test_turn_stairs() {
        let result = execute_turn(GameState::Playing, 1, "계단", 0);
        assert_eq!(result.state_change, Some(GameState::LevelTransition));
    }

    #[test]
    fn test_turn_not_playing() {
        let result = execute_turn(GameState::MainMenu, 1, "이동", 0);
        assert!(!result.player_acted);
    }

    #[test]
    fn test_init_warrior() {
        let config = GameInitConfig {
            player_name: "용사".to_string(),
            player_race: "인간".to_string(),
            player_role: "전사".to_string(),
            player_gender: "남성".to_string(),
            player_alignment: "합법".to_string(),
            difficulty: "보통".to_string(),
            seed: Some(42),
            tutorial_mode: false,
        };
        let result = initialize_game(&config);
        assert!(result.success);
        assert_eq!(result.starting_hp, 16);
    }

    #[test]
    fn test_init_wizard() {
        let config = GameInitConfig {
            player_name: "마도사".to_string(),
            player_race: "엘프".to_string(),
            player_role: "마법사".to_string(),
            player_gender: "여성".to_string(),
            player_alignment: "중립".to_string(),
            difficulty: "보통".to_string(),
            seed: None,
            tutorial_mode: true,
        };
        let result = initialize_game(&config);
        assert_eq!(result.starting_mp, 15);
        assert!(result.welcome_message.contains("튜토리얼"));
    }

    #[test]
    fn test_turn_save() {
        let result = execute_turn(GameState::Playing, 100, "저장", 0);
        assert_eq!(result.state_change, Some(GameState::Saving));
    }

    #[test]
    fn test_turn_counter() {
        let result = execute_turn(GameState::Playing, 99, "이동", 3);
        assert_eq!(result.turn_number, 100);
    }
}
