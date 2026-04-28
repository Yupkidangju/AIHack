use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // ============================
    //
    // ============================
    MoveN,
    MoveS,
    MoveE,
    MoveW,
    MoveNE,
    MoveNW,
    MoveSE,
    MoveSW,

    // ============================
    //
    // ============================
    RunN,  // H
    RunS,  // J
    RunE,  // L
    RunW,
    RunNE, // Shift+u
    RunNW, // Shift+y
    RunSE, // Shift+n
    RunSW, // Shift+b

    // ============================
    //
    // ============================
    Travel,

    // ============================
    //
    // ============================
    Wait,   // .
    Search, // s

    // ============================
    //
    // ============================
    Inventory,
    Pickup,
    Drop,      // d
    Quaff,
    Read,
    Eat,       // e
    Wear,
    TakeOff,
    Wield,
    Apply,
    Zap,
    Cast,
    Throw,
    Fire,
    Quiver,
    Swap,
    TwoWeapon,
    Engrave,

    // ============================
    //
    // ============================
    WhatIs,
    LookHere,
    LookAtFloor,
    ShowWeapon,
    ShowArmor,
    ShowRings,
    ShowAmulet,
    ShowTool,
    Discoveries,
    InventoryClass,

    // ============================
    //
    // ============================
    Open,
    Close,
    Kick,
    Loot,    // L ??#loot
    Pay,
    Pray,
    Sit,
    Offer,
    Talk,
    Name,    // N ??#name
    Call,    // C ??#call
    Enhance, // Alt+e ??#enhance
    Invoke,  // Alt+i ??#invoke

    // ============================
    //
    // ============================
    ExtendedCommand,
    Dip,
    Force,
    Jump,
    Ride,
    Rub,
    Tip,
    TurnUndead,
    Untrap,
    Wipe,
    Chat,
    Adjust,
    Monster,

    // ============================
    //
    // ============================
    Conduct,
    Score,
    Overview,
    Version,
    TurnCount,

    // ============================
    //
    // ============================
    Descend,
    Ascend,
    Save,
    Quit,
    Help,
    LogHistory,
    CharacterSheet,
    Cancel,
    Unknown,
}

