// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
//
// [v1.9.0
// =============================================================================
//
//
//
//

use crate::core::dungeon::tile::TileType;
use crate::core::entity::monster::MonsterTemplate;

// =============================================================================
//
// =============================================================================

///
pub fn tile_symbol(typ: TileType) -> char {
    match typ {
        TileType::Stone => ' ',
        TileType::VWall
        | TileType::HWall
        | TileType::TlCorner
        | TileType::TrCorner
        | TileType::BlCorner
        | TileType::BrCorner
        | TileType::CrossWall
        | TileType::TuWall
        | TileType::TdWall
        | TileType::TlWall
        | TileType::TrWall
        | TileType::DbWall => '#',
        TileType::Room => '.',
        TileType::Corr => '#',
        TileType::Door => '+',
        TileType::OpenDoor => '|',
        TileType::SDoor => '#',
        TileType::SCorr => '#',
        TileType::StairsUp => '<',
        TileType::StairsDown => '>',
        TileType::Altar => '_',
        TileType::Fountain => '{',
        TileType::Throne => '\\',
        TileType::Sink => '#',
        TileType::Grave => '|',
        TileType::Pool | TileType::Moat | TileType::Water | TileType::LavaPool => '}',
        TileType::Ice => '.',
        TileType::Air => ' ',
        TileType::Cloud => '#',
        TileType::IronBars => '#',
        TileType::Tree => '#',
        TileType::DrawbridgeUp | TileType::DrawbridgeDown => '.',
        TileType::Ladder => '>',
        TileType::Hole | TileType::TrapDoor => '^',
    }
}

///
pub fn tile_name(typ: TileType) -> &'static str {
    match typ {
        TileType::Stone => "solid stone",
        TileType::VWall | TileType::HWall => "wall",
        TileType::TlCorner
        | TileType::TrCorner
        | TileType::BlCorner
        | TileType::BrCorner
        | TileType::CrossWall
        | TileType::TuWall
        | TileType::TdWall
        | TileType::TlWall
        | TileType::TrWall
        | TileType::DbWall => "wall",
        TileType::Room => "floor of a room",
        TileType::Corr => "corridor",
        TileType::Door => "closed door",
        TileType::OpenDoor => "open door",
        TileType::SDoor => "wall",
        TileType::SCorr => "wall",
        TileType::StairsUp => "staircase up",
        TileType::StairsDown => "staircase down",
        TileType::Ladder => "ladder",
        TileType::Altar => "altar",
        TileType::Fountain => "fountain",
        TileType::Throne => "opulent throne",
        TileType::Sink => "kitchen sink",
        TileType::Grave => "grave",
        TileType::Pool => "pool of water",
        TileType::Moat => "moat",
        TileType::Water => "water",
        TileType::LavaPool => "molten lava",
        TileType::Ice => "ice",
        TileType::Air => "open air",
        TileType::Cloud => "cloud",
        TileType::IronBars => "iron bars",
        TileType::Tree => "tree",
        TileType::DrawbridgeUp => "raised drawbridge",
        TileType::DrawbridgeDown => "lowered drawbridge",
        TileType::Hole => "hole in the floor",
        TileType::TrapDoor => "trap door",
    }
}

