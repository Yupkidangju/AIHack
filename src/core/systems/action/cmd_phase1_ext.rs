// ============================================================================
// [v2.25.0 Phase 1-2] 커맨드 시스템 확장 (cmd_phase1_ext.rs)
// 원본: NetHack 3.6.7 src/cmd.c L1500-3500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 키 바인딩 테이블 — key_to_command (cmd.c L140-340)
// =============================================================================

/// [v2.25.0 1-2] 커맨드 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameCommand {
    // 이동
    MoveNorth, MoveSouth, MoveEast, MoveWest,
    MoveNE, MoveNW, MoveSE, MoveSW,
    // 특수 이동
    MoveUp, MoveDown, Run, Go, Fight, ForceFight,
    // 행동
    Search, Kick, Open, Close, Pickup, Drop,
    Eat, Drink, Read, Zap, Apply, Throw,
    Fire, Cast, Pray, Offer,
    // 장비
    Wear, TakeOff, Wield, PutOn, Remove, TwoWeapon,
    // 인벤토리
    Inventory, InventoryFull, Look, PickupMenu,
    // 정보
    WhatsHere, WhatsThis, Help, Discoveries, History,
    // 마법
    EnhanceSkill, TurnUndead, Monster,
    // 시스템
    Save, Quit, Options, ExtCmd, Rest, Repeat,
    // 특수
    CountPrefix, Travel, Explore, SwapWeapon,
    // 알 수 없는 입력
    Unknown,
}

/// [v2.25.0 1-2] 키 → 커맨드 매핑 (vi-keys 기본)
/// 원본: cmd.c cmdlist[] / Strstrp()
pub fn key_to_command(key: char, number_pad: bool) -> GameCommand {
    if number_pad {
        // 넘패드 모드
        match key {
            '8' | 'K' => GameCommand::MoveNorth,
            '2' | 'J' => GameCommand::MoveSouth,
            '6' | 'L' => GameCommand::MoveEast,
            '4' | 'H' => GameCommand::MoveWest,
            '9' => GameCommand::MoveNE,
            '7' => GameCommand::MoveNW,
            '3' => GameCommand::MoveSE,
            '1' => GameCommand::MoveSW,
            '5' | '.' => GameCommand::Rest,
            _ => key_to_command_common(key),
        }
    } else {
        // vi-key 모드 (기본)
        match key {
            'k' => GameCommand::MoveNorth,
            'j' => GameCommand::MoveSouth,
            'l' => GameCommand::MoveEast,
            'h' => GameCommand::MoveWest,
            'y' => GameCommand::MoveNW,
            'u' => GameCommand::MoveNE,
            'b' => GameCommand::MoveSW,
            'n' => GameCommand::MoveSE,
            '.' => GameCommand::Rest,
            _ => key_to_command_common(key),
        }
    }
}

/// [v2.25.0 1-2] 공통 키 매핑
fn key_to_command_common(key: char) -> GameCommand {
    match key {
        '<' => GameCommand::MoveUp,
        '>' => GameCommand::MoveDown,
        's' => GameCommand::Search,
        'i' => GameCommand::Inventory,
        'I' => GameCommand::InventoryFull,
        'o' => GameCommand::Open,
        'c' => GameCommand::Close,
        'e' => GameCommand::Eat,
        'q' => GameCommand::Drink,
        'r' => GameCommand::Read,
        'z' => GameCommand::Zap,
        'a' => GameCommand::Apply,
        't' => GameCommand::Throw,
        'f' => GameCommand::Fire,
        'Z' => GameCommand::Cast,
        'd' => GameCommand::Drop,
        'w' => GameCommand::Wield,
        'W' => GameCommand::Wear,
        'T' => GameCommand::TakeOff,
        'P' => GameCommand::PutOn,
        'R' => GameCommand::Remove,
        'D' => GameCommand::Drop,
        ',' => GameCommand::Pickup,
        ':' => GameCommand::WhatsHere,
        '/' => GameCommand::WhatsThis,
        '?' => GameCommand::Help,
        '#' => GameCommand::ExtCmd,
        'S' => GameCommand::Save,
        'Q' => GameCommand::Quit,
        'O' => GameCommand::Options,
        'p' => GameCommand::Pray,
        '_' => GameCommand::Travel,
        'X' => GameCommand::Explore,
        'x' => GameCommand::SwapWeapon,
        'F' => GameCommand::ForceFight,
        'g' => GameCommand::Go,
        'G' => GameCommand::Run,
        '^' => GameCommand::TurnUndead,
        '\x01' => GameCommand::Repeat, // Ctrl-A
        _ => GameCommand::Unknown,
    }
}

// =============================================================================
// [2] 커맨드 반복 제한 — command_repeat_limit (cmd.c L3200-3350)
// =============================================================================

/// [v2.25.0 1-2] 커맨드 반복 가능 여부
/// 원본: cmd.c 이동/탐색만 반복 가능, 위험 행동은 불가
pub fn can_repeat_command(cmd: GameCommand) -> bool {
    matches!(
        cmd,
        GameCommand::MoveNorth
            | GameCommand::MoveSouth
            | GameCommand::MoveEast
            | GameCommand::MoveWest
            | GameCommand::MoveNE
            | GameCommand::MoveNW
            | GameCommand::MoveSE
            | GameCommand::MoveSW
            | GameCommand::Search
            | GameCommand::Rest
            | GameCommand::Pickup
    )
}

