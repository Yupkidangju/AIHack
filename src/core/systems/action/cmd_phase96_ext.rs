// ============================================================================
// [v2.32.0 Phase 96-4] 입력/명령 확장 (cmd_phase96_ext.rs)
// 원본: NetHack 3.6.7 src/cmd.c L500-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 명령 매핑 — command_map (cmd.c L500-1000)
// =============================================================================

/// [v2.32.0 96-4] 게임 명령
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommand {
    // 이동
    MoveN,
    MoveS,
    MoveE,
    MoveW,
    MoveNE,
    MoveNW,
    MoveSE,
    MoveSW,
    RunN,
    RunS,
    RunE,
    RunW,
    RunNE,
    RunNW,
    RunSE,
    RunSW,
    // 전투
    Fight,
    Kick,
    Throw,
    Fire,
    ForceAttack,
    // 아이템
    Pickup,
    Drop,
    Eat,
    Drink,
    Read,
    Zap,
    Apply,
    Wear,
    Remove,
    Wield,
    PutOn,
    TakeOff,
    // 주문
    Cast,
    Study,
    // 상호작용
    Open,
    Close,
    Chat,
    Pay,
    Pray,
    Offer,
    Loot,
    Tip,
    // 정보
    Inventory,
    Look,
    Search,
    WhatIs,
    Discoveries,
    // 기타
    Wait,
    Rest,
    GoUp,
    GoDown,
    Save,
    Quit,
    Help,
    Options,
    AutoTravel,
    Explore,
    // 특수
    ExtCommand,
    Repeat,
    SwapWeapon,
    TwoWeapon,
    // 디버그 (위저드 모드)
    WizGenesis,
    WizIdentify,
    WizMap,
    // 알 수 없음
    Unknown,
}

/// [v2.32.0 96-4] 키 → 명령 매핑
pub fn key_to_command(key: char, is_shift: bool, is_ctrl: bool) -> GameCommand {
    if is_ctrl {
        return match key {
            'd' => GameCommand::Kick,
            'p' => GameCommand::Repeat,
            'a' => GameCommand::Pray,
            't' => GameCommand::ExtCommand, // 확장 명령 (텔레포트 등)
            _ => GameCommand::Unknown,
        };
        // 텔레포트 미정의 → Unknown으로 폴백 (위에서 반환)
    }

    if is_shift {
        return match key {
            'H' => GameCommand::RunW,
            'J' => GameCommand::RunS,
            'K' => GameCommand::RunN,
            'L' => GameCommand::RunE,
            'Y' => GameCommand::RunNW,
            'U' => GameCommand::RunNE,
            'B' => GameCommand::RunSW,
            'N' => GameCommand::RunSE,
            'S' => GameCommand::Save,
            _ => GameCommand::Unknown,
        };
    }

    match key {
        'h' => GameCommand::MoveW,
        'j' => GameCommand::MoveS,
        'k' => GameCommand::MoveN,
        'l' => GameCommand::MoveE,
        'y' => GameCommand::MoveNW,
        'u' => GameCommand::MoveNE,
        'b' => GameCommand::MoveSW,
        'n' => GameCommand::MoveSE,
        ',' => GameCommand::Pickup,
        'd' => GameCommand::Drop,
        'e' => GameCommand::Eat,
        'q' => GameCommand::Drink,
        'r' => GameCommand::Read,
        'z' => GameCommand::Zap,
        'a' => GameCommand::Apply,
        'W' => GameCommand::Wear,
        'R' => GameCommand::Remove,
        'w' => GameCommand::Wield,
        'P' => GameCommand::PutOn,
        'T' => GameCommand::TakeOff,
        'Z' => GameCommand::Cast,
        'f' => GameCommand::Fire,
        't' => GameCommand::Throw,
        'o' => GameCommand::Open,
        'c' => GameCommand::Close,
        '#' => GameCommand::ExtCommand,
        'i' => GameCommand::Inventory,
        ':' => GameCommand::Look,
        's' => GameCommand::Search,
        '/' => GameCommand::WhatIs,
        '\\' => GameCommand::Discoveries,
        '.' => GameCommand::Wait,
        '<' => GameCommand::GoUp,
        '>' => GameCommand::GoDown,
        'p' => GameCommand::Pay,
        'F' => GameCommand::Fight,
        '_' => GameCommand::AutoTravel,
        '?' => GameCommand::Help,
        'O' => GameCommand::Options,
        'x' => GameCommand::SwapWeapon,
        'X' => GameCommand::TwoWeapon,
        _ => GameCommand::Unknown,
    }
}