impl Command {
    ///
    pub fn display_name(&self) -> &'static str {
        match self {
            Command::MoveN => "Move North",
            Command::MoveS => "Move South",
            Command::MoveE => "Move East",
            Command::MoveW => "Move West",
            Command::MoveNE => "Move NE",
            Command::MoveNW => "Move NW",
            Command::MoveSE => "Move SE",
            Command::MoveSW => "Move SW",
            Command::RunN => "Run North",
            Command::RunS => "Run South",
            Command::RunE => "Run East",
            Command::RunW => "Run West",
            Command::RunNE => "Run NE",
            Command::RunNW => "Run NW",
            Command::RunSE => "Run SE",
            Command::RunSW => "Run SW",
            Command::Travel => "Travel",
            Command::Wait => "Wait",
            Command::Search => "Search",
            Command::Inventory => "Inventory",
            Command::Pickup => "Pick up",
            Command::Drop => "Drop",
            Command::Quaff => "Quaff (drink)",
            Command::Read => "Read",
            Command::Eat => "Eat",
            Command::Wear => "Wear armor",
            Command::TakeOff => "Take off armor",
            Command::Wield => "Wield weapon",
            Command::Apply => "Apply (use tool)",
            Command::Zap => "Zap wand",
            Command::Cast => "Cast spell",
            Command::Throw => "Throw",
            Command::Fire => "Fire from quiver",
            Command::Quiver => "Select quiver",
            Command::Swap => "Swap weapons",
            Command::TwoWeapon => "Two-weapon toggle",
            Command::Engrave => "Engrave",
            Command::WhatIs => "What is (identify symbol)",
            Command::LookHere => "Look at position",
            Command::LookAtFloor => "Look at floor",
            Command::ShowWeapon => "Show weapon",
            Command::ShowArmor => "Show armor",
            Command::ShowRings => "Show rings",
            Command::ShowAmulet => "Show amulet",
            Command::ShowTool => "Show tool",
            Command::Discoveries => "Discoveries",
            Command::InventoryClass => "Inventory (by class)",
            Command::Open => "Open door",
            Command::Close => "Close door",
            Command::Kick => "Kick",
            Command::Loot => "Loot",
            Command::Pay => "Pay shopkeeper",
            Command::Pray => "Pray",
            Command::Sit => "Sit down",
            Command::Offer => "Offer sacrifice",
            Command::Talk => "Talk to NPC",
            Command::Name => "Name object",
            Command::Call => "Call (name type)",
            Command::Enhance => "Enhance skills",
            Command::Invoke => "Invoke artifact",
            Command::ExtendedCommand => "Extended command (#)",
            Command::Dip => "Dip item",
            Command::Force => "Force lock",
            Command::Jump => "Jump",
            Command::Ride => "Ride mount",
            Command::Rub => "Rub lamp",
            Command::Tip => "Tip container",
            Command::TurnUndead => "Turn undead",
            Command::Untrap => "Untrap",
            Command::Wipe => "Wipe face",
            Command::Chat => "Chat with NPC",
            Command::Adjust => "Adjust inventory letters",
            Command::Monster => "Monster ability",
            Command::Conduct => "Show conducts",
            Command::Score => "Show score",
            Command::Overview => "Overview",
            Command::Version => "Version",
            Command::TurnCount => "Turn count",
            Command::Descend => "Go down stairs",
            Command::Ascend => "Go up stairs",
            Command::Save => "Save game",
            Command::Quit => "Quit game",
            Command::Help => "Help",
            Command::LogHistory => "Message history",
            Command::CharacterSheet => "Character sheet",
            Command::Cancel => "Cancel",
            Command::Unknown => "(none)",
        }
    }

    ///
    pub fn keybinding(&self) -> &'static str {
        match self {
            Command::MoveN => "k / Up",
            Command::MoveS => "j / Down",
            Command::MoveE => "l / Right",
            Command::MoveW => "h / Left",
            Command::MoveNE => "u",
            Command::MoveNW => "y",
            Command::MoveSE => "n",
            Command::MoveSW => "b",
            Command::RunN => "Shift+K",
            Command::RunS => "Shift+J",
            Command::RunE => "Shift+L",
            Command::RunW => "Shift+H",
            Command::RunNE => "Shift+U",
            Command::RunNW => "Shift+Y",
            Command::RunSE => "Shift+N",
            Command::RunSW => "Shift+B",
            Command::Travel => "_",
            Command::Wait => ".",
            Command::Search => "s",
            Command::Inventory => "i",
            Command::Pickup => ", / g",
            Command::Drop => "d",
            Command::Quaff => "q",
            Command::Read => "r",
            Command::Eat => "e",
            Command::Wear => "W",
            Command::TakeOff => "T",
            Command::Wield => "w",
            Command::Apply => "a",
            Command::Zap => "z",
            Command::Cast => "Z",
            Command::Throw => "t",
            Command::Fire => "f",
            Command::Quiver => "Q",
            Command::Swap => "x",
            Command::TwoWeapon => "X",
            Command::Engrave => "E",
            Command::WhatIs => "/",
            Command::LookHere => ";",
            Command::LookAtFloor => ":",
            Command::ShowWeapon => ")",
            Command::ShowArmor => "[",
            Command::ShowRings => "=",
            Command::ShowAmulet => "\"",
            Command::ShowTool => "(",
            Command::Discoveries => "\\",
            Command::InventoryClass => "I",
            Command::Open => "o",
            Command::Close => "c",
            Command::Kick => "K",
            Command::Loot => "L",
            Command::Pay => "p",
            Command::Pray => "P",
            Command::Sit => "#sit",
            Command::Offer => "#offer",
            Command::Talk => "Alt+t",
            Command::Name => "N / #name",
            Command::Call => "C / #call",
            Command::Enhance => "Alt+e",
            Command::Invoke => "Alt+i",
            Command::ExtendedCommand => "#",
            Command::Dip => "#dip",
            Command::Force => "#force",
            Command::Jump => "#jump",
            Command::Ride => "#ride",
            Command::Rub => "#rub",
            Command::Tip => "#tip",
            Command::TurnUndead => "#turn",
            Command::Untrap => "#untrap",
            Command::Wipe => "#wipe",
            Command::Chat => "#chat",
            Command::Adjust => "#adjust",
            Command::Monster => "#monster",
            Command::Conduct => "#conduct",
            Command::Score => "#score",
            Command::Overview => "Ctrl+O",
            Command::Version => "v",
            Command::TurnCount => "#turncount",
            Command::Descend => ">",
            Command::Ascend => "<",
            Command::Save => "S",
            Command::Quit => "Shift+Q",
            Command::Help => "?",
            Command::LogHistory => "Ctrl+P",
            Command::CharacterSheet => "Shift+C",
            Command::Cancel => "ESC",
            Command::Unknown => "",
        }
    }

    ///
    ///
    pub fn from_extended_str(s: &str) -> Option<Command> {
        match s.trim().to_lowercase().as_str() {
            "dip" => Some(Command::Dip),
            "force" => Some(Command::Force),
            "jump" => Some(Command::Jump),
            "ride" => Some(Command::Ride),
            "rub" => Some(Command::Rub),
            "tip" => Some(Command::Tip),
            "turn" => Some(Command::TurnUndead),
            "untrap" => Some(Command::Untrap),
            "wipe" => Some(Command::Wipe),
            "chat" => Some(Command::Chat),
            "adjust" => Some(Command::Adjust),
            "monster" => Some(Command::Monster),
            "conduct" => Some(Command::Conduct),
            "score" => Some(Command::Score),
            "overview" => Some(Command::Overview),
            "version" => Some(Command::Version),
            "turncount" => Some(Command::TurnCount),
            //
            "pray" => Some(Command::Pray),
            "offer" => Some(Command::Offer),
            "sit" => Some(Command::Sit),
            "loot" => Some(Command::Loot),
            "name" => Some(Command::Name),
            "call" => Some(Command::Call),
            "enhance" => Some(Command::Enhance),
            "invoke" => Some(Command::Invoke),
            "twoweapon" => Some(Command::TwoWeapon),
            "quit" => Some(Command::Quit),
            "save" => Some(Command::Save),
            "help" | "?" => Some(Command::Help),
            "travel" => Some(Command::Travel),
            _ => None,
        }
    }

    ///
    pub fn all_commands_categorized() -> Vec<(&'static str, Vec<Command>)> {
        vec![
            (
                "",
                vec![
                    Command::MoveN,
                    Command::MoveS,
                    Command::MoveE,
                    Command::MoveW,
                    Command::MoveNE,
                    Command::MoveNW,
                    Command::MoveSE,
                    Command::MoveSW,
                    Command::RunN,
                    Command::RunS,
                    Command::RunE,
                    Command::RunW,
                    Command::RunNE,
                    Command::RunNW,
                    Command::RunSE,
                    Command::RunSW,
                    Command::Travel,
                    Command::Wait,
                    Command::Search,
                ],
            ),
            (
                "Item Use",
                vec![
                    Command::Inventory,
                    Command::Pickup,
                    Command::Drop,
                    Command::Eat,
                    Command::Quaff,
                    Command::Read,
                    Command::Wear,
                    Command::TakeOff,
                    Command::Wield,
                    Command::Apply,
                    Command::Zap,
                    Command::Cast,
                    Command::Throw,
                    Command::Fire,
                    Command::Quiver,
                    Command::Swap,
                    Command::TwoWeapon,
                    Command::Engrave,
                ],
            ),
            (
                "Item Info",
                vec![
                    Command::WhatIs,
                    Command::LookHere,
                    Command::LookAtFloor,
                    Command::ShowWeapon,
                    Command::ShowArmor,
                    Command::ShowRings,
                    Command::ShowAmulet,
                    Command::ShowTool,
                    Command::Discoveries,
                    Command::InventoryClass,
                ],
            ),
            (
                "Interaction",
                vec![
                    Command::Open,
                    Command::Close,
                    Command::Kick,
                    Command::Loot,
                    Command::Pay,
                    Command::Pray,
                    Command::Sit,
                    Command::Offer,
                    Command::Talk,
                    Command::Name,
                    Command::Call,
                    Command::Enhance,
                    Command::Invoke,
                ],
            ),
            (
                "Extended (#)",
                vec![
                    Command::ExtendedCommand,
                    Command::Dip,
                    Command::Force,
                    Command::Jump,
                    Command::Ride,
                    Command::Rub,
                    Command::Tip,
                    Command::TurnUndead,
                    Command::Untrap,
                    Command::Wipe,
                    Command::Chat,
                    Command::Adjust,
                    Command::Monster,
                ],
            ),
            (
                "Info",
                vec![
                    Command::Conduct,
                    Command::Score,
                    Command::Overview,
                    Command::Version,
                    Command::TurnCount,
                    Command::Help,
                    Command::LogHistory,
                    Command::CharacterSheet,
                ],
            ),
            (
                "System",
                vec![
                    Command::Descend,
                    Command::Ascend,
                    Command::Save,
                    Command::Quit,
                    Command::Cancel,
                ],
            ),
        ]
    }
}

