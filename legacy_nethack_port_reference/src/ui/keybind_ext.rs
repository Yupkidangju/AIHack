// ============================================================================
// [v2.38.0 R26-5] 입력/명령 매핑 (keybind_ext.rs)
// 원본: NetHack 3.6.7 cmd.c 키 매핑 확장
// 명령어→키, 키→명령어, 리바인드
// ============================================================================

use std::collections::HashMap;

/// [v2.38.0 R26-5] 게임 명령
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameCommand {
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
    Wait,
    Search,
    Kick,
    Open,
    Close,
    PickUp,
    Drop,
    Inventory,
    Eat,
    Quaff,
    Read,
    Zap,
    Wear,
    Wield,
    Remove,
    Cast,
    Pray,
    Offer,
    Engrave,
    Apply,
    Throw,
    Fire,
    Ride,
    Look,
    Help,
    Save,
    Quit,
    GoUp,
    GoDown,
}

/// [v2.38.0 R26-5] 기본 키 바인딩
pub fn default_keybinds() -> HashMap<char, GameCommand> {
    let mut m = HashMap::new();
    m.insert('k', GameCommand::MoveN);
    m.insert('j', GameCommand::MoveS);
    m.insert('l', GameCommand::MoveE);
    m.insert('h', GameCommand::MoveW);
    m.insert('y', GameCommand::MoveNW);
    m.insert('u', GameCommand::MoveNE);
    m.insert('b', GameCommand::MoveSW);
    m.insert('n', GameCommand::MoveSE);
    m.insert('.', GameCommand::Wait);
    m.insert('s', GameCommand::Search);
    m.insert(',', GameCommand::PickUp);
    m.insert('d', GameCommand::Drop);
    m.insert('i', GameCommand::Inventory);
    m.insert('e', GameCommand::Eat);
    m.insert('q', GameCommand::Quaff);
    m.insert('r', GameCommand::Read);
    m.insert('z', GameCommand::Zap);
    m.insert('w', GameCommand::Wield);
    m.insert('t', GameCommand::Throw);
    m.insert('f', GameCommand::Fire);
    m.insert('a', GameCommand::Apply);
    m.insert('p', GameCommand::Pray);
    m.insert('<', GameCommand::GoUp);
    m.insert('>', GameCommand::GoDown);
    m.insert('S', GameCommand::Save);
    m.insert(':', GameCommand::Look);
    m
}

/// [v2.38.0 R26-5] 키→명령 조회
pub fn lookup_command(binds: &HashMap<char, GameCommand>, key: char) -> Option<GameCommand> {
    binds.get(&key).copied()
}

/// [v2.38.0 R26-5] 리바인드
pub fn rebind(binds: &mut HashMap<char, GameCommand>, key: char, cmd: GameCommand) {
    binds.insert(key, cmd);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let binds = default_keybinds();
        assert_eq!(lookup_command(&binds, 'k'), Some(GameCommand::MoveN));
    }

    #[test]
    fn test_not_bound() {
        let binds = default_keybinds();
        assert_eq!(lookup_command(&binds, 'X'), None);
    }

    #[test]
    fn test_rebind() {
        let mut binds = default_keybinds();
        rebind(&mut binds, 'x', GameCommand::Kick);
        assert_eq!(lookup_command(&binds, 'x'), Some(GameCommand::Kick));
    }

    #[test]
    fn test_count() {
        let binds = default_keybinds();
        assert!(binds.len() >= 25);
    }
}