/// [v2.32.0 96-4] 확장 명령 (#명령)
pub fn ext_command(name: &str) -> GameCommand {
    match name {
        "pray" => GameCommand::Pray,
        "offer" => GameCommand::Offer,
        "chat" => GameCommand::Chat,
        "loot" => GameCommand::Loot,
        "tip" => GameCommand::Tip,
        "explore" => GameCommand::Explore,
        "quit" => GameCommand::Quit,
        "save" => GameCommand::Save,
        "ride" => GameCommand::Apply,
        "force" => GameCommand::ForceAttack,
        "wiz genesis" | "genesis" => GameCommand::WizGenesis,
        "wiz identify" | "identify" => GameCommand::WizIdentify,
        "wiz map" | "map" => GameCommand::WizMap,
        _ => GameCommand::Unknown,
    }
}

// 컴파일 에러 방지: 위에서 ctrl+t에 사용할 Teleport 미정의이므로 Unknown 반환
// 실제로는 GameCommand에 Teleport가 없으므로 #[allow] 처리
// → ctrl+t는 #[allow(unreachable_patterns)]로 Unknown 반환
// 위 코드에서 Teleport 사용 → 컴파일 에러 발생 가능하므로 Travel로 대체:

// =============================================================================
// [2] 명령 유효성 — command_validate
// =============================================================================

/// [v2.32.0 96-4] 명령 유효성 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandValidity {
    Valid,
    NeedDirection,
    NeedTarget,
    NeedItem,
    Blocked { reason: String },
    InvalidState { reason: String },
}

/// [v2.32.0 96-4] 명령 유효성 확인
pub fn validate_command(
    cmd: GameCommand,
    has_items: bool,
    is_blind: bool,
    is_immobile: bool,
    is_confused: bool,
) -> CommandValidity {
    if is_immobile {
        match cmd {
            GameCommand::Wait
            | GameCommand::Inventory
            | GameCommand::Look
            | GameCommand::WhatIs
            | GameCommand::Help
            | GameCommand::Options
            | GameCommand::Save
            | GameCommand::Quit => {}
            _ => {
                return CommandValidity::Blocked {
                    reason: "움직일 수 없다!".to_string(),
                }
            }
        }
    }

    match cmd {
        GameCommand::Read if is_blind => CommandValidity::Blocked {
            reason: "앞이 보이지 않아 읽을 수 없다!".to_string(),
        },
        GameCommand::Drop
        | GameCommand::Eat
        | GameCommand::Drink
        | GameCommand::Read
        | GameCommand::Zap
        | GameCommand::Apply
        | GameCommand::Throw
            if !has_items =>
        {
            CommandValidity::Blocked {
                reason: "사용할 아이템이 없다.".to_string(),
            }
        }
        GameCommand::MoveN
        | GameCommand::MoveS
        | GameCommand::MoveE
        | GameCommand::MoveW
        | GameCommand::MoveNE
        | GameCommand::MoveNW
        | GameCommand::MoveSE
        | GameCommand::MoveSW
        | GameCommand::Fight
        | GameCommand::Kick => CommandValidity::Valid,
        GameCommand::Throw | GameCommand::Fire | GameCommand::Zap => CommandValidity::NeedDirection,
        _ => CommandValidity::Valid,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_movement() {
        assert_eq!(key_to_command('h', false, false), GameCommand::MoveW);
        assert_eq!(key_to_command('j', false, false), GameCommand::MoveS);
        assert_eq!(key_to_command('k', false, false), GameCommand::MoveN);
        assert_eq!(key_to_command('l', false, false), GameCommand::MoveE);
    }

    #[test]
    fn test_running() {
        assert_eq!(key_to_command('H', true, false), GameCommand::RunW);
        assert_eq!(key_to_command('L', true, false), GameCommand::RunE);
    }

    #[test]
    fn test_items() {
        assert_eq!(key_to_command(',', false, false), GameCommand::Pickup);
        assert_eq!(key_to_command('d', false, false), GameCommand::Drop);
        assert_eq!(key_to_command('e', false, false), GameCommand::Eat);
    }

    #[test]
    fn test_ext_command() {
        assert_eq!(ext_command("pray"), GameCommand::Pray);
        assert_eq!(ext_command("unknown"), GameCommand::Unknown);
    }

    #[test]
    fn test_validate_immobile() {
        let result = validate_command(GameCommand::MoveN, true, false, true, false);
        assert!(matches!(result, CommandValidity::Blocked { .. }));
    }

    #[test]
    fn test_validate_blind_read() {
        let result = validate_command(GameCommand::Read, true, true, false, false);
        assert!(matches!(result, CommandValidity::Blocked { .. }));
    }

    #[test]
    fn test_validate_no_items() {
        let result = validate_command(GameCommand::Drop, false, false, false, false);
        assert!(matches!(result, CommandValidity::Blocked { .. }));
    }

    #[test]
    fn test_validate_valid() {
        let result = validate_command(GameCommand::MoveN, true, false, false, false);
        assert!(matches!(result, CommandValidity::Valid));
    }
}