pub struct InputHandler;

impl InputHandler {
    ///
    pub fn map_key(event: KeyEvent) -> Command {
        //
        if event.modifiers.contains(KeyModifiers::ALT) && event.code == KeyCode::Char('e') {
            return Command::Enhance;
        }
        if event.modifiers.contains(KeyModifiers::ALT) && event.code == KeyCode::Char('i') {
            return Command::Invoke;
        }
        if event.modifiers.contains(KeyModifiers::ALT) && event.code == KeyCode::Char('t') {
            return Command::Talk;
        }

        match event.code {
            //
            KeyCode::Char('h') | KeyCode::Left => Command::MoveW,
            KeyCode::Char('j') | KeyCode::Down => Command::MoveS,
            KeyCode::Char('k') | KeyCode::Up => Command::MoveN,
            KeyCode::Char('l') | KeyCode::Right => Command::MoveE,
            KeyCode::Char('y') => Command::MoveNW,
            KeyCode::Char('u') => Command::MoveNE,
            KeyCode::Char('b') => Command::MoveSW,
            KeyCode::Char('n') => Command::MoveSE,

            //
            KeyCode::Char('H') => Command::RunW,
            KeyCode::Char('J') => Command::RunS,
            KeyCode::Char('K') => Command::RunN,
            KeyCode::Char('L') => Command::RunE,
            KeyCode::Char('Y') => Command::RunNW,
            KeyCode::Char('U') => Command::RunNE,
            KeyCode::Char('B') => Command::RunSW,
            KeyCode::Char('N') => Command::RunSE,

            //
            KeyCode::Char('.') => Command::Wait,
            // Travel
            KeyCode::Char('_') => Command::Travel,

            //
            KeyCode::Char('i') => Command::Inventory,
            KeyCode::Char(',') | KeyCode::Char('g') => Command::Pickup,
            KeyCode::Char('d') => Command::Drop,
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::SHIFT) => Command::Quit,
            KeyCode::Char('q') => Command::Quaff,
            KeyCode::Char('r') => Command::Read,
            KeyCode::Char('e') => Command::Eat,
            KeyCode::Char('W') => Command::Wear,
            KeyCode::Char('T') => Command::TakeOff,
            KeyCode::Char('w') => Command::Wield,
            KeyCode::Char('a') => Command::Apply,
            KeyCode::Char('z') => Command::Zap,
            KeyCode::Char('Z') => Command::Cast,
            KeyCode::Char('t') => Command::Throw,
            KeyCode::Char('f') => Command::Fire,
            KeyCode::Char('Q') => Command::Quiver,
            KeyCode::Char('x') => Command::Swap,
            KeyCode::Char('X') => Command::TwoWeapon,
            KeyCode::Char('E') => Command::Engrave,

            //
            KeyCode::Char('/') => Command::WhatIs,
            KeyCode::Char(';') => Command::LookHere,
            KeyCode::Char(':') => Command::LookAtFloor,
            KeyCode::Char(')') => Command::ShowWeapon,
            KeyCode::Char('[') => Command::ShowArmor,
            KeyCode::Char('=') => Command::ShowRings,
            KeyCode::Char('"') => Command::ShowAmulet,
            KeyCode::Char('(') => Command::ShowTool,
            KeyCode::Char('\\') => Command::Discoveries,
            KeyCode::Char('I') => Command::InventoryClass,

            //
            KeyCode::Char('o') => Command::Open,
            KeyCode::Char('c') => Command::Close,
            KeyCode::Char('p') => Command::Pay,
            KeyCode::Char('P') => Command::Pray,
            KeyCode::Char('s') => Command::Search,
            KeyCode::Char('v') => Command::Version,
            KeyCode::Char('C') => Command::Call,
            KeyCode::Char('O') => Command::Offer,
            //

            //
            KeyCode::Char('>') => Command::Descend,
            KeyCode::Char('<') => Command::Ascend,
            KeyCode::Char('S') => Command::Save,
            KeyCode::Char('?') => Command::Help,

            //
            KeyCode::Char('#') => Command::ExtendedCommand,

            //
            KeyCode::Esc => Command::Cancel,

            _ => Command::Unknown,
        }
    }
}