/// [v2.25.0 1-2] 이동 커맨드인지 판정
pub fn is_movement_command(cmd: GameCommand) -> bool {
    matches!(
        cmd,
        GameCommand::MoveNorth
            | GameCommand::MoveSouth
            | GameCommand::MoveEast
            | GameCommand::MoveWest
            | GameCommand::MoveNE
            | GameCommand::MoveNW
            | GameCommand::MoveSE
            | GameCommand::MoveSW
            | GameCommand::MoveUp
            | GameCommand::MoveDown
            | GameCommand::Run
            | GameCommand::Go
            | GameCommand::Travel
    )
}

/// [v2.25.0 1-2] 턴 소모 커맨드인지 판정
pub fn consumes_turn(cmd: GameCommand) -> bool {
    match cmd {
        // 정보 조회는 턴 소모 없음
        GameCommand::Inventory
        | GameCommand::InventoryFull
        | GameCommand::WhatsHere
        | GameCommand::WhatsThis
        | GameCommand::Help
        | GameCommand::Discoveries
        | GameCommand::History
        | GameCommand::Options
        | GameCommand::ExtCmd
        | GameCommand::Unknown => false,
        // 시스템 명령은 턴 소모 없음
        GameCommand::Save | GameCommand::Quit => false,
        // 그 외 모든 행동은 턴 소모
        _ => true,
    }
}

// =============================================================================
// [3] 방향 키 + 이동 삼합 — 이동 유효성 검사 (cmd.c 이동 관련)
// =============================================================================

/// [v2.25.0 1-2] 이동 결과 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveResult {
    /// 정상 이동
    Success { new_x: i32, new_y: i32 },
    /// 벽/장애물로 이동 불가
    Blocked { message: String },
    /// 닫힌 문 → 열기 시도
    ClosedDoor { door_x: i32, door_y: i32 },
    /// 몬스터와 전투
    Attack { target_x: i32, target_y: i32 },
    /// 함정 발동
    Trap { trap_x: i32, trap_y: i32 },
    /// 계단 이동 (레벨 전환)
    LevelChange { direction: i32 }, // 1=아래, -1=위
}

/// [v2.25.0 1-2] 이동 유효성 검사
/// 원본: cmd.c domove() 단순화
pub fn check_move(
    player_x: i32,
    player_y: i32,
    dx: i32,
    dy: i32,
    dest_walkable: bool,
    dest_has_monster: bool,
    dest_has_closed_door: bool,
    dest_has_trap: bool,
    is_stairs_up: bool,
    is_stairs_down: bool,
    map_width: i32,
    map_height: i32,
) -> MoveResult {
    let new_x = player_x + dx;
    let new_y = player_y + dy;

    // 범위 검사
    if new_x < 0 || new_x >= map_width || new_y < 0 || new_y >= map_height {
        return MoveResult::Blocked {
            message: "맵 가장자리를 넘을 수 없다.".to_string(),
        };
    }

    // 계단 (제자리 이동 + 위/아래 키)
    if dx == 0 && dy == 0 {
        if is_stairs_up {
            return MoveResult::LevelChange { direction: -1 };
        }
        if is_stairs_down {
            return MoveResult::LevelChange { direction: 1 };
        }
    }

    // 몬스터 → 전투
    if dest_has_monster {
        return MoveResult::Attack {
            target_x: new_x,
            target_y: new_y,
        };
    }

    // 닫힌 문
    if dest_has_closed_door {
        return MoveResult::ClosedDoor {
            door_x: new_x,
            door_y: new_y,
        };
    }

    // 함정
    if dest_has_trap {
        return MoveResult::Trap {
            trap_x: new_x,
            trap_y: new_y,
        };
    }

    // 이동 가능 여부
    if !dest_walkable {
        return MoveResult::Blocked {
            message: "벽이 가로막고 있다.".to_string(),
        };
    }

    MoveResult::Success {
        new_x,
        new_y,
    }
}

// =============================================================================
// [4] 커맨드 도움말 — command_help (cmd.c L3400-3500)
// =============================================================================