///
pub fn tile_description(typ: TileType) -> &'static str {
    match typ {
        TileType::Stone => "Solid rock. You cannot pass through it.",
        TileType::VWall
        | TileType::HWall
        | TileType::TlCorner
        | TileType::TrCorner
        | TileType::BlCorner
        | TileType::BrCorner
        | TileType::CrossWall
        | TileType::TuWall
        | TileType::TdWall
        | TileType::TlWall
        | TileType::TrWall
        | TileType::DbWall => "A wall. Solid and impassable without special means.",
        TileType::Room => "The floor of a room. You can walk here freely.",
        TileType::Corr => "A corridor connecting rooms.",
        TileType::Door => "A closed door. Open it with 'o', kick it with Ctrl+D, or pick the lock.",
        TileType::OpenDoor => "An open doorway. Close it with 'c'.",
        TileType::SDoor => "A wall.",
        TileType::SCorr => "A wall.",
        TileType::StairsUp => "A staircase going up. Use '<' to ascend.",
        TileType::StairsDown => "A staircase going down. Use '>' to descend.",
        TileType::Ladder => "A ladder connecting levels.",
        TileType::Altar => "An altar. You can sacrifice corpses here with #offer, or pray on it.",
        TileType::Fountain => "A fountain. You can drink from it with 'q' or dip items with #dip.",
        TileType::Throne => "An opulent throne. Sitting on it (#sit) may have magical effects.",
        TileType::Sink => "A kitchen sink. You can drink from it or kick it.",
        TileType::Grave => "A grave. You can dig it up with a digging tool.",
        TileType::Pool => "A pool of water. Swimming into it without water walking is dangerous.",
        TileType::Moat => "A moat surrounding a castle.",
        TileType::LavaPool => {
            "Molten lava! Stepping in without fire resistance and levitation is instantly fatal."
        }
        TileType::Water => "Deep water. You need magical breathing or levitation to survive.",
        TileType::Ice => "Ice. You may slip when walking on it.",
        TileType::Air => "Open air. You need levitation or flying to traverse it.",
        TileType::Cloud => "A cloud. Obscures vision.",
        TileType::IronBars => "Iron bars. They block passage but allow seeing through.",
        TileType::Tree => "A tree. It blocks movement.",
        TileType::DrawbridgeUp | TileType::DrawbridgeDown => {
            "A drawbridge. It can be raised or lowered."
        }
        TileType::Hole => "A gaping hole in the floor. Falling through will drop you a level.",
        TileType::TrapDoor => "A trap door. You may fall through it unexpectedly.",
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn monster_info(template: &MonsterTemplate, hp: i32, max_hp: i32) -> String {
    let mut info = Vec::new();

    info.push(format!("{} ({})", template.name, template.symbol));
    info.push(format!("Level: {}", template.level));
    info.push(format!("HP: {}/{}", hp, max_hp));
    info.push(format!("AC: {}", template.ac));
    info.push(format!("Speed: {}", template.movement));

    //
    if !template.attacks.is_empty() {
        let mut atk_strs = Vec::new();
        for atk in &template.attacks {
            atk_strs.push(format!("{}d{} {:?}", atk.dice, atk.sides, atk.adtype));
        }
        info.push(format!("Attacks: {}", atk_strs.join(", ")));
    }

    //
    let mut resists = Vec::new();
    if template.resists_fire() {
        resists.push("fire");
    }
    if template.resists_cold() {
        resists.push("cold");
    }
    if template.resists_elec() {
        resists.push("shock");
    }
    if template.resists_poison() {
        resists.push("poison");
    }
    if template.resists_sleep() {
        resists.push("sleep");
    }
    if !resists.is_empty() {
        info.push(format!("Resists: {}", resists.join(", ")));
    }

    info.join("\n")
}

///
pub fn monster_one_liner(name: &str, level: i8, hp: i32, max_hp: i32, hostile: bool) -> String {
    let attitude = if hostile { "hostile" } else { "peaceful" };
    format!("{} (L{}, {}/{} HP, {})", name, level, hp, max_hp, attitude)
}

// =============================================================================
//
// =============================================================================

///
pub fn symbol_description(ch: char) -> &'static str {
    match ch {
        ' ' => "dark part of a room, or solid rock",
        '.' => "floor of a room, or ice, or lawful altar",
        '#' => "corridor, or iron bars, or tree, or cloud",
        '<' => "staircase up",
        '>' => "staircase down",
        '+' => "closed door, or spellbook",
        '|' => "open door, or wall",
        '-' => "wall",
        '_' => "altar, or iron chain",
        '{' => "fountain",
        '}' => "pool, moat, or lava",
        '\\' => "opulent throne",
        '^' => "trap",
        '"' => "amulet",
        ')' => "weapon",
        '[' => "armor",
        '!' => "potion",
        '?' => "scroll",
        '/' => "wand",
        '=' => "ring",
        '*' => "gem or rock",
        '(' => "useful item (tool)",
        '%' => "food",
        '$' => "gold piece",
        '0' => "iron ball",
        '`' => "boulder or statue",
        '@' => "human or humanoid",
        'a' => "ant or other insect",
        'b' => "blob",
        'c' => "cockatrice or similar",
        'd' => "dog or other canine",
        'e' => "eye or sphere",
        'f' => "feline (cat)",
        'g' => "gremlin or gargoyle",
        'h' => "humanoid",
        'i' => "imp or minor demon",
        'j' => "jelly",
        'k' => "kobold",
        'l' => "leprechaun",
        'm' => "mimic",
        'n' => "nymph",
        'o' => "orc",
        'p' => "piercer",
        'q' => "quadruped",
        'r' => "rodent",
        's' => "spider or scorpion",
        't' => "trapper or lurker above",
        'u' => "horse or unicorn",
        'v' => "vortex",
        'w' => "worm",
        'x' => "xan and other insects",
        'y' => "light / yeti or apelike",
        'z' => "zruty",
        'A' => "angelic being",
        'B' => "bat or bird",
        'C' => "centaur",
        'D' => "dragon",
        'E' => "elemental",
        'F' => "fungus or mold",
        'G' => "gnome",
        'H' => "giant humanoid",
        'I' => "invisible stalker",
        'J' => "jabberwock",
        'K' => "Keystone Kop",
        'L' => "lich",
        'M' => "mummy",
        'N' => "naga",
        'O' => "ogre",
        'P' => "pudding or ooze",
        'Q' => "quantum mechanic",
        'R' => "rust monster or disenchanter",
        'S' => "snake",
        'T' => "troll",
        'U' => "umber hulk",
        'V' => "vampire",
        'W' => "wraith",
        'X' => "xorn",
        'Y' => "apelike creature",
        'Z' => "zombie",
        '&' => "demon or devil",
        '\'' => "golem",
        ':' => "lizard-like creature",
        ';' => "sea monster",
        '~' => "tail of a long worm",
        _ => "something unknown",
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn look_here(
    tile_type: TileType,
    items_here: &[(String, char)],
    trap_name: Option<&str>,
    inscription: Option<&str>,
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("You are on {}.", tile_name(tile_type)));

    if let Some(trap) = trap_name {
        lines.push(format!("There is a {} here.", trap));
    }

    if let Some(insc) = inscription {
        lines.push(format!("Something is written here: \"{}\"", insc));
    }

    match items_here.len() {
        0 => {}
        1 => {
            lines.push(format!("You see here {}.", items_here[0].0));
        }
        _ => {
            lines.push("Things that are here:".to_string());
            for (name, _sym) in items_here {
                lines.push(format!("  {}", name));
            }
        }
    }

    lines
}

// =============================================================================
//
// =============================================================================

///
pub fn help_topics() -> Vec<(&'static str, &'static str)> {
    vec![
        ("?", "Help menu"),
        (
            "Long description of the game",
            "Describes the game of NetHack.",
        ),
        ("List of game commands", "Shows all available commands."),
        ("i", "Inventory - display your current items"),
        ("I", "Inventory - display specific item categories"),
        ("w", "Wield a weapon"),
        ("W", "Wear armor"),
        ("P", "Put on ring/amulet"),
        ("T", "Take off armor"),
        ("R", "Remove ring/amulet"),
        ("d", "Drop items"),
        ("D", "Drop specific types of items"),
        ("a", "Apply (use) a tool"),
        ("e", "Eat something"),
        ("q", "Quaff (drink) a potion"),
        ("r", "Read a scroll or spellbook"),
        ("z", "Zap a wand"),
        ("Z", "Cast a spell"),
        ("t", "Throw an item"),
        ("f", "Fire from quiver"),
        (",", "Pickup items"),
        ("<", "Go upstairs"),
        (">", "Go downstairs"),
        ("o", "Open a door"),
        ("c", "Close a door"),
        ("s", "Search for hidden things"),
        (".", "Rest (wait one turn)"),
        (":", "Look at what's on the floor"),
        (";", "Look at a map position"),
        ("/", "Identify a symbol"),
        ("\\", "Show discovered items"),
        ("@", "Toggle autopickup"),
        ("S", "Save your game"),
        ("#quit", "Quit the game"),
    ]
}

///
pub fn keybinding_help() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Movement", "hjklyubn or numpad"),
        ("Run", "Shift+direction"),
        ("Open", "o"),
        ("Close", "c"),
        ("Search", "s"),
        ("Rest", "., space"),
        ("Inventory", "i"),
        ("Wield", "w"),
        ("Wear", "W"),
        ("Take off", "T"),
        ("Put on", "P"),
        ("Remove", "R"),
        ("Eat", "e"),
        ("Quaff", "q"),
        ("Read", "r"),
        ("Zap", "z"),
        ("Cast spell", "Z"),
        ("Apply tool", "a"),
        ("Throw", "t"),
        ("Fire", "f"),
        ("Drop", "d"),
        ("Pickup", ","),
        ("Loot", "#loot"),
        ("Pray", "#pray"),
        ("Offer", "#offer"),
        ("Go up", "<"),
        ("Go down", ">"),
        ("Look", ";"),
        ("What is", "/"),
        ("Floor", ":"),
        ("Discoveries", "\\"),
        ("Extended command", "#"),
        ("Help", "?"),
        ("Save", "S"),
    ]
}

// =============================================================================
//
// =============================================================================

///
pub fn version_info() -> String {
    format!(
        "NetHack-RS\n\
         Based on NetHack 3.6.7\n\
         Rust implementation by AIHack Team\n\
         Original by the NetHack DevTeam"
    )
}

///
pub fn format_discoveries(discovered: &[(String, String)]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Discoveries:".to_string());

    if discovered.is_empty() {
        lines.push("  You haven't discovered anything yet.".to_string());
        return lines;
    }

    for (real_name, appearance) in discovered {
        if appearance.is_empty() {
            lines.push(format!("  {}", real_name));
        } else {
            lines.push(format!("  {} ({})", real_name, appearance));
        }
    }

    lines
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_description() {
        assert_eq!(
            symbol_description('.'),
            "floor of a room, or ice, or lawful altar"
        );
        assert_eq!(symbol_description('D'), "dragon");
        assert_eq!(symbol_description('@'), "human or humanoid");
    }

    #[test]
    fn test_tile_name() {
        assert_eq!(tile_name(TileType::Room), "floor of a room");
        assert_eq!(tile_name(TileType::StairsDown), "staircase down");
    }

    #[test]
    fn test_help_topics() {
        let topics = help_topics();
        assert!(topics.len() > 10);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct MonsterDetail {
    pub name: String,
    pub symbol: char,
    pub level: i32,
    pub speed: i32,
    pub ac: i32,
    pub magic_resist: i32,
    pub attacks: Vec<String>,
    pub resistances: Vec<String>,
    pub special_traits: Vec<String>,
}

///
pub fn monster_detail_description(detail: &MonsterDetail) -> String {
    let mut desc = format!(
        "{} ('{}') ??Lv {} | Spd {} | AC {} | MR {}%\n",
        detail.name, detail.symbol, detail.level, detail.speed, detail.ac, detail.magic_resist
    );
    if !detail.attacks.is_empty() {
        desc.push_str(&format!("Attacks: {}\n", detail.attacks.join(", ")));
    }
    if !detail.resistances.is_empty() {
        desc.push_str(&format!("Resists: {}\n", detail.resistances.join(", ")));
    }
    if !detail.special_traits.is_empty() {
        desc.push_str(&format!("Traits: {}\n", detail.special_traits.join(", ")));
    }
    desc
}

///
pub fn monster_threat_level(
    monster_level: i32,
    player_level: i32,
    _monster_mr: i32,
) -> &'static str {
    let diff = monster_level - player_level;
    if diff >= 10 {
        "DEADLY"
    } else if diff >= 5 {
        "Very Dangerous"
    } else if diff >= 2 {
        "Dangerous"
    } else if diff >= 0 {
        "Challenging"
    } else if diff >= -3 {
        "Moderate"
    } else if diff >= -7 {
        "Easy"
    } else {
        "Trivial"
    }
}

///
pub fn item_detail_description(
    name: &str,
    item_class: &str,
    weight: i32,
    is_blessed: bool,
    is_cursed: bool,
    enchantment: i32,
) -> String {
    let mut desc = format!("{} [{}]", name, item_class);
    if is_blessed {
        desc.push_str(" (blessed)");
    }
    if is_cursed {
        desc.push_str(" (cursed)");
    }
    if enchantment != 0 {
        desc.push_str(&format!(" {:+}", enchantment));
    }
    desc.push_str(&format!(" | wt: {}", weight));
    desc
}

///
#[derive(Debug, Clone)]
pub enum LookAtResult {
    Monster(MonsterDetail),
    Item(String),
    Tile(String),
    Trap(String),
    Engraving(String),
    Nothing,
}

///
pub fn look_at_summary(result: &LookAtResult) -> String {
    match result {
        LookAtResult::Monster(d) => monster_detail_description(d),
        LookAtResult::Item(s) => format!("You see: {}", s),
        LookAtResult::Tile(s) => format!("Terrain: {}", s),
        LookAtResult::Trap(s) => format!("Trap: {}", s),
        LookAtResult::Engraving(t) => format!("Engraving: \"{}\"", t),
        LookAtResult::Nothing => "You see nothing special.".to_string(),
    }
}

///
pub struct EncyclopediaEntry {
    pub name: String,
    pub text: String,
}

///
pub fn encyclopedia_lookup(query: &str) -> Option<EncyclopediaEntry> {
    let entries = [
        (
            "dragon",
            "Dragons are powerful reptilian creatures that can breathe various elements.",
        ),
        (
            "lich",
            "Liches are undead spellcasters who have achieved immortality through dark magic.",
        ),
        (
            "mind flayer",
            "Mind flayers feed on brains and can cause intelligence drain.",
        ),
        (
            "cockatrice",
            "The cockatrice's touch can turn victims to stone.",
        ),
        ("medusa", "Medusa's gaze attack can turn victims to stone."),
        (
            "unicorn",
            "Offering a gem to a unicorn of your alignment increases your luck.",
        ),
        (
            "nymph",
            "Nymphs steal items from adventurers and teleport away.",
        ),
    ];
    let lower = query.to_lowercase();
    entries
        .iter()
        .find(|(name, _)| lower.contains(name))
        .map(|(name, text)| EncyclopediaEntry {
            name: name.to_string(),
            text: text.to_string(),
        })
}

///
#[derive(Debug, Clone, Default)]
pub struct PagerStatistics {
    pub monsters_examined: u32,
    pub items_examined: u32,
    pub tiles_examined: u32,
    pub encyclopedia_lookups: u32,
}

impl PagerStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_look(&mut self, result: &LookAtResult) {
        match result {
            LookAtResult::Monster(_) => self.monsters_examined += 1,
            LookAtResult::Item(_) => self.items_examined += 1,
            LookAtResult::Tile(_) => self.tiles_examined += 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod pager_extended_tests {
    use super::*;

    #[test]
    fn test_monster_detail() {
        let d = MonsterDetail {
            name: "Red Dragon".to_string(),
            symbol: 'D',
            level: 15,
            speed: 9,
            ac: -1,
            magic_resist: 20,
            attacks: vec!["breath fire 6d6".to_string()],
            resistances: vec!["fire".to_string()],
            special_traits: vec!["fly".to_string()],
        };
        let desc = monster_detail_description(&d);
        assert!(desc.contains("Red Dragon"));
    }

    #[test]
    fn test_threat() {
        assert_eq!(monster_threat_level(20, 5, 50), "DEADLY");
        assert_eq!(monster_threat_level(5, 15, 0), "Trivial");
    }

    #[test]
    fn test_item_desc() {
        let d = item_detail_description("long sword", "weapon", 40, true, false, 3);
        assert!(d.contains("+3"));
        assert!(d.contains("blessed"));
    }

    #[test]
    fn test_encyclopedia() {
        assert!(encyclopedia_lookup("dragon").is_some());
        assert!(encyclopedia_lookup("xyzzy").is_none());
    }

    #[test]
    fn test_look_at() {
        let r = LookAtResult::Trap("bear trap".to_string());
        assert!(look_at_summary(&r).contains("bear trap"));
    }

    #[test]
    fn test_pager_stats() {
        let mut s = PagerStatistics::new();
        s.record_look(&LookAtResult::Item("sword".to_string()));
        assert_eq!(s.items_examined, 1);
    }
}