/// [v2.25.0 1-2] 커맨드 도움말 텍스트
pub fn command_help(cmd: GameCommand) -> &'static str {
    match cmd {
        GameCommand::MoveNorth => "북쪽으로 이동",
        GameCommand::MoveSouth => "남쪽으로 이동",
        GameCommand::MoveEast => "동쪽으로 이동",
        GameCommand::MoveWest => "서쪽으로 이동",
        GameCommand::MoveNE => "북동쪽으로 이동",
        GameCommand::MoveNW => "북서쪽으로 이동",
        GameCommand::MoveSE => "남동쪽으로 이동",
        GameCommand::MoveSW => "남서쪽으로 이동",
        GameCommand::MoveUp => "위층으로 이동 (계단)",
        GameCommand::MoveDown => "아래층으로 이동 (계단)",
        GameCommand::Search => "주변 탐색 (숨겨진 문/함정)",
        GameCommand::Kick => "발차기",
        GameCommand::Open => "문 열기",
        GameCommand::Close => "문 닫기",
        GameCommand::Pickup => "아이템 줍기",
        GameCommand::Drop => "아이템 내려놓기",
        GameCommand::Eat => "음식 먹기",
        GameCommand::Drink => "포션 마시기",
        GameCommand::Read => "두루마리/마법서 읽기",
        GameCommand::Zap => "지팡이 사용",
        GameCommand::Apply => "도구 사용",
        GameCommand::Throw => "아이템 던지기",
        GameCommand::Fire => "원거리 무기 발사",
        GameCommand::Cast => "마법 시전",
        GameCommand::Pray => "기도하기",
        GameCommand::Wear => "갑옷 입기",
        GameCommand::TakeOff => "갑옷 벗기",
        GameCommand::Wield => "무기 장비",
        GameCommand::Inventory => "소지품 목록",
        GameCommand::Save => "게임 저장",
        GameCommand::Quit => "게임 종료",
        GameCommand::Rest => "한 턴 쉬기",
        GameCommand::Travel => "자동 이동 (목적지 지정)",
        _ => "알 수 없는 커맨드",
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- key_to_command ---

    #[test]
    fn test_vi_keys() {
        assert_eq!(key_to_command('k', false), GameCommand::MoveNorth);
        assert_eq!(key_to_command('j', false), GameCommand::MoveSouth);
        assert_eq!(key_to_command('h', false), GameCommand::MoveWest);
        assert_eq!(key_to_command('l', false), GameCommand::MoveEast);
    }

    #[test]
    fn test_numpad_keys() {
        assert_eq!(key_to_command('8', true), GameCommand::MoveNorth);
        assert_eq!(key_to_command('2', true), GameCommand::MoveSouth);
        assert_eq!(key_to_command('6', true), GameCommand::MoveEast);
        assert_eq!(key_to_command('4', true), GameCommand::MoveWest);
    }

    #[test]
    fn test_common_keys() {
        assert_eq!(key_to_command('i', false), GameCommand::Inventory);
        assert_eq!(key_to_command('e', false), GameCommand::Eat);
        assert_eq!(key_to_command('S', false), GameCommand::Save);
        assert_eq!(key_to_command('<', false), GameCommand::MoveUp);
        assert_eq!(key_to_command('>', false), GameCommand::MoveDown);
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(key_to_command('~', false), GameCommand::Unknown);
    }

    // --- can_repeat/is_movement/consumes_turn ---

    #[test]
    fn test_can_repeat() {
        assert!(can_repeat_command(GameCommand::MoveNorth));
        assert!(can_repeat_command(GameCommand::Search));
        assert!(!can_repeat_command(GameCommand::Eat));
        assert!(!can_repeat_command(GameCommand::Cast));
    }

    #[test]
    fn test_is_movement() {
        assert!(is_movement_command(GameCommand::MoveNorth));
        assert!(is_movement_command(GameCommand::Travel));
        assert!(!is_movement_command(GameCommand::Eat));
    }

    #[test]
    fn test_consumes_turn() {
        assert!(consumes_turn(GameCommand::MoveNorth));
        assert!(consumes_turn(GameCommand::Eat));
        assert!(!consumes_turn(GameCommand::Inventory));
        assert!(!consumes_turn(GameCommand::Help));
        assert!(!consumes_turn(GameCommand::Save));
    }

    // --- check_move ---

    #[test]
    fn test_move_success() {
        let result = check_move(10, 10, 1, 0, true, false, false, false, false, false, 80, 21);
        assert!(matches!(result, MoveResult::Success { new_x: 11, new_y: 10 }));
    }

    #[test]
    fn test_move_blocked() {
        let result = check_move(10, 10, 1, 0, false, false, false, false, false, false, 80, 21);
        assert!(matches!(result, MoveResult::Blocked { .. }));
    }

    #[test]
    fn test_move_attack() {
        let result = check_move(10, 10, 1, 0, true, true, false, false, false, false, 80, 21);
        assert!(matches!(result, MoveResult::Attack { .. }));
    }

    #[test]
    fn test_move_door() {
        let result = check_move(10, 10, 0, 1, false, false, true, false, false, false, 80, 21);
        assert!(matches!(result, MoveResult::ClosedDoor { .. }));
    }

    #[test]
    fn test_move_out_of_bounds() {
        let result = check_move(0, 0, -1, 0, true, false, false, false, false, false, 80, 21);
        assert!(matches!(result, MoveResult::Blocked { .. }));
    }

    #[test]
    fn test_stairs_down() {
        let result = check_move(10, 10, 0, 0, true, false, false, false, false, true, 80, 21);
        assert!(matches!(result, MoveResult::LevelChange { direction: 1 }));
    }

    // --- command_help ---

    #[test]
    fn test_help_text() {
        assert!(!command_help(GameCommand::MoveNorth).is_empty());
        assert!(!command_help(GameCommand::Eat).is_empty());
    }
}
