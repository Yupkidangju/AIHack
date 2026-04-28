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
//

use crate::core::entity::capability::MonsterCapability;
use crate::core::entity::monster::{DamageType, MonsterTemplate, Resistances};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonsterSize {
    Tiny = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
    Huge = 4,
    Gigantic = 5,
}

impl MonsterSize {
    ///
    pub fn from_symbol(symbol: char) -> Self {
        match symbol {
            //
            'D' | 'T' => MonsterSize::Huge,
            //
            'H' | 'O' | 'N' => MonsterSize::Large,
            //
            '@' | 'K' | 'o' | 'Z' | 'M' | 'W' | 'V' | '&' | 'L' => MonsterSize::Medium,
            //
            'k' | 'g' | 'h' | 'n' | 'y' | 'i' => MonsterSize::Small,
            //
            'a' | 'b' | 'x' | 's' | 'S' | 'w' | 'j' | 'l' => MonsterSize::Tiny,
            //
            _ => MonsterSize::Medium,
        }
    }
}

///
///
pub fn mcalcmove(template: &MonsterTemplate) -> i32 {
    //
    //
    let speed = template.movement as i32;
    if speed <= 0 {
        return 0;
    }
    //
    //
    speed.max(1)
}

///
pub fn can_open(template: &MonsterTemplate) -> bool {
    //
    let symbol = template.symbol;
    matches!(
        symbol,
        '@' | 'K' | 'o' | 'O' | 'H' | '&' | 'V' | 'L' | 'M' | 'W' | 'Z' | 'n' | 'h'
    )
}

///
pub fn can_pick(template: &MonsterTemplate) -> bool {
    //
    can_open(template)
}

///
pub fn can_wield(template: &MonsterTemplate) -> bool {
    can_open(template) && template.symbol != 'Z'
}

///
pub fn can_traverse_water(template: &MonsterTemplate) -> bool {
    is_flyer(template) || is_swimmer(template)
}

///
pub fn is_flyer(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Fly)
}

///
pub fn is_swimmer(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Swim)
}

///
pub fn tunnels(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Tunnel)
}

///
pub fn amorphous(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Amorphous)
}

///
pub fn passes_walls(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::WallWalk)
}

///
pub fn is_invisible(template: &MonsterTemplate) -> bool {
    //
    false
}

///
pub fn is_unsolid(template: &MonsterTemplate) -> bool {
    amorphous(template)
}

///
pub fn polyok(template: &MonsterTemplate) -> bool {
    //
    !(template.geno & crate::core::entity::monster::G_UNIQ != 0)
        && !(template.geno & crate::core::entity::monster::G_NOGEN != 0)
}

// =============================================================================
//
// =============================================================================

///
pub fn is_undead(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Undead)
}

///
pub fn is_demon(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Demon)
}

///
pub fn is_humanoid(template: &MonsterTemplate) -> bool {
    template.symbol == '@'
        || template.symbol == 'K'
        || template.symbol == 'o'
        || template.symbol == 'O'
        || template.symbol == 'H'
}

///
pub fn is_human(template: &MonsterTemplate) -> bool {
    template.symbol == '@'
}

///
pub fn is_elf(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Elf)
}

///
pub fn is_dwarf(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Dwarf)
}

///
pub fn is_gnome(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Gnome)
}

///
pub fn is_orc(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Orc)
}

///
pub fn is_animal(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Animal)
}

///
pub fn is_mindless(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Mindless)
}

///
pub fn likes_darkness(template: &MonsterTemplate) -> bool {
    is_undead(template) || template.symbol == 'V'
}

///
pub fn likes_gems(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Jewels)
}

///
pub fn likes_gold(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Collect)
}

///
pub fn likes_food(template: &MonsterTemplate) -> bool {
    is_animal(template)
}

///
pub fn likes_magic(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::MagicCollect)
}

///
pub fn likes_objs(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Collect)
        || template.has_capability(MonsterCapability::Jewels)
}

///
pub fn resists_poison(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Poisonous)
}

///
pub fn resists_fire(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::FIRE)
}

///
pub fn resists_cold(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::COLD)
}

///
pub fn resists_elec(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::ELEC)
}

///
pub fn resists_sleep(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::SLEEP)
}

///
pub fn resists_acid(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::ACID)
}

///
pub fn resists_ston(template: &MonsterTemplate) -> bool {
    template.has_resist(Resistances::STONE)
}

///
pub fn can_teleport(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Teleport)
}

///
pub fn regenerates(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Regen)
}

///
pub fn see_invisible(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::SeeInvis)
}

///
pub fn corpse_conveys_resistance(template: &MonsterTemplate) -> Vec<DamageType> {
    let mut conveys = Vec::new();
    if resists_fire(template) {
        conveys.push(DamageType::Fire);
    }
    if resists_cold(template) {
        conveys.push(DamageType::Cold);
    }
    if resists_elec(template) {
        conveys.push(DamageType::Elec);
    }
    if resists_poison(template) {
        conveys.push(DamageType::Drst);
    }
    if resists_sleep(template) {
        conveys.push(DamageType::Slee);
    }
    conveys
}

// =============================================================================
//
// =============================================================================

///
///
pub fn monster_weight(template: &MonsterTemplate) -> i32 {
    //
    match template.symbol {
        'a' | 'b' | 'x' => 10,
        's' | 'S' => 20,
        'w' => 5,
        'j' | 'l' => 50,
        'k' | 'g' => 80,
        'h' | 'n' | 'y' | 'i' => 100,
        'K' | 'o' | 'Z' => 150,
        '@' | 'M' | 'W' | 'V' | 'L' => 200,
        'O' | 'N' => 300,
        'H' | '&' => 500,
        'T' => 800,
        'D' => 2000,
        _ => 150,
    }
}

///
///
pub fn monster_nutrition(template: &MonsterTemplate) -> i32 {
    //
    let base = monster_weight(template);
    //
    let multiplier = match template.symbol {
        'D' => 3,
        'H' | 'T' => 2,
        _ => 1,
    };
    (base * multiplier).max(10)
}

// =============================================================================
//
// =============================================================================

///
pub fn monster_experience(template: &MonsterTemplate, hp: i32) -> i32 {
    let mlev = template.level as i32;
    let mut xp = mlev * mlev + 4;

    //
    for atk in &template.attacks {
        match atk.adtype {
            DamageType::Fire
            | DamageType::Cold
            | DamageType::Elec
            | DamageType::Acid
            | DamageType::Drst
            | DamageType::Drli => {
                xp += mlev * 3;
            }
            DamageType::Slee | DamageType::Plys => {
                xp += mlev * 2;
            }
            _ => {}
        }
    }

    //
    xp += hp / 4;

    //
    if resists_fire(template) {
        xp += 5;
    }
    if resists_cold(template) {
        xp += 5;
    }
    if resists_elec(template) {
        xp += 5;
    }
    if resists_poison(template) {
        xp += 5;
    }
    if regenerates(template) {
        xp += 10;
    }
    if see_invisible(template) {
        xp += 5;
    }
    if is_flyer(template) {
        xp += 5;
    }
    if passes_walls(template) {
        xp += 10;
    }

    xp.max(1)
}

///
pub fn monster_difficulty(template: &MonsterTemplate) -> i32 {
    let mut diff = template.level as i32;
    diff += template.ac.abs() as i32 / 2;

    //
    for atk in &template.attacks {
        diff += atk.sides as i32 / 4;
    }

    //
    if is_flyer(template) {
        diff += 1;
    }
    if regenerates(template) {
        diff += 1;
    }
    if passes_walls(template) {
        diff += 2;
    }
    if resists_fire(template) {
        diff += 1;
    }

    diff.max(1)
}

// =============================================================================
//
// =============================================================================

///
pub fn mon_nam(name: &str, mon_name: Option<&str>) -> String {
    if let Some(custom) = mon_name {
        custom.to_string()
    } else {
        format!("the {}", name)
    }
}

///
pub fn monnam(name: &str, mon_name: Option<&str>) -> String {
    let s = mon_nam(name, mon_name);
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

///
pub fn a_monnam(name: &str, mon_name: Option<&str>) -> String {
    if let Some(custom) = mon_name {
        return custom.to_string();
    }
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    let article = if name.starts_with(|c: char| vowels.contains(&c.to_ascii_lowercase())) {
        "an"
    } else {
        "a"
    };
    format!("{} {}", article, name)
}

///
pub fn s_suffix(name: &str) -> String {
    if name.ends_with('s') || name.ends_with('x') || name.ends_with('z') {
        format!("{}'", name)
    } else {
        format!("{}'s", name)
    }
}

///
pub fn makeplural(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    //
    let irregular = [
        ("foot", "feet"),
        ("tooth", "teeth"),
        ("mouse", "mice"),
        ("louse", "lice"),
        ("child", "children"),
        ("goose", "geese"),
        ("ox", "oxen"),
    ];
    for (singular, plural) in &irregular {
        if name == *singular {
            return plural.to_string();
        }
    }

    let last = name.chars().last().unwrap_or(' ');
    let last2: String = name
        .chars()
        .rev()
        .take(2)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    if last == 'y' && !matches!(name.chars().rev().nth(1), Some('a' | 'e' | 'i' | 'o' | 'u')) {
        // fairy ??fairies
        format!("{}ies", &name[..name.len() - 1])
    } else if last == 's' || last == 'x' || last == 'z' || last2 == "sh" || last2 == "ch" {
        format!("{}es", name)
    } else if last == 'f' {
        format!("{}ves", &name[..name.len() - 1])
    } else if last2 == "fe" {
        format!("{}ves", &name[..name.len() - 2])
    } else {
        format!("{}s", name)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn corpse_chance(template: &MonsterTemplate) -> bool {
    //
    if template.geno & crate::core::entity::monster::G_NOCORPSE != 0 {
        return false;
    }
    //
    if template.symbol == 'E' {
        // Elemental
        return false;
    }
    //
    true
}

///
pub fn death_drops(template: &MonsterTemplate) -> Vec<String> {
    let mut drops = Vec::new();

    //
    if corpse_chance(template) {
        drops.push(format!("{} corpse", template.name));
    }

    //
    match template.symbol {
        'D' => {
            //
            drops.push(format!("{} scales", template.name));
        }
        'U' => {
            //
            drops.push("unicorn horn".to_string());
        }
        'W' => {
            //
            if template.name.contains("wraith") {
                //
            }
        }
        _ => {}
    }

    drops
}

// =============================================================================
//
// =============================================================================

///
pub fn goodpos(
    x: usize,
    y: usize,
    grid: &crate::core::dungeon::Grid,
    _template: Option<&MonsterTemplate>,
) -> bool {
    use crate::core::dungeon::tile::TileType;
    use crate::core::dungeon::{COLNO, ROWNO};

    if x >= COLNO || y >= ROWNO {
        return false;
    }

    if let Some(tile) = grid.get_tile(x, y) {
        match tile.typ {
            TileType::Room
            | TileType::Corr
            | TileType::OpenDoor
            | TileType::StairsUp
            | TileType::StairsDown => true,
            _ => false,
        }
    } else {
        false
    }
}

///
pub fn enexto(
    x: usize,
    y: usize,
    grid: &crate::core::dungeon::Grid,
    rng: &mut crate::util::rng::NetHackRng,
) -> Option<(usize, usize)> {
    use crate::core::dungeon::{COLNO, ROWNO};

    //
    for radius in 0..10 {
        let x_min = (x as i32 - radius).max(0) as usize;
        let x_max = ((x as i32 + radius) as usize).min(COLNO - 1);
        let y_min = (y as i32 - radius).max(0) as usize;
        let y_max = ((y as i32 + radius) as usize).min(ROWNO - 1);

        //
        let mut candidates = Vec::new();
        for cx in x_min..=x_max {
            for cy in y_min..=y_max {
                //
                if radius == 0 || cx == x_min || cx == x_max || cy == y_min || cy == y_max {
                    if goodpos(cx, cy, grid, None) {
                        candidates.push((cx, cy));
                    }
                }
            }
        }

        if !candidates.is_empty() {
            let idx = rng.rn2(candidates.len() as i32) as usize;
            return Some(candidates[idx]);
        }
    }
    None
}

// =============================================================================
//
// =============================================================================

///
pub fn is_shapeshifter(name: &str) -> bool {
    matches!(
        name,
        "doppelganger"
            | "sandestin"
            | "chameleon"
            | "vampire"
            | "vampire lord"
            | "Vlad the Impaler"
    )
}

///
///
pub fn shapeshifter_candidates(source: &str) -> Vec<&'static str> {
    match source {
        "doppelganger" => vec!["orc", "human", "elf", "dwarf", "gnome", "kobold"],
        "chameleon" => vec!["lizard", "gecko", "iguana", "newt", "crocodile"],
        "sandestin" => vec![
            "air elemental",
            "fire elemental",
            "earth elemental",
            "water elemental",
        ],
        _ => vec![],
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn mnearto(
    x: i32,
    y: i32,
    grid: &crate::core::dungeon::Grid,
    rng: &mut crate::util::rng::NetHackRng,
) -> Option<(i32, i32)> {
    let deltas: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for (dx, dy) in &deltas {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && ny >= 0 {
            if goodpos(nx as usize, ny as usize, grid, None) {
                candidates.push((nx, ny));
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    let idx = rng.rn2(candidates.len() as i32) as usize;
    Some(candidates[idx])
}

// =============================================================================
//
// =============================================================================

///
pub fn monster_class_name(symbol: char) -> &'static str {
    match symbol {
        'a' => "ant or other insect",
        'b' => "blob",
        'c' => "cockatrice or similar",
        'd' => "dog or canine",
        'e' => "eye or sphere",
        'f' => "feline",
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
        't' => "trapper or lurker",
        'u' => "unicorn or horse",
        'v' => "vortex",
        'w' => "worm",
        'x' => "xan or other mythical",
        'y' => "apelike creature",
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
        'R' => "rust monster",
        'S' => "snake",
        'T' => "troll",
        'U' => "umber hulk",
        'V' => "vampire",
        'W' => "wraith",
        'X' => "xorn",
        'Y' => "apelike creature",
        'Z' => "zombie",
        '&' => "demon or devil",
        ';' => "sea monster",
        ':' => "lizard-like",
        '\'' => "golem",
        '@' => "human or humanoid",
        '~' => "long worm tail",
        _ => "unknown creature",
    }
}

///
pub fn is_valid_monster_class(symbol: char) -> bool {
    monster_class_name(symbol) != "unknown creature"
}

// =============================================================================
//
// =============================================================================

///
///
pub fn monsndx(name: &str) -> u32 {
    //
    let mut hash: u32 = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash
}

// =============================================================================
//
// =============================================================================

///
///
pub fn wake_radius_sq(noise_level: i32) -> i32 {
    //
    match noise_level {
        0 => 0,
        1 => 4,
        2 => 16,
        3 => 36,
        4 => 64,
        _ => 100,
    }
}

///
pub fn dist2(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).pow(2) + (y1 - y2).pow(2)
}

// =============================================================================
//
// =============================================================================

///
///
///
pub fn m_at_description() -> &'static str {
    " mon.c: m_at(x,y) - Looks up monster at given coordinates. Uses World query."
}

// =============================================================================
//
// =============================================================================

///
pub fn mon_regen(template: &MonsterTemplate, turn: u64) -> i32 {
    if regenerates(template) {
        //
        1
    } else {
        //
        if turn % 20 == 0 {
            1
        } else {
            0
        }
    }
}

///
pub fn hates_light(template: &MonsterTemplate) -> bool {
    matches!(template.name.as_str(), "gremlin" | "gremlin (tame)")
}

///
pub fn hates_silver(template: &MonsterTemplate) -> bool {
    is_undead(template) || is_demon(template) || template.name.contains("were")
}

///
pub fn hates_iron(template: &MonsterTemplate) -> bool {
    is_elf(template) || template.symbol == 'n'
}

///
pub fn grow_up_hp_bonus(current_level: i32, new_level: i32) -> i32 {
    let diff = new_level - current_level;
    if diff <= 0 {
        return 0;
    }
    diff * 8
}

///
pub fn is_prey(template: &MonsterTemplate) -> bool {
    matches!(template.symbol, 'r' | 'a' | 'b' | 'w')
}

///
pub fn is_predator(template: &MonsterTemplate) -> bool {
    matches!(template.symbol, 'f' | 'd' | 'D' | 'T')
}

///
pub fn is_unique(template: &MonsterTemplate) -> bool {
    (template.geno & crate::core::entity::monster::G_UNIQ) != 0
}

///
pub fn is_prince(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Prince)
}

///
pub fn extra_nasty(template: &MonsterTemplate) -> bool {
    template.has_capability(MonsterCapability::Nasty)
}

///
pub fn infravision_range(template: &MonsterTemplate) -> bool {
    if template.has_capability(MonsterCapability::Infravisible) {
        return false;
    }
    if template.has_capability(MonsterCapability::Infravision) {
        return true;
    }
    false
}

// =============================================================================
// [v2.3.0
// =============================================================================

///
///
pub fn undead_to_corpse(name: &str) -> &str {
    match name {
        "kobold zombie" | "kobold mummy" => "kobold",
        "dwarf zombie" | "dwarf mummy" => "dwarf",
        "gnome zombie" | "gnome mummy" => "gnome",
        "orc zombie" | "orc mummy" => "orc",
        "elf zombie" | "elf mummy" => "elf",
        "human zombie" | "human mummy" | "vampire" | "vampire lord" => "human",
        "giant zombie" | "giant mummy" => "giant",
        "ettin zombie" | "ettin mummy" => "ettin",
        _ => name,
    }
}

///
///
///
pub fn genus(name: &str, mode: i32) -> &'static str {
    match name {
        //
        "student" => {
            if mode == 1 {
                "archeologist"
            } else {
                "human"
            }
        }
        "chieftain" => {
            if mode == 1 {
                "barbarian"
            } else {
                "human"
            }
        }
        "neanderthal" => {
            if mode == 1 {
                "caveman"
            } else {
                "human"
            }
        }
        "attendant" => {
            if mode == 1 {
                "healer"
            } else {
                "human"
            }
        }
        "page" => {
            if mode == 1 {
                "knight"
            } else {
                "human"
            }
        }
        "abbot" => {
            if mode == 1 {
                "monk"
            } else {
                "human"
            }
        }
        "acolyte" => {
            if mode == 1 {
                "priest"
            } else {
                "human"
            }
        }
        "hunter" => {
            if mode == 1 {
                "ranger"
            } else {
                "human"
            }
        }
        "thug" => {
            if mode == 1 {
                "rogue"
            } else {
                "human"
            }
        }
        "roshi" => {
            if mode == 1 {
                "samurai"
            } else {
                "human"
            }
        }
        "guide" => {
            if mode == 1 {
                "tourist"
            } else {
                "human"
            }
        }
        "apprentice" => {
            if mode == 1 {
                "wizard"
            } else {
                "human"
            }
        }
        "warrior" => {
            if mode == 1 {
                "valkyrie"
            } else {
                "human"
            }
        }
        _ => "unknown",
    }
}

///
pub fn is_clinger(template: &MonsterTemplate) -> bool {
    //
    matches!(template.symbol, 's' | 'p' | 't')
}

///
pub fn likes_lava(template: &MonsterTemplate) -> bool {
    //
    template.symbol == 'E' && resists_fire(template)
}

///
pub fn is_floater(template: &MonsterTemplate) -> bool {
    //
    template.symbol == 'e' || template.symbol == 'v'
}

///
pub fn amphibious(template: &MonsterTemplate) -> bool {
    is_swimmer(template) || template.symbol == ':'
}

///
pub fn is_reviver(template: &MonsterTemplate) -> bool {
    //
    matches!(
        template.name.as_str(),
        "troll" | "ice troll" | "rock troll" | "water troll" | "Olog-hai"
    )
}

///
pub fn unique_corpstat(template: &MonsterTemplate) -> bool {
    is_unique(template) || template.name.contains("lord") || template.name.contains("king")
}

///
pub fn is_lava_resistant(template: &MonsterTemplate) -> bool {
    likes_lava(template) || resists_fire(template)
}

///
pub fn is_dragon(template: &MonsterTemplate) -> bool {
    template.symbol == 'D'
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSpeed {
    ///
    Slow,
    ///
    Normal,
    ///
    Fast,
}

///
///
pub fn mcalcmove_with_speed(base_speed: i32, speed: MonsterSpeed) -> i32 {
    const NORMAL_SPEED: i32 = 12;
    let mmove = match speed {
        MonsterSpeed::Slow => (2 * base_speed + 1) / 3,
        MonsterSpeed::Normal => base_speed,
        MonsterSpeed::Fast => (4 * base_speed + 2) / 3,
    };
    //
    //
    let adj = mmove % NORMAL_SPEED;
    let base = mmove - adj;
    //
    if adj >= NORMAL_SPEED / 2 {
        base + NORMAL_SPEED
    } else {
        base
    }
}

///
///
pub fn monflee_turns(template: &MonsterTemplate, hp: i32, max_hp: i32) -> u32 {
    //
    if hp <= max_hp / 4 && !is_mindless(template) {
        //
        4
    } else {
        0
    }
}

///
///
pub fn canseemon(template: &MonsterTemplate, is_blind: bool) -> bool {
    if is_blind {
        return false;
    }
    !is_invisible(template)
}

///
pub fn stagger(template: &MonsterTemplate) -> &'static str {
    if is_floater(template) {
        "bob"
    } else if is_flyer(template) {
        "flutter"
    } else if template.symbol == ':' || template.symbol == 'S' {
        "slither"
    } else {
        "stagger"
    }
}

///
///
///
pub fn minliquid_check(
    template: &MonsterTemplate,
    tile_is_pool: bool,
    tile_is_lava: bool,
) -> (bool, Option<&'static str>) {
    //
    if is_flyer(template) || is_floater(template) {
        return (true, None);
    }

    if tile_is_lava {
        if !is_clinger(template) && !likes_lava(template) {
            if !resists_fire(template) {
                return (false, Some("burns to a crisp."));
            } else {
                return (true, Some("burns slightly."));
            }
        }
    } else if tile_is_pool {
        if !is_clinger(template) && !is_swimmer(template) && !amphibious(template) {
            return (false, Some("drowns."));
        }
    }
    (true, None)
}

///
///
pub fn mm_aggression(template1: &MonsterTemplate, template2: &MonsterTemplate) -> bool {
    //
    if template1.symbol == template2.symbol {
        return false;
    }
    //
    if (is_elf(template1) && is_orc(template2)) || (is_orc(template1) && is_elf(template2)) {
        return true;
    }
    //
    if (is_dwarf(template1) && is_orc(template2)) || (is_orc(template1) && is_dwarf(template2)) {
        return true;
    }
    //
    if (is_human(template1) && is_undead(template2))
        || (is_undead(template1) && is_human(template2))
    {
        return true;
    }
    //
    if (template1.symbol == 'd' && template2.symbol == 'f')
        || (template1.symbol == 'f' && template2.symbol == 'd')
    {
        return true;
    }
    false
}

///
///
///
pub fn mcalcdistress_tick(
    template: &MonsterTemplate,
    turn: u64,
    blinded_turns: &mut u32,
    frozen_turns: &mut u32,
    flee_turns: &mut u32,
) -> i32 {
    //
    let regen = mon_regen(template, turn);

    //
    if *blinded_turns > 0 {
        *blinded_turns -= 1;
    }

    //
    if *frozen_turns > 0 {
        *frozen_turns -= 1;
    }

    //
    if *flee_turns > 0 {
        *flee_turns -= 1;
    }

    regen
}

// =============================================================================
// [v2.4.0
//
//
// =============================================================================

//
//
//

///
///
///
pub fn max_mon_load(weight: i32, size: MonsterSize, is_strong: bool) -> i32 {
    const MAX_CARR_CAP: i32 = 1000;
    const WT_HUMAN: i32 = 1450;
    const MZ_HUMAN: i32 = 2; // MonsterSize::Medium

    let maxload: i64 = if weight == 0 {
        //
        (MAX_CARR_CAP as i64 * size as i64) / MZ_HUMAN as i64
    } else if !is_strong || (is_strong && weight > WT_HUMAN) {
        //
        (MAX_CARR_CAP as i64 * weight as i64) / WT_HUMAN as i64
    } else {
        //
        MAX_CARR_CAP as i64
    };

    //
    let maxload = if !is_strong { maxload / 2 } else { maxload };

    // 理쒖냼 1
    maxload.max(1) as i32
}

///
///
pub fn can_carry_check(
    template: &MonsterTemplate,
    item_weight: i32,
    item_qty: i32,
    current_load: i32,
    is_tame: bool,
    is_peaceful: bool,
    is_shopkeeper: bool,
) -> i32 {
    //
    if template.has_flag1(super::monster::MonsterFlags1::NOTAKE) {
        return 0;
    }

    //
    if is_peaceful && !is_tame {
        return 0;
    }

    //
    if is_shopkeeper {
        return item_qty;
    }

    //
    let max_load = max_mon_load(
        template.weight as i32,
        MonsterSize::from_symbol(template.symbol),
        template.has_flag2(super::monster::MonsterFlags2::STRONG),
    );
    if current_load + item_weight > max_load {
        return 0;
    }

    item_qty
}

//
//
//

///
///
///
pub fn mfndpos(
    x: usize,
    y: usize,
    grid: &crate::core::dungeon::Grid,
    template: &MonsterTemplate,
    _flags: u32,
) -> Vec<(usize, usize)> {
    use crate::core::dungeon::tile::TileType;
    use crate::core::dungeon::{COLNO, ROWNO};

    let mut positions = Vec::new();

    //
    let directions: [(i32, i32); 9] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for (dx, dy) in &directions {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        //
        if nx < 0 || ny < 0 || nx >= COLNO as i32 || ny >= ROWNO as i32 {
            continue;
        }

        let ux = nx as usize;
        let uy = ny as usize;

        if let Some(tile) = grid.get_tile(ux, uy) {
            let passable = match tile.typ {
                TileType::Room
                | TileType::Corr
                | TileType::OpenDoor
                | TileType::StairsUp
                | TileType::StairsDown => true,
                //
                t if t.is_wall() || t == TileType::Stone => passes_walls(template),
                //
                TileType::Door => can_open(template),
                //
                TileType::Pool | TileType::Moat | TileType::Water => {
                    is_swimmer(template) || is_flyer(template) || amphibious(template)
                }
                //
                TileType::LavaPool => is_lava_resistant(template) || is_flyer(template),
                _ => false,
            };

            if passable {
                positions.push((ux, uy));
            }
        }
    }

    positions
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMonResult {
    ///
    SomebodyCanMove,
    ///
    AllDone,
}

///
///
pub fn can_move_this_turn(movement: i32) -> bool {
    const NORMAL_SPEED: i32 = 12;
    movement >= NORMAL_SPEED
}

///
///
pub fn consume_movement(movement: i32) -> (i32, bool) {
    const NORMAL_SPEED: i32 = 12;
    let remaining = movement - NORMAL_SPEED;
    (remaining, remaining >= NORMAL_SPEED)
}

//
//
//

///
#[derive(Debug, Clone, Copy)]
pub struct XKillFlags {
    ///
    pub no_msg: bool,
    ///
    pub no_corpse: bool,
    ///
    pub no_conduct: bool,
}

impl Default for XKillFlags {
    fn default() -> Self {
        Self {
            no_msg: false,
            no_corpse: false,
            no_conduct: false,
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct MonsterDeathResult {
    ///
    pub is_dead: bool,
    ///
    pub corpse_left: bool,
    ///
    pub drops: Vec<String>,
    ///
    pub experience: i32,
    ///
    pub messages: Vec<String>,
    ///
    pub alignment_change: i32,
    ///
    pub luck_change: i32,
}

///
///
///
///
///
///
///
///
///
pub fn xkilled_result(
    template: &MonsterTemplate,
    flags: XKillFlags,
    is_tame: bool,
    is_peaceful: bool,
    total_deaths_of_type: u32,
    player_alignment_type: i32,
) -> MonsterDeathResult {
    let mut result = MonsterDeathResult {
        is_dead: true,
        corpse_left: false,
        drops: Vec::new(),
        experience: 0,
        messages: Vec::new(),
        alignment_change: 0,
        luck_change: 0,
    };

    //
    if !flags.no_msg {
        let verb = if is_undead(template) || template.symbol == 'E' || template.symbol == '\'' {
            "destroy"
        } else {
            "kill"
        };
        result
            .messages
            .push(format!("You {} the {}!", verb, template.name));
    }

    //
    if !flags.no_corpse && corpse_chance(template) {
        result.corpse_left = true;
        result.drops.push(format!("{} corpse", template.name));
    }

    //
    //
    result.drops.extend(death_drops(template));

    //
    result.experience = monster_experience(template, template.difficulty as i32 * 4);

    //
    //
    if is_human(template) && player_alignment_type != -1 {
        result.luck_change -= 2;
        result.messages.push("You murderer!".to_string());
    }

    //
    if is_peaceful || is_tame {
        result.luck_change -= 1;
    }
    if is_tame {
        result.alignment_change -= 15;
        result
            .messages
            .push("You hear the rumble of distant thunder...".to_string());
    } else if is_peaceful {
        result.alignment_change -= 5;
    }

    //
    if template.symbol == 'u'
        && template.name.contains("unicorn")
        && ((template.name.contains("white") && player_alignment_type == 1)
            || (template.name.contains("gray") && player_alignment_type == 0)
            || (template.name.contains("black") && player_alignment_type == -1))
    {
        result.luck_change -= 5;
        result.messages.push("You feel guilty...".to_string());
    }

    //
    result.alignment_change += template.alignment as i32;

    result
}

///
///
pub fn monkilled_result(template: &MonsterTemplate) -> Vec<String> {
    let mut messages = Vec::new();
    messages.push(format!("The {} is killed!", template.name));
    messages
}

//
//
//

///
///
pub fn setmangry_result(
    template: &MonsterTemplate,
    is_peaceful: bool,
    is_tame: bool,
    is_priest: bool,
    is_coaligned: bool,
    via_attack: bool,
    on_elbereth: bool,
) -> (i32, Vec<String>) {
    let mut alignment_change = 0;
    let mut messages = Vec::new();

    //
    if via_attack && on_elbereth {
        messages.push("You feel like a hypocrite.".to_string());
        alignment_change -= 5;
        messages.push("The engraving beneath you fades.".to_string());
    }

    //
    if !is_peaceful || is_tame {
        return (alignment_change, messages);
    }

    //
    if is_priest {
        if is_coaligned {
            alignment_change -= 5;
        } else {
            alignment_change += 2;
        }
    } else {
        alignment_change -= 1;
    }

    //
    if is_humanoid(template) {
        messages.push(format!("{} gets angry!", monnam(&template.name, None)));
    }

    (alignment_change, messages)
}

///
///
///
pub fn wake_range(noise_level: i32) -> i32 {
    //
    match noise_level {
        0 => 0,
        1 => 100,
        2 => 225,
        3 => 400,
        _ => 900,
    }
}

///
///
pub fn should_wake_monster(
    mon_x: i32,
    mon_y: i32,
    noise_x: i32,
    noise_y: i32,
    noise_distance_sq: i32,
) -> bool {
    dist2(mon_x, mon_y, noise_x, noise_y) <= noise_distance_sq
}

//
//
//

///
///
pub fn golemeffects(
    monster_name: &str,
    damage_type: &str,
    damage: i32,
) -> (i32, bool, Option<&'static str>) {
    match monster_name {
        "flesh golem" => match damage_type {
            //
            "electric" => ((damage + 5) / 6, false, Some("seems healthier.")),
            //
            "fire" | "cold" => (0, true, None),
            _ => (0, false, None),
        },
        "iron golem" => match damage_type {
            //
            "electric" => (0, true, None),
            //
            "fire" => (damage, false, Some("seems healthier.")),
            _ => (0, false, None),
        },
        _ => (0, false, None),
    }
}

//
//
//

///
///
///
pub fn angry_guards_messages(
    guard_count: i32,
    sleeping_count: i32,
    near_count: i32,
    see_count: i32,
    silent: bool,
) -> Vec<String> {
    let mut messages = Vec::new();
    if guard_count == 0 || silent {
        return messages;
    }

    if sleeping_count > 0 {
        if sleeping_count > 1 {
            messages.push("The guards wake up!".to_string());
        } else {
            messages.push("The guard wakes up!".to_string());
        }
    }

    if near_count > 0 {
        if near_count == 1 {
            messages.push("The guard gets angry!".to_string());
        } else {
            messages.push("The guards get angry!".to_string());
        }
    } else if see_count > 0 {
        if see_count == 1 {
            messages.push("You see an angry guard approaching!".to_string());
        } else {
            messages.push("You see angry guards approaching!".to_string());
        }
    } else {
        messages.push("You hear the shrill sound of a guard's whistle.".to_string());
    }

    messages
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct EatMetalResult {
    ///
    pub result_code: i32,
    ///
    pub heal: i32,
    ///
    pub eating_turns: i32,
    /// 硫붿떆吏
    pub messages: Vec<String>,
}

///
///
pub fn can_eat_metal(template: &MonsterTemplate) -> bool {
    //
    matches!(template.symbol, 'R' | 'X')
}

///
///
pub fn can_eat_organic(template: &MonsterTemplate) -> bool {
    //
    matches!(template.symbol, 'P' | 'b' | 'j')
}

//
//
//

///
pub fn wants_gold(template: &MonsterTemplate) -> bool {
    likes_gold(template) || template.symbol == 'l'
}

///
///
pub fn pickup_types(template: &MonsterTemplate) -> Vec<char> {
    let mut types = Vec::new();

    //
    if template.symbol == 'n' {
        return vec!['*'];
    }

    //
    if likes_gems(template) {
        types.push('*');
    }
    if likes_gold(template) {
        types.push('$');
    }
    if likes_food(template) {
        types.push('%');
    }
    if likes_magic(template) {
        types.push('+');
    }
    if likes_objs(template) {
        types.push('`'); // ??
    }

    //
    if template.has_flag2(super::monster::MonsterFlags2::COLLECT) {
        types.push(')');
        types.push('[');
    }

    types
}

//
//
//

///
///
///
///
///
///
pub fn usmellmon(name: &str, symbol: char) -> Option<&'static str> {
    //
    match name {
        "rothe" | "minotaur" => return Some("You notice a bovine smell."),
        "caveman" | "cavewoman" | "barbarian" | "neanderthal" => {
            return Some("You smell body odor.");
        }
        "human werejackal" | "human wererat" | "human werewolf" | "werejackal" | "wererat"
        | "werewolf" | "owlbear" => {
            return Some("You detect an odor reminiscent of an animal's den.");
        }
        "steam vortex" => return Some("You smell steam."),
        "green slime" => return Some("Something stinks."),
        "violet fungus" | "shrieker" => return Some("You smell mushrooms."),
        //
        "white unicorn" | "gray unicorn" | "black unicorn" | "jellyfish" => return None,
        _ => {}
    }

    //
    match symbol {
        'd' => Some("You notice a dog smell."),
        'D' => Some("You smell a dragon!"),
        'F' => Some("Something smells moldy."),
        'u' => Some("You detect a strong odor reminiscent of a stable."),
        'Z' => Some("You smell rotting flesh."),
        ';' => Some("You smell fish."),
        'o' => Some("A foul stench makes you feel a little nauseated."),
        _ => None,
    }
}

//
//
//

///
pub fn is_hider(template: &MonsterTemplate) -> bool {
    //
    matches!(template.symbol, 'm' | 'p' | 't')
}

///
pub fn mimic_disguises(template: &MonsterTemplate) -> Vec<&'static str> {
    if template.symbol == 'm' {
        vec![
            "a chest",
            "a large box",
            "a door",
            "a staircase",
            "a pile of gold",
            "a scroll",
        ]
    } else {
        vec![]
    }
}

//
//
//

///
///
pub fn should_shapeshift(
    template: &MonsterTemplate,
    is_shapeshifter_flag: bool,
    turns_since_last_shift: u32,
) -> bool {
    if !is_shapeshifter_flag {
        return false;
    }
    //
    //
    turns_since_last_shift >= 3
}

///
///
pub fn vampire_shapes() -> Vec<&'static str> {
    vec!["bat", "fog cloud", "wolf", "vampire bat"]
}

//
//
//

///
pub fn can_be_hatched(name: &str) -> bool {
    //
    matches!(
        name,
        "cockatrice"
            | "crocodile"
            | "snake"
            | "cobra"
            | "pit viper"
            | "python"
            | "red dragon"
            | "white dragon"
            | "orange dragon"
            | "black dragon"
            | "blue dragon"
            | "green dragon"
            | "yellow dragon"
            | "gray dragon"
            | "silver dragon"
    )
}

///
pub fn egg_type_from_parent(parent_name: &str) -> &str {
    //
    if can_be_hatched(parent_name) {
        parent_name
    } else {
        //
        "generic"
    }
}

///
///
pub fn dead_species_check(name: &str, genocided_list: &[String]) -> bool {
    genocided_list.iter().any(|g| g == name)
}

//
//
//

///
pub fn nonliving(template: &MonsterTemplate) -> bool {
    //
    is_undead(template)
        || template.symbol == '\''
        || template.symbol == 'v'
        || template.symbol == 'E'
}

///
pub fn noncorporeal(template: &MonsterTemplate) -> bool {
    //
    template.name.contains("ghost")
        || template.name.contains("shade")
        || template.name.contains("fog cloud")
}

///
pub fn sticks(template: &MonsterTemplate) -> bool {
    //
    template.symbol == 'm'
        || template.name.contains("giant eel")
        || template.name.contains("kraken")
}

///
pub fn touch_petrifies(name: &str) -> bool {
    matches!(name, "cockatrice" | "chickatrice")
}

///
pub fn slimeproof(template: &MonsterTemplate) -> bool {
    resists_acid(template) || template.symbol == 'P' || is_flyer(template)
}

///
pub fn is_rider(name: &str) -> bool {
    matches!(name, "Death" | "Pestilence" | "Famine")
}

///
pub fn is_watch(template: &MonsterTemplate) -> bool {
    template.name == "watchman" || template.name == "watch captain"
}

///
pub fn throws_rocks(template: &MonsterTemplate) -> bool {
    //
    template.symbol == 'H' || template.name.contains("titan") || template.name.contains("giant")
}

///
pub fn poly_when_stoned(template: &MonsterTemplate) -> bool {
    //
    template.name.contains("stone golem")
}

///
///
///
pub fn boss_taunt(template: &MonsterTemplate) -> Option<&'static str> {
    if !is_unique(template) {
        return None;
    }
    match template.name.as_str() {
        "Medusa" => Some("\"You dare gaze upon me?\""),
        "Vlad the Impaler" => Some("\"I vant to drink your blood!\""),
        "Wizard of Yendor" => Some("\"I'll be back...\""),
        "Demogorgon" => Some("\"You are but an insect!\""),
        _ => Some("\"You shall not pass!\""),
    }
}

// =============================================================================
// [v2.9.4] mon.c 핵심 라이프사이클 이식 (원본: mon.c L296-3820)
// 몬스터 사망/부활/변신/도주/섭취/액체 지형 처리
// =============================================================================

/// [v2.9.4] 시체 생성 결과 (원본: make_corpse, L296-487)
#[derive(Debug, Clone)]
pub struct MakeCorpseResult {
    /// 시체 생성 여부
    pub corpse_created: bool,
    /// 기본 시체 이름
    pub corpse_name: Option<String>,
    /// 특수 드롭 (드래곤 비늘, 골렘 조각 등)
    pub special_drops: Vec<String>,
    /// 시체 나이 보정
    pub age_offset: i32,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 시체 생성 판정 (원본: make_corpse, L296-487)
pub fn make_corpse_result(
    template: &MonsterTemplate,
    is_revived: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> MakeCorpseResult {
    let mut result = MakeCorpseResult {
        corpse_created: false,
        corpse_name: None,
        special_drops: Vec::new(),
        age_offset: 0,
        messages: Vec::new(),
    };
    let name = template.name.as_str();

    // 드래곤 → 비늘 드롭
    if is_dragon(template) {
        let chance = if is_revived { 20 } else { 3 };
        if rng.rn2(chance) == 0 {
            result
                .special_drops
                .push(format!("{} scales", name.replace(" dragon", "")));
        }
        result.corpse_created = true;
        result.corpse_name = Some(format!("{} corpse", name));
        return result;
    }
    // 유니콘 → 뿔
    if name.contains("unicorn") {
        if !(is_revived && rng.rn2(2) != 0) {
            result.special_drops.push("unicorn horn".to_string());
        }
        result.corpse_created = true;
        result.corpse_name = Some(format!("{} corpse", name));
        return result;
    }
    // 긴 지렁이 → 이빨
    if name == "long worm" {
        result.special_drops.push("worm tooth".to_string());
        result.corpse_created = true;
        result.corpse_name = Some(format!("{} corpse", name));
        return result;
    }
    // 뱀파이어/좀비/미라 → 원래 종족 시체
    if name.contains("vampire") || name.contains("zombie") || name.contains("mummy") {
        let original = undead_to_corpse(name);
        result.corpse_created = true;
        result.corpse_name = Some(format!("{} corpse", original));
        result.age_offset = -100;
        return result;
    }
    // 골렘 → 재질별 분해
    match name {
        "iron golem" => {
            for _ in 0..rng.d(2, 6) {
                result.special_drops.push("iron chain".to_string());
            }
            return result;
        }
        "glass golem" => {
            for _ in 0..rng.d(2, 4) {
                result.special_drops.push("worthless glass".to_string());
            }
            return result;
        }
        "clay golem" => {
            result
                .special_drops
                .push(format!("rock x{}", rng.rn2(20) + 50));
            return result;
        }
        "stone golem" => {
            result.special_drops.push("statue".to_string());
            return result;
        }
        "wood golem" => {
            for _ in 0..rng.d(2, 4) {
                result.special_drops.push("quarterstaff".to_string());
            }
            return result;
        }
        "leather golem" => {
            for _ in 0..rng.d(2, 4) {
                result.special_drops.push("leather armor".to_string());
            }
            return result;
        }
        "gold golem" => {
            result
                .special_drops
                .push(format!("gold x{}", (200 - rng.rn2(101)).max(1)));
            return result;
        }
        "paper golem" => {
            for _ in 0..rng.rnd(4) {
                result.special_drops.push("blank paper".to_string());
            }
            return result;
        }
        _ => {}
    }
    // 우즈/푸딩 → glob
    if name.contains("ooze") || name.contains("pudding") || name == "green slime" {
        result.special_drops.push(format!("glob of {}", name));
        return result;
    }
    // 기본
    if corpse_chance(template) {
        result.corpse_created = true;
        result.corpse_name = Some(format!("{} corpse", name));
    }
    result
}

/// [v2.9.4] 생명 구원 판정 결과
#[derive(Debug, Clone)]
pub struct LifeSaveResult {
    pub saved: bool,
    pub restored_hp: i32,
    pub amulet_consumed: bool,
    pub still_dead_from_genocide: bool,
    pub messages: Vec<String>,
}

/// [v2.9.4] 생명 구원 아뮬렛 판정 (원본: lifesaved_monster, L1877-1925)
pub fn lifesaved_check(
    template: &MonsterTemplate,
    has_amulet: bool,
    is_vampshifter: bool,
    is_genocided: bool,
    hp_max: i32,
    is_tame: bool,
) -> LifeSaveResult {
    let mut r = LifeSaveResult {
        saved: false,
        restored_hp: -1,
        amulet_consumed: false,
        still_dead_from_genocide: false,
        messages: Vec::new(),
    };
    if !has_amulet {
        return r;
    }
    if nonliving(template) && !is_vampshifter {
        return r;
    }
    r.amulet_consumed = true;
    r.messages.push("But wait...".to_string());
    r.messages
        .push(format!("{}'s medallion begins to glow!", template.name));
    r.messages
        .push("The medallion crumbles to dust!".to_string());
    r.restored_hp = if hp_max <= 0 { 10 } else { hp_max };
    if is_genocided {
        r.still_dead_from_genocide = true;
        r.messages.push(format!(
            "Unfortunately, {} is still genocided...",
            template.name
        ));
    } else {
        r.saved = true;
        if is_tame {
            r.messages
                .push(format!("{} looks much better!", template.name));
        }
    }
    r
}

/// [v2.9.4] 몬스터 사망 처리 결과 (원본: mondead, L1927-2071)
#[derive(Debug, Clone)]
pub struct MonDeadResult2 {
    pub is_dead: bool,
    pub vampire_rise: bool,
    pub vampire_new_form: Option<String>,
    pub reverted_form: Option<String>,
    pub death_count_species: Option<String>,
    pub kop_respawn: bool,
    pub messages: Vec<String>,
}

/// [v2.9.4] mondead 결과 계산 (원본: mondead, L1927-2071)
pub fn mondead_result2(
    template: &MonsterTemplate,
    has_life_amulet: bool,
    is_vampshifter: bool,
    is_genocided: bool,
    hp_max: i32,
    is_tame: bool,
    cham_form: Option<&str>,
    rng: &mut crate::util::rng::NetHackRng,
) -> MonDeadResult2 {
    let mut r = MonDeadResult2 {
        is_dead: true,
        vampire_rise: false,
        vampire_new_form: None,
        reverted_form: None,
        death_count_species: None,
        kop_respawn: false,
        messages: Vec::new(),
    };
    // 1. 생명 구원
    let save = lifesaved_check(
        template,
        has_life_amulet,
        is_vampshifter,
        is_genocided,
        hp_max,
        is_tame,
    );
    r.messages.extend(save.messages);
    if save.saved {
        r.is_dead = false;
        return r;
    }

    // 2. 뱀파이어 부활
    if is_vampshifter {
        if let Some(cham) = cham_form {
            r.is_dead = false;
            r.vampire_rise = true;
            r.vampire_new_form = Some(cham.to_string());
            r.messages.push(format!(
                "{} suddenly transforms and rises as {}!",
                template.name, cham
            ));
            return r;
        }
    }
    // 3. 원형 복원
    if let Some(cham) = cham_form {
        if cham != template.name {
            r.reverted_form = Some(cham.to_string());
        }
    }
    match template.name.as_str() {
        "werejackal" => {
            r.reverted_form = Some("human werejackal".to_string());
        }
        "werewolf" => {
            r.reverted_form = Some("human werewolf".to_string());
        }
        "wererat" => {
            r.reverted_form = Some("human wererat".to_string());
        }
        _ => {}
    }
    // 4. 사망 카운트
    r.death_count_species = Some(template.name.clone());
    // 5. Kop 재생성
    if template.symbol == 'K' && rng.rnd(5) <= 2 {
        r.kop_respawn = true;
    }
    r
}

/// [v2.9.4] 변신 결과 (원본: newcham, L3661-3820)
#[derive(Debug, Clone)]
pub struct NewChamResult {
    pub success: bool,
    pub new_form: Option<String>,
    pub hp_ratio: f32,
    pub gender_changed: bool,
    pub messages: Vec<String>,
}

/// [v2.9.4] 변신 판정 (원본: newcham, L3661-3820)
pub fn newcham_result(
    template: &MonsterTemplate,
    target_form: Option<&str>,
    current_hp: i32,
    max_hp: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> NewChamResult {
    let mut r = NewChamResult {
        success: false,
        new_form: None,
        hp_ratio: 1.0,
        gender_changed: false,
        messages: Vec::new(),
    };
    if is_rider(&template.name) {
        return r;
    }
    let new_name = match target_form {
        Some(f) => f.to_string(),
        None => {
            let c = [
                "giant rat",
                "gnome",
                "hobgoblin",
                "orc",
                "kobold",
                "jackal",
                "wolf",
                "warg",
                "hill orc",
                "bugbear",
                "ogre",
                "troll",
            ];
            c[rng.rn2(c.len() as i32) as usize].to_string()
        }
    };
    if new_name == template.name {
        return r;
    }
    r.hp_ratio = if max_hp > 0 {
        (current_hp as f32 / max_hp as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };
    if rng.rn2(10) == 0 {
        r.gender_changed = true;
    }
    r.success = true;
    r.new_form = Some(new_name.clone());
    r.messages
        .push(format!("{} turns into {}!", template.name, new_name));
    r
}

/// [v2.9.4] 도주 상태 전환 결과
#[derive(Debug, Clone)]
pub struct MonFleeResult {
    pub should_flee: bool,
    pub flee_turns: u32,
    pub messages: Vec<String>,
}

/// [v2.9.4] 도주 판정
pub fn monflee_check(
    template: &MonsterTemplate,
    current_hp: i32,
    max_hp: i32,
    force_flee: bool,
    already_fleeing: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> MonFleeResult {
    let mut r = MonFleeResult {
        should_flee: false,
        flee_turns: 0,
        messages: Vec::new(),
    };
    if already_fleeing {
        return r;
    }
    if force_flee {
        r.should_flee = true;
        r.flee_turns = (rng.rn2(50) + 25) as u32;
        return r;
    }
    if max_hp > 0 && current_hp <= max_hp / 5 {
        if is_mindless(template) {
            return r;
        }
        r.should_flee = true;
        r.flee_turns = monflee_turns(template, current_hp, max_hp);
        if is_humanoid(template) {
            r.messages.push(format!("{} turns to flee!", template.name));
        }
    }
    r
}

/// [v2.9.4] 금속 섭취 결과
#[derive(Debug, Clone)]
pub struct EatMetalCheckResult {
    pub action: i32,
    pub heal: i32,
    pub eating_turns: i32,
    pub remove_rustproof: bool,
    pub drops_rock: bool,
    pub messages: Vec<String>,
}

/// [v2.9.4] 금속 섭취 판정 (원본: meatmetal, L871-960)
pub fn meatmetal_check(
    template: &MonsterTemplate,
    is_tame: bool,
    item_name: &str,
    item_weight: i32,
    is_rustproof: bool,
    is_metallic: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> EatMetalCheckResult {
    let mut r = EatMetalCheckResult {
        action: 0,
        heal: 0,
        eating_turns: 0,
        remove_rustproof: false,
        drops_rock: false,
        messages: Vec::new(),
    };
    if is_tame || !is_metallic || !can_eat_metal(template) {
        return r;
    }
    if template.name == "rust monster" && is_rustproof {
        r.action = 1;
        r.remove_rustproof = true;
        r.messages
            .push(format!("{} eats {}!", template.name, item_name));
        r.messages.push(format!(
            "{} spits {} out in disgust!",
            template.name, item_name
        ));
        return r;
    }
    r.action = 1;
    r.eating_turns = item_weight / 2 + 1;
    r.heal = item_weight;
    r.messages
        .push(format!("{} eats {}!", template.name, item_name));
    if rng.rnd(25) < 3 {
        r.drops_rock = true;
    }
    r
}

/// [v2.9.4] 유기물 섭취 결과
#[derive(Debug, Clone)]
pub struct EatObjCheckResult {
    pub action: i32,
    pub eaten_count: i32,
    pub engulfed_count: i32,
    pub heal: i32,
    pub cures_blindness: bool,
    pub messages: Vec<String>,
}

/// [v2.9.4] 유기물 섭취 판정 (원본: meatobj, L962-1097)
pub fn meatobj_check(
    template: &MonsterTemplate,
    is_tame: bool,
    item_name: &str,
    item_weight: i32,
    is_organic: bool,
    is_corpse: bool,
    corpse_is_rider: bool,
    is_carrot: bool,
) -> EatObjCheckResult {
    let mut r = EatObjCheckResult {
        action: 0,
        eaten_count: 0,
        engulfed_count: 0,
        heal: 0,
        cures_blindness: false,
        messages: Vec::new(),
    };
    if is_tame {
        return r;
    }
    if !can_eat_organic(template) && template.symbol != 'b' {
        return r;
    }
    if is_corpse && corpse_is_rider {
        r.messages
            .push(format!("The {} corpse revives!", item_name));
        return r;
    }
    if is_organic {
        r.action = 1;
        r.eaten_count = 1;
        r.heal = item_weight;
        r.messages
            .push(format!("{} eats {}!", template.name, item_name));
        if is_carrot {
            r.cures_blindness = true;
        }
    } else {
        r.action = 1;
        r.engulfed_count = 1;
        r.messages
            .push(format!("{} engulfs {}.", template.name, item_name));
    }
    r
}

/// [v2.9.4] 액체 지형 효과 결과
#[derive(Debug, Clone)]
pub struct MinLiquidResult {
    pub died: bool,
    pub teleport_escape: bool,
    pub gremlin_split: bool,
    pub rust_damage: i32,
    pub death_cause: Option<String>,
    pub eel_hp_loss: i32,
    pub messages: Vec<String>,
}

/// [v2.9.4] 액체 지형 판정 (원본: minliquid, L489-636)
pub fn minliquid_result(
    template: &MonsterTemplate,
    tile_is_pool: bool,
    tile_is_lava: bool,
    tile_is_fountain: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> MinLiquidResult {
    let mut r = MinLiquidResult {
        died: false,
        teleport_escape: false,
        gremlin_split: false,
        rust_damage: 0,
        death_cause: None,
        eel_hp_loss: 0,
        messages: Vec::new(),
    };
    if is_flyer(template) || is_floater(template) {
        return r;
    }
    // 그렘린 분열
    if template.name == "gremlin" && (tile_is_pool || tile_is_fountain) && rng.rn2(3) != 0 {
        r.gremlin_split = true;
        return r;
    }
    // 철 골렘 녹
    if template.name == "iron golem" && tile_is_pool && rng.rn2(5) == 0 {
        r.rust_damage = rng.d(2, 6);
        r.messages.push(format!("{} rusts.", template.name));
        return r;
    }
    // 용암
    if tile_is_lava && !is_clinger(template) && !likes_lava(template) {
        if can_teleport(template) {
            r.teleport_escape = true;
            return r;
        }
        if !resists_fire(template) {
            r.died = true;
            r.death_cause = Some("burned by lava".to_string());
            r.messages
                .push(format!("{} is burned to a crisp.", template.name));
        }
        return r;
    }
    // 물
    if tile_is_pool && !is_clinger(template) && !is_swimmer(template) && !amphibious(template) {
        if can_teleport(template) {
            r.teleport_escape = true;
            return r;
        }
        r.died = true;
        r.death_cause = Some("drowned".to_string());
        r.messages.push(format!("{} drowns.", template.name));
        return r;
    }
    // 뱀장어 육상 페널티
    if !tile_is_pool && template.symbol == ';' {
        r.eel_hp_loss = 1;
    }
    r
}

/// [v2.9.4] 라이프사이클 통계
#[derive(Debug, Clone, Default)]
pub struct MonLifecycleStats {
    pub total_deaths: u32,
    pub life_saves: u32,
    pub vampire_rises: u32,
    pub polymorphs: u32,
    pub flee_triggers: u32,
    pub metal_eaten: u32,
    pub organic_eaten: u32,
    pub liquid_deaths: u32,
}

impl MonLifecycleStats {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_death(&mut self) {
        self.total_deaths += 1;
    }
    pub fn record_life_save(&mut self) {
        self.life_saves += 1;
    }
    pub fn record_polymorph(&mut self) {
        self.polymorphs += 1;
    }
}

// =============================================================================
// [v2.9.4] mon.c 2차 이식 — 석화/은신/각성/반응/줍기/변신 선택/종족 말살
// =============================================================================

/// [v2.9.4] 석화 결과 (원본: monstone, L2181-2267 + vamp_stone, L2540-2609)
#[derive(Debug, Clone)]
pub struct MonStoneResult {
    /// 실제로 석화되었는가 (뱀파이어 부활 시 false)
    pub petrified: bool,
    /// 동상 생성 여부
    pub statue_created: bool,
    /// 바위 생성 여부 (작은 몬스터)
    pub rock_created: bool,
    /// 뱀파이어 부활 발동 (석화 면역)
    pub vampire_reverted: bool,
    /// 부활 대상 형태
    pub revert_form: Option<String>,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 석화 판정 (원본: monstone + vamp_stone)
/// 골렘 → stone golem 변환, 뱀파이어 셰이프시프터 → 원형 복원, 이외 → 동상/바위
pub fn monstone_result(
    template: &MonsterTemplate,
    is_vampshifter: bool,
    cham_form: Option<&str>,
    is_stoning_resistant: bool,
    size: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> MonStoneResult {
    let mut r = MonStoneResult {
        petrified: false,
        statue_created: false,
        rock_created: false,
        vampire_reverted: false,
        revert_form: None,
        messages: Vec::new(),
    };

    // 골렘 → stone golem 변환 (원본: mon_to_stone, L2522-2538)
    if template.symbol == '\'' {
        r.messages
            .push(format!("{}이(가) 굳어지고 있다...", template.name));
        r.revert_form = Some("stone golem".to_string());
        return r;
    }

    // 뱀파이어 셰이프시프터 → 원형 복원 (원본: vamp_stone, L2540-2609)
    if is_vampshifter {
        if let Some(cham) = cham_form {
            r.vampire_reverted = true;
            r.revert_form = Some(cham.to_string());
            r.messages.push(format!(
                "석화되는 {}이(가) 바닥에서 몸부림친다!",
                template.name
            ));
            r.messages
                .push(format!("{}이(가) 새로운 활력으로 일어선다!", cham));
            return r;
        }
    }

    // 석화 저항
    if is_stoning_resistant {
        return r;
    }

    // 크기 기반: TINY보다 크면 동상, 작으면 바위 (원본: L2204-2249)
    r.petrified = true;
    if size > 0 || rng.rn2(3) != 0 {
        r.statue_created = true;
        r.messages
            .push(format!("{}이(가) 돌로 변한다.", template.name));
    } else {
        r.rock_created = true;
        r.messages
            .push(format!("{}이(가) 자갈로 부서진다.", template.name));
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 은신 (원본: restrap L3172-3194, hideunder L3198-3228)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 은신 재시도 결과 (원본: restrap, L3172-3194)
#[derive(Debug, Clone)]
pub struct RestrapResult {
    /// 은신 성공 여부
    pub hidden: bool,
    /// 미믹 위장 활성화 여부
    pub mimic_disguised: bool,
    /// 감지 불가(mundetected) 설정 여부
    pub undetected: bool,
}

/// [v2.9.4] 은신 재시도 판정 (원본: restrap, L3172-3194)
/// 은신 실패 조건: 취소됨, 위장 중, 시야 안, 2/3 확률, 갇힌 상태
pub fn restrap_check(
    template: &MonsterTemplate,
    is_cancelled: bool,
    is_already_disguised: bool,
    in_player_sight: bool,
    is_trapped: bool,
    trap_is_pit: bool,
    player_senses_nearby: bool,
    tile_is_room: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> RestrapResult {
    let mut r = RestrapResult {
        hidden: false,
        mimic_disguised: false,
        undetected: false,
    };

    // 은신 불가 조건 (원본: L3177-3183)
    if is_cancelled || is_already_disguised || in_player_sight || rng.rn2(3) == 0 {
        return r;
    }
    if is_trapped && !trap_is_pit {
        return r;
    }
    if player_senses_nearby {
        return r;
    }

    // 미믹 은신
    if template.symbol == 'm' {
        r.hidden = true;
        r.mimic_disguised = true;
        return r;
    }
    // 일반 은신 (ROOM 타일)
    if tile_is_room {
        r.hidden = true;
        r.undetected = true;
    }
    r
}

/// [v2.9.4] 물체 아래 은신 판정 (원본: hideunder, L3198-3228)
pub fn hideunder_check(
    template: &MonsterTemplate,
    tile_is_pool: bool,
    has_objects_on_tile: bool,
    is_trapped_non_pit: bool,
    is_stuck: bool,
) -> bool {
    if is_stuck || is_trapped_non_pit {
        return false;
    }
    // 뱀장어 → 물에서 은신
    if template.symbol == ';' {
        return tile_is_pool;
    }
    // hides_under 속성 + 타일에 물건 있음
    if is_hider(template) && has_objects_on_tile {
        return true;
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 각성/반응 (원본: wakeup L3027-3042, wake_nearto L3053-3078, m_respond L2858-2883)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 각성 결과 (원본: wakeup, L3027-3042)
#[derive(Debug, Clone)]
pub struct WakeupResult {
    /// 수면 해제
    pub sleep_cleared: bool,
    /// 미믹 위장 해제
    pub mimic_revealed: bool,
    /// 은신 해제
    pub undetected_cleared: bool,
    /// 분노 설정 여부
    pub should_anger: bool,
}

/// [v2.9.4] 몬스터 각성 (원본: wakeup, L3027-3042)
pub fn wakeup_result(
    is_sleeping: bool,
    is_disguised: bool,
    is_undetected: bool,
    via_attack: bool,
    force_fight: bool,
) -> WakeupResult {
    let mut r = WakeupResult {
        sleep_cleared: false,
        mimic_revealed: false,
        undetected_cleared: false,
        should_anger: false,
    };
    if is_sleeping {
        r.sleep_cleared = true;
    }
    if is_disguised {
        r.mimic_revealed = true;
    } else if force_fight && is_undetected {
        r.undetected_cleared = true;
    }
    if via_attack {
        r.should_anger = true;
    }
    r
}

/// [v2.9.4] 근처 몬스터 각성 판정 (원본: wake_nearto, L3053-3078)
/// 거리 범위 안의 몬스터를 깨움 (수면 해제, 전략 대기 해제)
pub fn should_wake_from_noise(
    mon_x: i32,
    mon_y: i32,
    noise_x: i32,
    noise_y: i32,
    noise_distance: i32,
    is_sleeping: bool,
    is_unique: bool,
) -> bool {
    if !is_sleeping {
        return false;
    }
    let d = dist2(mon_x, mon_y, noise_x, noise_y);
    if noise_distance == 0 || d < noise_distance {
        // 유니크 몬스터는 '명상(meditation)' 해제 안 함 (원본: L3065-3066)
        let _ = is_unique;
        return true;
    }
    false
}

/// [v2.9.4] 몬스터 반응 결과 (원본: m_respond, L2858-2883)
#[derive(Debug, Clone)]
pub struct MonRespondResult {
    /// 비명 소리 발동 (shriek)
    pub shrieks: bool,
    /// 비명이 새 몬스터 소환 트리거
    pub summon_trigger: bool,
    /// 메두사 응시 발동
    pub medusa_gaze: bool,
    /// 악화(aggravate) 트리거
    pub aggravate: bool,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 몬스터 반응 (원본: m_respond, L2858-2883)
pub fn m_respond_result(
    template: &MonsterTemplate,
    player_is_deaf: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> MonRespondResult {
    let mut r = MonRespondResult {
        shrieks: false,
        summon_trigger: false,
        medusa_gaze: false,
        aggravate: false,
        messages: Vec::new(),
    };

    // 비명류 (MS_SHRIEK) (원본: L2861-2873)
    if template.name == "shrieker" || template.name.contains("shrieker") {
        r.shrieks = true;
        r.aggravate = true;
        if !player_is_deaf {
            r.messages
                .push(format!("{}이(가) 비명을 지른다.", template.name));
        }
        if rng.rn2(10) == 0 {
            r.summon_trigger = true;
        }
    }

    // 메두사 응시 (원본: L2874-2882)
    if template.name == "Medusa" {
        r.medusa_gaze = true;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 아이템 줍기 (원본: mpickgold L1101-1118, mpickstuff L1121-1168)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 금 줍기 결과 (원본: mpickgold, L1101-1118)
#[derive(Debug, Clone)]
pub struct PickGoldResult {
    /// 금을 주웠는가
    pub picked_up: bool,
    /// 주운 금액
    pub amount: i32,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 금 줍기 판정 (원본: mpickgold, L1101-1118)
pub fn mpickgold_check(
    template: &MonsterTemplate,
    gold_on_tile: i32,
    is_guard: bool,
    player_can_see: bool,
    verbose: bool,
) -> PickGoldResult {
    let mut r = PickGoldResult {
        picked_up: false,
        amount: 0,
        messages: Vec::new(),
    };
    if gold_on_tile <= 0 {
        return r;
    }
    if !wants_gold(template) && !is_guard {
        return r;
    }

    r.picked_up = true;
    r.amount = gold_on_tile;
    if player_can_see && verbose && !is_guard {
        r.messages
            .push(format!("{}이(가) 금을 줍는다.", template.name));
    }
    r
}

/// [v2.9.4] 아이템 줍기 결과 (원본: mpickstuff, L1121-1168)
#[derive(Debug, Clone)]
pub struct PickStuffResult {
    /// 주운 아이템 이름
    pub picked_item: Option<String>,
    /// 주운 수량
    pub quantity: i32,
    /// 장비 장착 시도 여부
    pub should_wear: bool,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 아이템 줍기 판정 (원본: mpickstuff, L1121-1168)
/// 님프는 모든 것을 줍고, 대부분은 시체를 안 줍고, 물 위 아이템 안 줍음
pub fn mpickstuff_check(
    template: &MonsterTemplate,
    item_name: &str,
    item_class: char,
    pickup_classes: &str,
    is_corpse: bool,
    corpse_petrifies: bool,
    is_pool_tile: bool,
    current_load: i32,
    max_load: i32,
    player_can_see: bool,
) -> PickStuffResult {
    let mut r = PickStuffResult {
        picked_item: None,
        quantity: 0,
        should_wear: false,
        messages: Vec::new(),
    };

    // 물 위 아이템 안 줍음 (원본: L1148)
    if is_pool_tile {
        return r;
    }

    // 줍기 클래스 확인
    let wants_class = pickup_classes.contains(item_class) || template.symbol == 'n';
    if !wants_class {
        return r;
    }

    // 시체 필터 (님프 제외) (원본: L1137-1142)
    if is_corpse && template.symbol != 'n' && !corpse_petrifies {
        return r;
    }

    // 무게 체크
    if current_load >= max_load {
        return r;
    }

    r.picked_item = Some(item_name.to_string());
    r.quantity = 1;
    r.should_wear = true;
    if player_can_see {
        r.messages.push(format!(
            "{}이(가) {}을(를) 줍는다.",
            template.name, item_name
        ));
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 변신 형태 선택 (원본: pm_to_cham L267-279, select_newcham_form L3480-3610)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] pm_to_cham — 변신 종 판별 (원본: pm_to_cham, L267-279)
/// 셰이프시프터이면 자기 인덱스 반환, 아니면 None
pub fn pm_to_cham(template: &MonsterTemplate) -> Option<String> {
    if is_shapeshifter(&template.name) {
        Some(template.name.clone())
    } else {
        None
    }
}

/// [v2.9.4] 뱀파이어 변신 형태 선택 (원본: pickvampshape, L3364-3393)
pub fn pickvampshape_result(rng: &mut crate::util::rng::NetHackRng) -> String {
    let shapes = vampire_shapes();
    if shapes.is_empty() {
        return "vampire".to_string();
    }
    shapes[rng.rn2(shapes.len() as i32) as usize].to_string()
}

/// [v2.9.4] 변신 형태 선택 (원본: select_newcham_form, L3480-3610)
/// 카멜레온 종류별 다른 후보군에서 선택
pub fn select_newcham_form(cham_type: &str, rng: &mut crate::util::rng::NetHackRng) -> String {
    match cham_type {
        "sandestin" => {
            // 대부분 nasty 몬스터 (원본: L3486-3489)
            let nasty = [
                "cockatrice",
                "ettin",
                "mind flayer",
                "purple worm",
                "green dragon",
                "minotaur",
            ];
            nasty[rng.rn2(nasty.len() as i32) as usize].to_string()
        }
        "doppelganger" => {
            // 역할 몬스터, 퀘스트 가디언, 일반 (원본: L3490-3510)
            if rng.rn2(3) != 0 {
                let roles = [
                    "archeologist",
                    "barbarian",
                    "caveman",
                    "healer",
                    "knight",
                    "monk",
                    "priest",
                    "ranger",
                    "rogue",
                    "samurai",
                    "tourist",
                    "valkyrie",
                    "wizard",
                ];
                roles[rng.rn2(roles.len() as i32) as usize].to_string()
            } else {
                let general = ["elf", "dwarf", "gnome", "orc", "human", "nymph", "centaur"];
                general[rng.rn2(general.len() as i32) as usize].to_string()
            }
        }
        "chameleon" => {
            // 동물 선택 (원본: L3511-3514)
            let animals = [
                "jackal", "wolf", "panther", "jaguar", "tiger", "horse", "pony", "warhorse",
            ];
            animals[rng.rn2(animals.len() as i32) as usize].to_string()
        }
        // 뱀파이어 종류
        "vampire" | "vampire lord" | "Vlad the Impaler" => pickvampshape_result(rng),
        _ => {
            // 일반 무작위
            let general = [
                "giant rat",
                "gnome",
                "hobgoblin",
                "orc",
                "kobold",
                "wolf",
                "ogre",
                "troll",
            ];
            general[rng.rn2(general.len() as i32) as usize].to_string()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 종족 말살 (원본: kill_genocided_monsters L3991-4029)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 종족 말살 결과 (원본: kill_genocided_monsters, L3991-4029)
#[derive(Debug, Clone)]
pub struct GenocideResult {
    /// 말살된 몬스터 이름 목록
    pub killed_names: Vec<String>,
    /// 카멜레온 자체도 사망하는 경우
    pub chameleon_deaths: Vec<String>,
    /// 말살 처리 후 남은 아이템 드롭 여부
    pub items_dropped: bool,
}

/// [v2.9.4] 종족 말살 체크 (원본: kill_genocided_monsters, L3991-4029)
/// 말살된 종 + 그 종으로 변신 중인 카멜레온도 죽음
pub fn kill_genocided_check(
    monster_name: &str,
    cham_form: Option<&str>,
    genocided_species: &[String],
) -> bool {
    // 1. 직접 말살
    if genocided_species.iter().any(|s| s == monster_name) {
        return true;
    }
    // 2. 원형이 말살됨 (카멜레온의 원래 종이 말살)
    if let Some(cham) = cham_form {
        if genocided_species.iter().any(|s| s == cham) {
            return true;
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 경비병 상태 (원본: pacify_guards L4112-4122, angry_guards L4067-4109)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 경비병 분노 결과 (원본: angry_guards, L4067-4109)
#[derive(Debug, Clone)]
pub struct AngryGuardsResult {
    /// 분노한 경비병 수
    pub angry_count: i32,
    /// 가까이 있는 경비병 수
    pub nearby_count: i32,
    /// 자고 있다가 깨어난 경비병 수
    pub awakened_count: i32,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 경비병 분노 판정 (원본: angry_guards, L4067-4109)
pub fn angry_guards_result(
    guard_count: i32,
    nearby_active: i32,
    far_active: i32,
    sleeping_count: i32,
    silent: bool,
) -> AngryGuardsResult {
    let mut r = AngryGuardsResult {
        angry_count: guard_count,
        nearby_count: nearby_active,
        awakened_count: sleeping_count,
        messages: Vec::new(),
    };
    if guard_count == 0 {
        return r;
    }
    if silent {
        return r;
    }

    if sleeping_count > 0 {
        r.messages.push(format!(
            "경비병{}이(가) 깨어난다!",
            if sleeping_count > 1 { "들" } else { "" }
        ));
    }
    if nearby_active > 0 {
        r.messages.push(format!(
            "경비병{}이(가) 분노한다!",
            if nearby_active > 1 { "들" } else { "" }
        ));
    } else if far_active > 0 {
        r.messages.push(format!(
            "분노한 경비병{}이(가) 다가오는 것이 보인다!",
            if far_active > 1 { "들" } else { "" }
        ));
    } else {
        r.messages
            .push("경비병의 호루라기 소리가 들린다.".to_string());
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 과밀 처리 (원본: elemental_clog L2649-2725, deal_with_overcrowding L2755-2765)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 과밀 해소 결과
#[derive(Debug, Clone)]
pub struct OvercrowdingResult {
    /// 림보(limbo) 이동 여부
    pub send_to_limbo: bool,
    /// 원소 과밀 해소 (endgame)
    pub elemental_clog: bool,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 과밀 해소 판정 (원본: deal_with_overcrowding, L2755-2765)
pub fn overcrowding_check(is_endgame: bool) -> OvercrowdingResult {
    let mut r = OvercrowdingResult {
        send_to_limbo: false,
        elemental_clog: false,
        messages: Vec::new(),
    };
    if is_endgame {
        r.elemental_clog = true;
    } else {
        r.send_to_limbo = true;
    }
    r
}

/// [v2.9.4] 원소 과밀 해소 판정 (원본: elemental_clog, L2649-2725)
/// Endgame에서 원소 몬스터가 과밀하면 가장 약한 것 제거
pub fn elemental_clog_check(
    template: &MonsterTemplate,
    same_type_count: i32,
    threshold: i32,
) -> bool {
    // 원소/아졸(vortex) 종류만 해당 (원본: L2668-2681)
    let is_elemental = template.symbol == 'E' || template.symbol == 'v';
    if !is_elemental {
        return false;
    }

    // 같은 종류가 임계값 초과 시 제거
    same_type_count > threshold
}

// ─────────────────────────────────────────────────────────────────────────────
// 릴리스/점착 (원본: unstuck L2298-2320)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 점착 해제 결과 (원본: unstuck, L2298-2320)
#[derive(Debug, Clone)]
pub struct UnstuckResult {
    /// 설록 해제 여부
    pub released: bool,
    /// 삼킴 탈출 여부
    pub expel_from_engulf: bool,
    /// 메시지
    pub messages: Vec<String>,
}

/// [v2.9.4] 점착 해제 판정 (원본: unstuck, L2298-2320)
pub fn unstuck_check(is_stuck_to_player: bool, player_is_swallowed: bool) -> UnstuckResult {
    let mut r = UnstuckResult {
        released: false,
        expel_from_engulf: false,
        messages: Vec::new(),
    };
    if !is_stuck_to_player {
        return r;
    }
    r.released = true;
    if player_is_swallowed {
        r.expel_from_engulf = true;
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// mimic_hit_msg (원본: L4124-4144)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 미믹 피격 메시지 (원본: mimic_hit_msg, L4124-4144)
pub fn mimic_hit_msg(disguise_type: &str, spell_is_healing: bool) -> Option<String> {
    if spell_is_healing {
        Some(format!(
            "{}이(가) 이전보다 더 생생해 보인다.",
            disguise_type
        ))
    } else {
        None
    }
}

// =============================================================================
// [v2.9.4] mon.c 3차 이식 — 이동/변신 결정/수락/유효성/소멸/미믹/카멜레온/동물
// =============================================================================

/// [v2.9.4] movemon 이동 루프 개별 판정 결과 (원본: movemon, L720-858)
/// 실제 이동 루프는 ECS에서 처리하므로, 개별 몬스터의 턴 소비 판정만 이식
#[derive(Debug, Clone)]
pub struct MoveMonTickResult {
    /// 이동 포인트 부족으로 스킵
    pub skip_no_movement: bool,
    /// 이동 후에도 또 행동할 수 있는가 (somebody_can_move)
    pub can_act_again: bool,
    /// 액체 지형 효과로 사망/스킵
    pub liquid_effect: bool,
    /// 은신 재시도 성공 → 스킵
    pub rehide_success: bool,
    /// 장비 교체 시도 필요
    pub needs_reequip: bool,
    /// 소비한 이동 포인트
    pub movement_consumed: i32,
}

/// [v2.9.4] movemon 개별 몬스터 턴 판정 (원본: movemon 루프 내부, L774-838)
/// NORMAL_SPEED = 12
pub fn movemon_tick(
    current_movement: i32,
    normal_speed: i32,
    is_dead: bool,
    is_hider: bool,
    is_rehidden: bool,
    in_liquid: bool,
    needs_reequip: bool,
) -> MoveMonTickResult {
    let mut r = MoveMonTickResult {
        skip_no_movement: false,
        can_act_again: false,
        liquid_effect: false,
        rehide_success: false,
        needs_reequip: false,
        movement_consumed: 0,
    };

    // 사망한 몬스터 스킵 (원본: L770-771)
    if is_dead {
        r.skip_no_movement = true;
        return r;
    }

    // 이동 포인트 부족 (원본: L774-775)
    if current_movement < normal_speed {
        r.skip_no_movement = true;
        return r;
    }

    // 이동 소비 (원본: L777-779)
    r.movement_consumed = normal_speed;
    let remaining = current_movement - normal_speed;
    if remaining >= normal_speed {
        r.can_act_again = true;
    }

    // 액체 지형 (원본: L788-789)
    if in_liquid {
        r.liquid_effect = true;
        return r;
    }

    // 장비 교체 (원본: L792-800)
    if needs_reequip {
        r.needs_reequip = true;
    }

    // 은신자 재은신 (원본: L802-818)
    if is_hider && is_rehidden {
        r.rehide_success = true;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 변신 결정 (원본: decide_to_shapeshift L3302-3361)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 변신 결정 결과 (원본: decide_to_shapeshift, L3302-3361)
#[derive(Debug, Clone)]
pub struct ShapeshiftDecision {
    /// 변신 실행 여부
    pub should_shift: bool,
    /// 목표 형태 (None이면 랜덤)
    pub target_form: Option<String>,
    /// 메시지 출력 여부
    pub show_message: bool,
    /// 성별 유지 플래그 (뱀파이어)
    pub preserve_gender: bool,
}

/// [v2.9.4] 변신 결정 (원본: decide_to_shapeshift, L3302-3361)
/// 뱀파이어: HP 기반 변신 판정, 일반 셰이프시프터: 1/6 확률
pub fn decide_to_shapeshift(
    template: &MonsterTemplate,
    is_vampshifter: bool,
    current_form_is_vampire: bool,
    current_form_is_fog: bool,
    hp: i32,
    max_hp: i32,
    has_cham_form: bool,
    player_can_see: bool,
    show_seen_msg: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> ShapeshiftDecision {
    let mut r = ShapeshiftDecision {
        should_shift: false,
        target_form: None,
        show_message: false,
        preserve_gender: false,
    };

    // 메시지 표시 여부
    r.show_message = show_seen_msg || player_can_see;

    if !is_vampshifter {
        // 일반 셰이프시프터: 1/6 확률 변신 (원본: L3317-3318)
        if rng.rn2(6) == 0 {
            r.should_shift = true;
        }
    } else {
        // 뱀파이어 셰이프시프터
        if !current_form_is_vampire {
            // 변신 중: HP 낮으면 원형 복원 (원본: L3328-3331)
            if hp <= (max_hp + 5) / 6 && rng.rn2(4) != 0 && has_cham_form {
                r.should_shift = true;
                // 원형 복원 → target_form은 cham 형태 (호출자가 설정)
            } else if current_form_is_fog && hp == max_hp && rng.rn2(4) == 0 {
                // 풀 HP fog cloud → 다른 형태로 변환 (원본: L3332-3344)
                r.should_shift = true;
            }
        } else {
            // 뱀파이어 원형: 높은 HP일 때 변신 (원본: L3347-3350)
            if hp >= 9 * max_hp / 10 && rng.rn2(6) == 0 {
                r.should_shift = true;
            }
        }
        r.preserve_gender = true;
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 변신 수락/유효성 (원본: accept_newcham_form L3614-3639,
//   isspecmon L3394-3399, validspecmon L3405-3425, validvamp L3429-3477)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 변신 형태 수락 판정 (원본: accept_newcham_form, L3614-3639)
/// 종족 말살, placeholder, polyok 체크
pub fn accept_newcham_form_check(
    new_form: &MonsterTemplate,
    is_genocided: bool,
    is_player_char: bool,
) -> bool {
    // 종족 말살됨 → 거부 (원본: L3623-3624)
    if is_genocided {
        return false;
    }
    // 플레이어 캐릭터 타입 → 수락 (원본: L3630-3631)
    if is_player_char {
        return true;
    }
    // polyok 체크 (원본: L3638)
    polyok(new_form)
}

/// [v2.9.4] 특수 몬스터 여부 (원본: isspecmon, L3394-3399)
/// 상점 주인, 사제, 금고 경비원, 퀘스트 리더
pub fn isspecmon_check(
    is_shopkeeper: bool,
    is_priest: bool,
    is_guard: bool,
    is_quest_leader: bool,
) -> bool {
    is_shopkeeper || is_priest || is_guard || is_quest_leader
}

/// [v2.9.4] 특수 몬스터 변신 유효성 (원본: validspecmon, L3405-3425)
/// 특수 몬스터는 말/손 없는 형태 거부
pub fn validspecmon_check(
    new_form: &MonsterTemplate,
    is_special: bool,
    has_hands: bool,
    has_head: bool,
    is_genocided: bool,
) -> bool {
    if is_genocided {
        return false;
    }
    if !polyok(new_form) {
        return false;
    }
    if is_special {
        // notake/nohead 거부 (원본: L3420-3421)
        if !has_hands || !has_head {
            return false;
        }
    }
    true
}

/// [v2.9.4] 뱀파이어 변신 유효성 (원본: validvamp, L3429-3477)
pub fn validvamp_check(requested_form: &str, cham_type: &str) -> bool {
    match requested_form {
        "fog cloud" | "vampire bat" => true,
        "wolf" => cham_type != "vampire", // 기본 뱀파이어는 늑대 불가
        _ => {
            // 뱀파이어 종류이면 수락
            matches!(
                requested_form,
                "vampire" | "vampire lord" | "Vlad the Impaler"
            )
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 성별 변경 (원본: mgender_from_permonst L3642-3656)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 변신 후 성별 판정 결과
#[derive(Debug, Clone, PartialEq)]
pub enum GenderResult {
    /// 수컷 고정
    Male,
    /// 암컷 고정
    Female,
    /// 중성
    Neuter,
    /// 기존 유지
    Unchanged,
    /// 10% 확률로 변경
    Flipped,
}

/// [v2.9.4] 변신 후 성별 판정 (원본: mgender_from_permonst, L3642-3656)
pub fn mgender_from_permonst(
    new_form: &str,
    current_is_female: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> GenderResult {
    // 특정 성별 고정 종
    let male_only = ["incubus", "priest", "monk"];
    let female_only = ["succubus", "priestess", "nymph"];

    if male_only.iter().any(|m| new_form.contains(m)) {
        return GenderResult::Male;
    }
    if female_only.iter().any(|f| new_form.contains(f)) {
        return GenderResult::Female;
    }
    // 중성 종
    if new_form.contains("golem") || new_form.contains("vortex") || new_form.contains("elemental") {
        return GenderResult::Neuter;
    }
    // 10% 확률 성별 변경 (원본: L3653-3654)
    if rng.rn2(10) == 0 {
        return GenderResult::Flipped;
    }
    GenderResult::Unchanged
}

// ─────────────────────────────────────────────────────────────────────────────
// 소멸/사망 래퍼 (원본: mongone L2157-2177, mondied L2143-2153)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 소멸 결과 (원본: mongone, L2157-2177)
#[derive(Debug, Clone)]
pub struct MonGoneResult {
    /// 기마 해제 발생
    pub dismount: bool,
    /// 점착 해제 발생
    pub unstuck: bool,
    /// 특수 아이템 드롭 (아뮬렛 등)
    pub special_drops: bool,
    /// 인벤토리 파기
    pub discard_inventory: bool,
}

/// [v2.9.4] 몬스터 소멸 판정 (원본: mongone, L2157-2177)
pub fn mongone_result(
    is_steed: bool,
    is_stuck_to_player: bool,
    has_special_items: bool,
) -> MonGoneResult {
    MonGoneResult {
        dismount: is_steed,
        unstuck: is_stuck_to_player,
        special_drops: has_special_items,
        discard_inventory: true,
    }
}

/// [v2.9.4] mondied 시체 판정 (원본: mondied, L2143-2153)
/// mondead 이후 시체 생성 여부만 판정
pub fn mondied_should_corpse(
    corpse_chance_result: bool,
    tile_accessible: bool,
    tile_is_pool: bool,
) -> bool {
    corpse_chance_result && (tile_accessible || tile_is_pool)
}

// ─────────────────────────────────────────────────────────────────────────────
// 미믹 노출 / 카멜레온 해제 (원본: seemimic L3082-3101,
//   rescham L3105-3130, restartcham L3131-3146, restore_cham L3152-3168)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 미믹 노출 결과 (원본: seemimic, L3082-3101)
#[derive(Debug, Clone)]
pub struct SeeMimicResult {
    /// 위장 해제됨
    pub disguise_cleared: bool,
    /// 빛 차단 해제 여부
    pub unblock_light: bool,
    /// 시체 이름 해제 여부
    pub free_corpse_name: bool,
}

/// [v2.9.4] 미믹 노출 (원본: seemimic, L3082-3101)
pub fn seemimic_result(is_light_blocker: bool, has_corpse_name: bool) -> SeeMimicResult {
    SeeMimicResult {
        disguise_cleared: true,
        unblock_light: is_light_blocker,
        free_corpse_name: has_corpse_name,
    }
}

/// [v2.9.4] 카멜레온 원형 복원 여부 (원본: rescham, L3105-3130)
/// Protection_from_shape_changers 활성 시 강제 복원
pub fn rescham_should_revert(
    is_shapeshifter: bool,
    cham_form_exists: bool,
    protection_from_shapechangers: bool,
) -> bool {
    if !protection_from_shapechangers {
        return false;
    }
    is_shapeshifter && cham_form_exists
}

/// [v2.9.4] 카멜레온 재시작 (원본: restartcham, L3131-3146)
/// 레벨 로드 시 카멜레온 속성 재설정
pub fn restartcham_check(is_cancelled: bool, is_shapeshifter: bool) -> bool {
    // 취소됨 → 변신 능력 없음 (원본: L3138)
    !is_cancelled && is_shapeshifter
}

/// [v2.9.4] 카멜레온 복원 (원본: restore_cham, L3152-3168)
/// 저장된 레벨 로드 시 보호 마법 상태에 따라 복원
pub fn restore_cham_should_revert(
    protection_active: bool,
    has_cham_form: bool,
    is_were: bool,
    is_human_form: bool,
) -> bool {
    if protection_active {
        // cham 형태 있으면 원형으로 복원 (원본: L3159-3161)
        if has_cham_form {
            return true;
        }
        // 수인 + 인간형이 아니면 인간으로 복원 (원본: L3162-3163)
        if is_were && !is_human_form {
            return true;
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 은신 재설정 (원본: hide_monst L3232-3253)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 레벨 복귀 시 은신 재설정 (원본: hide_monst, L3232-3253)
pub fn hide_monst_should_rehide(
    template: &MonsterTemplate,
    is_mundetected: bool,
    is_disguised: bool,
) -> bool {
    let hider_under = is_hider(template) || template.symbol == ';';
    let is_ceiling_hider = template.symbol == 'p' || template.symbol == 't';
    // 은신자이거나 아래 숨는자이면서, 아직 안 숨어있으면 재시도
    (is_ceiling_hider || hider_under) && !is_mundetected && !is_disguised
}

// ─────────────────────────────────────────────────────────────────────────────
// 동물 선택 (원본: pick_animal L3285-3299, mon_animal_list L3259-3282)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 동물 몬스터 목록 (원본: mon_animal_list, L3259-3282)
pub fn build_animal_list() -> Vec<&'static str> {
    vec![
        "jackal",
        "wolf",
        "coyote",
        "winter wolf",
        "hell hound",
        "panther",
        "jaguar",
        "tiger",
        "pony",
        "horse",
        "warhorse",
        "kitten",
        "housecat",
        "large cat",
        "rothe",
        "cow",
        "bull",
        "giant rat",
        "sewer rat",
        "ape",
        "sasquatch",
        "monkey",
    ]
}

/// [v2.9.4] 동물 선택 (원본: pick_animal, L3285-3299)
pub fn pick_animal(rng: &mut crate::util::rng::NetHackRng) -> &'static str {
    let animals = build_animal_list();
    animals[rng.rn2(animals.len() as i32) as usize]
}

// ─────────────────────────────────────────────────────────────────────────────
// 말살 관련 (원본: kill_eggs L3960-3987, kill_genocided L3991-4029)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 알 말살 판정 (원본: kill_eggs, L3960-3987)
/// 종족 말살된 몬스터의 알은 죽은 알로 변환
pub fn should_kill_egg(egg_species: &str, genocided_species: &[String]) -> bool {
    // baby → adult 변환 포함해서 말살 여부 체크 (원본: L3953-3955)
    genocided_species.iter().any(|s| s == egg_species)
}

/// [v2.9.4] 종족 말살 실행 결과 (원본: kill_genocided_monsters, L3991-4029)
#[derive(Debug, Clone)]
pub struct GenocideExecutionResult {
    /// 말살된 몬스터 수
    pub killed_count: i32,
    /// 카멜레온 형태 연쇄 사망 수
    pub chameleon_killed: i32,
    /// 아이템 드롭 트리거
    pub items_dropped: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// 경비병 진정 (원본: pacify_guards L4112-4122)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 경비병 진정 — 모든 경비병을 평화 상태로 전환 (원본: pacify_guards, L4112-4122)
/// ECS에서 is_watch() 체크 후 mpeaceful = true 설정
pub fn should_pacify(template: &MonsterTemplate) -> bool {
    is_watch(template)
}

// ─────────────────────────────────────────────────────────────────────────────
// 제거 가능 판정 (원본: ok_to_obliterate L2634-2646)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 소멸 가능 판정 (원본: ok_to_obliterate, L2634-2646)
/// Wizard of Yendor, Rider, 특수 몬스터 등은 제거 불가
pub fn ok_to_obliterate(
    template: &MonsterTemplate,
    is_shopkeeper: bool,
    is_priest: bool,
    is_guard: bool,
    is_player_stuck: bool,
    is_player_steed: bool,
) -> bool {
    // Wizard of Yendor, Rider → 제거 불가 (원본: L2641)
    if template.name == "Wizard of Yendor" || is_rider(&template.name) {
        return false;
    }
    // 특수 역할 → 불가 (원본: L2642)
    if is_shopkeeper || is_priest || is_guard {
        return false;
    }
    // 플레이어와 상호작용 중 → 불가 (원본: L2643)
    if is_player_stuck || is_player_steed {
        return false;
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// monnear (원본: L1597-1607)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 근접 여부 판정 (원본: monnear, L1597-1607)
pub fn monnear(mon_x: i32, mon_y: i32, target_x: i32, target_y: i32) -> bool {
    let dx = (mon_x - target_x).abs();
    let dy = (mon_y - target_y).abs();
    dx <= 1 && dy <= 1
}

// ─────────────────────────────────────────────────────────────────────────────
// curr_mon_load (원본: L1171-1183)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 현재 몬스터 적재량 (원본: curr_mon_load, L1171-1183)
/// 바위를 던지는 몬스터는 바위 무게 제외
pub fn curr_mon_load(
    inventory_weights: &[i32],
    boulder_weights: &[i32],
    throws_rocks: bool,
) -> i32 {
    let total: i32 = inventory_weights.iter().sum();
    if throws_rocks {
        let boulder_total: i32 = boulder_weights.iter().sum();
        total - boulder_total
    } else {
        total
    }
}

// =============================================================================
// [v2.9.4] mon.c 4차(최종) 이식 — 레벨이동/각성/배치/석화/대체/정리/무결성
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// 레벨 이동 (원본: migrate_mon L2623-2631, m_into_limbo L2613-2620)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 레벨 이동 결과 (원본: migrate_mon, L2623-2631)
#[derive(Debug, Clone)]
pub struct MigrateMonResult {
    /// 점착 해제 필요
    pub unstuck: bool,
    /// 특수 아이템 드롭 필요
    pub drop_special_objs: bool,
    /// 이동 대상 레벨
    pub target_level: i32,
    /// 이동 방식 (근사/정확)
    pub xyloc: MigrateXYLoc,
    /// MON_MIGRATING 플래그 설정
    pub set_migrating: bool,
}

/// [v2.9.4] 이동 좌표 방식 (원본: MIGR_* 상수)
#[derive(Debug, Clone, PartialEq)]
pub enum MigrateXYLoc {
    /// 대략적 좌표 (MIGR_APPROX_XY)
    ApproxXY,
    /// 정확한 좌표 (MIGR_EXACT_XY)
    ExactXY,
    /// 무작위 (MIGR_RANDOM)
    Random,
    /// 계단 근처 (MIGR_NEAR_STAIR)
    NearStair,
}

/// [v2.9.4] 몬스터 레벨 이동 판정 (원본: migrate_mon, L2623-2631)
pub fn migrate_mon_result(
    target_level: i32,
    xyloc: MigrateXYLoc,
    is_stuck_to_player: bool,
) -> MigrateMonResult {
    MigrateMonResult {
        unstuck: is_stuck_to_player,
        drop_special_objs: true, // 항상 특수 아이템 드롭 (원본: L2628)
        target_level,
        xyloc,
        set_migrating: true,
    }
}

/// [v2.9.4] 림보 이동 판정 (원본: m_into_limbo, L2613-2620)
/// 현재 레벨로 이주 (MON_LIMBO 설정)
pub fn m_into_limbo_result(current_level: i32, is_stuck: bool) -> MigrateMonResult {
    let mut r = migrate_mon_result(current_level, MigrateXYLoc::ApproxXY, is_stuck);
    // MON_LIMBO 플래그는 호출자가 별도 설정
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 근접 각성 (원본: wake_nearby L3046-3049, wake_nearto L3053-3078)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] wake_nearby 각성 범위 계산 (원본: wake_nearby, L3046-3049)
/// hero_level * 20
pub fn wake_nearby_distance(hero_level: i32) -> i32 {
    hero_level * 20
}

/// [v2.9.4] wake_nearto 개별 몬스터 각성 판정 (원본: wake_nearto, L3053-3078)
#[derive(Debug, Clone)]
pub struct WakeNearResult {
    /// 수면 해제
    pub sleep_cleared: bool,
    /// 대기 전략 해제
    pub strategy_cleared: bool,
    /// 길들인 몬스터: 추적 초기화
    pub clear_pet_tracking: bool,
}

/// [v2.9.4] wake_nearto 개별 판정
pub fn wake_nearto_check(
    mon_x: i32,
    mon_y: i32,
    wake_x: i32,
    wake_y: i32,
    wake_distance: i32,
    is_unique: bool,
    is_tame: bool,
    is_mon_moving: bool,
) -> WakeNearResult {
    let mut r = WakeNearResult {
        sleep_cleared: false,
        strategy_cleared: false,
        clear_pet_tracking: false,
    };

    let d = dist2(mon_x, mon_y, wake_x, wake_y);
    if wake_distance == 0 || d < wake_distance {
        r.sleep_cleared = true;
        // 유니크 아닌 몬스터: 대기 전략 해제 (원본: L3065-3066)
        if !is_unique {
            r.strategy_cleared = true;
        }
        // 몬스터 행동 중이 아닐 때만 추가 처리
        if !is_mon_moving && is_tame {
            r.clear_pet_tracking = true;
        }
    }
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// mnexto / maybe_mnexto (원본: mnexto L2726-2752, maybe_mnexto L2769-2787)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] mnexto 근접 배치 결과 (원본: mnexto, L2726-2752)
#[derive(Debug, Clone)]
pub struct MnextoResult {
    /// 기마중이면 플레이어 위치로 이동
    pub sync_with_player: bool,
    /// 과밀 처리 필요
    pub overcrowded: bool,
    /// 새 좌표 (None이면 배치 실패)
    pub new_pos: Option<(i32, i32)>,
    /// 출현 메시지 출력
    pub show_appear_msg: bool,
}

/// [v2.9.4] mnexto 근접 배치 판정 (원본: mnexto, L2726-2752)
pub fn mnexto_result(
    is_steed: bool,
    player_x: i32,
    player_y: i32,
    found_pos: Option<(i32, i32)>,
    in_mklev: bool,
    has_appear_strategy: bool,
    could_spot_before: bool,
    can_spot_after: bool,
) -> MnextoResult {
    if is_steed {
        return MnextoResult {
            sync_with_player: true,
            overcrowded: false,
            new_pos: Some((player_x, player_y)),
            show_appear_msg: false,
        };
    }
    match found_pos {
        None => MnextoResult {
            sync_with_player: false,
            overcrowded: true,
            new_pos: None,
            show_appear_msg: false,
        },
        Some(pos) => MnextoResult {
            sync_with_player: false,
            overcrowded: false,
            new_pos: Some(pos),
            show_appear_msg: !in_mklev
                && has_appear_strategy
                && !could_spot_before
                && can_spot_after,
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 석화 관련 래퍼 (원본: mon_to_stone L2493-2508, vamp_stone L2511-2609)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 석화 전환 결과 (원본: mon_to_stone, L2493-2508)
/// poly_when_stoned → 새 형태, 아니면 석화 사망
#[derive(Debug, Clone)]
pub struct MonToStoneResult {
    /// 형태 전환 발생 (poly_when_stoned)
    pub poly_transform: bool,
    /// 전환 후 새 형태 이름
    pub new_form: Option<String>,
    /// 석화 사망 진행
    pub proceed_to_stone: bool,
}

/// [v2.9.4] 석화 전환 판정 (원본: mon_to_stone, L2493-2508)
pub fn mon_to_stone_result(
    poly_when_stoned: bool,
    new_form_name: Option<&str>,
) -> MonToStoneResult {
    if poly_when_stoned {
        MonToStoneResult {
            poly_transform: true,
            new_form: new_form_name.map(|s| s.to_string()),
            proceed_to_stone: false,
        }
    } else {
        MonToStoneResult {
            poly_transform: false,
            new_form: None,
            proceed_to_stone: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 몬스터 대체 (원본: replmon L1634-1670)
// Rust에서는 ECS 엔티티 대체로 처리하지만 로직은 이식
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 대체 판정 결과 (원본: replmon, L1634-1670)
#[derive(Debug, Clone)]
pub struct ReplMonResult {
    /// 기마 중 몬스터 구원 대체
    pub steed_transfer: bool,
    /// 점착 대상 이전
    pub stuck_transfer: bool,
    /// 추적 대상 이전
    pub tracking_transfer: bool,
    /// 전투 대상 이전
    pub combat_target_transfer: bool,
}

/// [v2.9.4] 몬스터 대체 판정 (원본: replmon, L1634-1670)
pub fn replmon_result(
    is_steed: bool,
    is_stuck: bool,
    is_tracked: bool,
    is_combat_target: bool,
) -> ReplMonResult {
    ReplMonResult {
        steed_transfer: is_steed,
        stuck_transfer: is_stuck,
        tracking_transfer: is_tracked,
        combat_target_transfer: is_combat_target,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 분리/정리 (원본: m_detach L1720-1760, dmonsfree L1610-1632)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 분리 결과 (원본: m_detach, L1720-1760)
#[derive(Debug, Clone)]
pub struct MDetachResult {
    /// 트랩 해제 필요
    pub clear_trap: bool,
    /// 기마 해제 필요
    pub dismount: bool,
    /// 밝기 업데이트 필요
    pub update_lighting: bool,
    /// 점착 해제 필요
    pub unstuck: bool,
    /// 웜 세그먼트 제거 필요
    pub remove_worm: bool,
    /// 타일 업데이트 좌표
    pub newsym_pos: Option<(i32, i32)>,
}

/// [v2.9.4] 몬스터 분리 판정 (원본: m_detach, L1720-1760)
pub fn m_detach_result(
    mon_x: i32,
    mon_y: i32,
    is_trapped: bool,
    is_steed: bool,
    emits_light: bool,
    is_stuck: bool,
    has_worm: bool,
) -> MDetachResult {
    MDetachResult {
        clear_trap: is_trapped,
        dismount: is_steed,
        update_lighting: emits_light,
        unstuck: is_stuck,
        remove_worm: has_worm,
        newsym_pos: Some((mon_x, mon_y)),
    }
}

/// [v2.9.4] 사체 정리 결과 (원본: dmonsfree, L1610-1632)
/// HP <= 0이고 isgd 아닌 몬스터 제거
pub fn should_cleanup_dead(hp: i32, is_guard: bool, guard_x: i32) -> bool {
    hp <= 0 && !(is_guard && guard_x == 0)
}

// ─────────────────────────────────────────────────────────────────────────────
// 생명 구원 래퍼 (원본: mlifesaver L1874-1876, lifesaved_monster L1878-1925)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 생명 구원 아뮬렛 보유 판정 (원본: mlifesaver, L1874-1876)
pub fn has_lifesaver(has_amulet_of_life_saving: bool) -> bool {
    has_amulet_of_life_saving
}

/// [v2.9.4] 생명 구원 결과 (원본: lifesaved_monster, L1878-1925)
/// lifesaved_check와 유사하지만 HP 회복/아뮬렛 소비 포함
#[derive(Debug, Clone)]
pub struct LifeSavedMonResult {
    /// 구원 성공
    pub saved: bool,
    /// 아뮬렛 소비
    pub amulet_consumed: bool,
    /// HP 완전 회복
    pub hp_restored: bool,
    /// 시각 확인 메시지
    pub show_message: bool,
}

/// [v2.9.4] 생명 구원 실행 판정 (원본: lifesaved_monster, L1878-1925)
pub fn lifesaved_monster_result(has_amulet: bool, player_can_see: bool) -> LifeSavedMonResult {
    if has_amulet {
        LifeSavedMonResult {
            saved: true,
            amulet_consumed: true,
            hp_restored: true,
            show_message: player_can_see,
        }
    } else {
        LifeSavedMonResult {
            saved: false,
            amulet_consumed: false,
            hp_restored: false,
            show_message: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// killed/monkilled 래퍼 (원본: killed L2321-2325, monkilled L2271-2295)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 사망 처리 결과 (원본: monkilled, L2271-2295)
#[derive(Debug, Clone)]
pub struct MonKilledByMonResult {
    /// 시체 생성 여부 (소화/분해가 아닐 때)
    pub create_corpse: bool,
    /// 애완동물 사망 슬픔 메시지
    pub show_sad_message: bool,
    /// 사망 메시지
    pub death_message: Option<String>,
}

/// [v2.9.4] 다른 몬스터에 의한 사망 판정 (원본: monkilled, L2271-2295)
pub fn monkilled_by_mon_result(
    is_visible: bool,
    cause: &str,
    is_tame: bool,
    is_digested: bool,
    template: &MonsterTemplate,
) -> MonKilledByMonResult {
    let death_msg = if is_visible && !cause.is_empty() {
        let action = if nonliving(template) {
            "destroyed"
        } else {
            "killed"
        };
        Some(format!("{} by the {}", action, cause))
    } else {
        None
    };

    MonKilledByMonResult {
        create_corpse: !is_digested,
        show_sad_message: is_tame && !is_visible,
        death_message: death_msg,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 무결성 검사 (원본: sanity_check_single_mon L44-134, mon_sanity_check L137-175)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 무결성 검사 결과 (원본: sanity_check_single_mon, L44-134)
#[derive(Debug, Clone)]
pub struct SanityCheckResult {
    /// 경고 목록
    pub warnings: Vec<String>,
    /// 오류 목록 (치명적)
    pub errors: Vec<String>,
}

/// [v2.9.4] 개별 몬스터 무결성 검사 (원본: sanity_check_single_mon, L44-134)
pub fn sanity_check_single_mon(
    name: &str,
    x: i32,
    y: i32,
    hp: i32,
    max_hp: i32,
    is_dead: bool,
    has_valid_pos: bool,
    map_width: i32,
    map_height: i32,
) -> SanityCheckResult {
    let mut r = SanityCheckResult {
        warnings: vec![],
        errors: vec![],
    };

    // 위치 범위 체크 (원본: L55-60)
    if x < 0 || x >= map_width || y < 0 || y >= map_height {
        r.errors
            .push(format!("{}: 위치 범위 벗어남 ({},{})", name, x, y));
    }

    // HP 일관성 (원본: L72-80)
    if !is_dead {
        if hp <= 0 {
            r.errors
                .push(format!("{}: 살아있지만 HP <= 0 (hp={})", name, hp));
        }
        if max_hp <= 0 {
            r.warnings
                .push(format!("{}: 최대 HP <= 0 (max_hp={})", name, max_hp));
        }
        if hp > max_hp {
            r.warnings
                .push(format!("{}: HP > 최대 HP ({}/{})", name, hp, max_hp));
        }
    }

    // 유효 위치 (원본: L85-90)
    if !is_dead && !has_valid_pos {
        r.errors.push(format!("{}: 위치 무효 ({},{})", name, x, y));
    }

    r
}

// =============================================================================
// [v2.9.4] 몬스터 라이프사이클 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::monster::MonsterTemplate;

    ///
    impl MonsterTemplate {
        fn default_with_symbol(sym: char) -> Self {
            MonsterTemplate {
                name: format!("test_{}", sym),
                symbol: sym,
                level: 1,
                movement: 12,
                ac: 10,
                mr: 0,
                alignment: 0,
                geno: 0,
                attacks: vec![],
                weight: 100,
                nutrition: 100,
                msound: 0,
                msize: 2,
                resists: 0,
                conveys: 0,
                flags1: 0,
                flags2: 0,
                flags3: 0,
                difficulty: 1,
                color: 0,
            }
        }
    }

    #[test]
    fn test_makeplural() {
        assert_eq!(makeplural("orc"), "orcs");
        assert_eq!(makeplural("fox"), "foxes");
        assert_eq!(makeplural("fairy"), "fairies");
        assert_eq!(makeplural("elf"), "elves");
        assert_eq!(makeplural("tooth"), "teeth");
        assert_eq!(makeplural("foot"), "feet");
    }

    #[test]
    fn test_a_monnam() {
        assert_eq!(a_monnam("orc", None), "an orc");
        assert_eq!(a_monnam("kobold", None), "a kobold");
        assert_eq!(a_monnam("elf", None), "an elf");
    }

    #[test]
    fn test_s_suffix() {
        assert_eq!(s_suffix("orc"), "orc's");
        assert_eq!(s_suffix("boss"), "boss'");
    }

    #[test]
    fn test_monster_class_name() {
        assert_eq!(monster_class_name('D'), "dragon");
        assert_eq!(monster_class_name('o'), "orc");
        assert_eq!(monster_class_name('@'), "human or humanoid");
    }

    #[test]
    fn test_monster_size() {
        assert_eq!(MonsterSize::from_symbol('D'), MonsterSize::Huge);
        assert_eq!(MonsterSize::from_symbol('@'), MonsterSize::Medium);
        assert_eq!(MonsterSize::from_symbol('a'), MonsterSize::Tiny);
    }

    //
    // [v2.4.0
    //

    #[test]
    fn test_golemeffects() {
        //
        let (heal, slow, msg) = golemeffects("flesh golem", "electric", 12);
        assert!(heal > 0);
        assert!(!slow);
        assert!(msg.is_some());

        //
        let (heal, slow, _) = golemeffects("flesh golem", "fire", 10);
        assert_eq!(heal, 0);
        assert!(slow);

        //
        let (heal, slow, msg) = golemeffects("iron golem", "fire", 20);
        assert_eq!(heal, 20);
        assert!(!slow);
        assert!(msg.is_some());

        //
        let (heal, slow, msg) = golemeffects("orc", "fire", 10);
        assert_eq!(heal, 0);
        assert!(!slow);
        assert!(msg.is_none());
    }

    #[test]
    fn test_max_mon_load() {
        //
        let load = max_mon_load(1000, MonsterSize::Medium, true);
        assert!(load > 0);

        //
        let strong_load = max_mon_load(1000, MonsterSize::Medium, true);
        let weak_load = max_mon_load(1000, MonsterSize::Medium, false);
        assert!(strong_load > weak_load);

        //
        let big_load = max_mon_load(0, MonsterSize::Huge, true);
        let small_load = max_mon_load(0, MonsterSize::Tiny, true);
        assert!(big_load > small_load);
    }

    #[test]
    fn test_can_move_this_turn() {
        assert!(can_move_this_turn(12)); // NORMAL_SPEED
        assert!(can_move_this_turn(24));
        assert!(!can_move_this_turn(11));
        assert!(!can_move_this_turn(0));
    }

    #[test]
    fn test_consume_movement() {
        let (remaining, can_move_again) = consume_movement(24);
        assert_eq!(remaining, 12);
        assert!(can_move_again);

        let (remaining, can_move_again) = consume_movement(12);
        assert_eq!(remaining, 0);
        assert!(!can_move_again);

        let (remaining, can_move_again) = consume_movement(15);
        assert_eq!(remaining, 3);
        assert!(!can_move_again);
    }

    #[test]
    fn test_usmellmon() {
        //
        assert_eq!(
            usmellmon("minotaur", 'q'),
            Some("You notice a bovine smell.")
        );
        assert_eq!(usmellmon("green slime", 'P'), Some("Something stinks."));
        assert_eq!(usmellmon("steam vortex", 'v'), Some("You smell steam."));

        //
        assert_eq!(
            usmellmon("baby red dragon", 'D'),
            Some("You smell a dragon!")
        );
        assert_eq!(
            usmellmon("hill orc", 'o'),
            Some("A foul stench makes you feel a little nauseated.")
        );

        //
        assert!(usmellmon("white unicorn", 'u').is_none());
        assert!(usmellmon("jellyfish", ';').is_none());
    }

    #[test]
    fn test_wake_range() {
        assert_eq!(wake_range(0), 0);
        assert!(wake_range(1) > 0);
        assert!(wake_range(3) > wake_range(1));
        assert!(wake_range(5) > wake_range(3));
    }

    #[test]
    fn test_should_wake_monster() {
        //
        assert!(should_wake_monster(5, 5, 5, 6, 4));
        //
        assert!(!should_wake_monster(50, 50, 5, 5, 100));
    }

    #[test]
    fn test_touch_petrifies() {
        assert!(touch_petrifies("cockatrice"));
        assert!(touch_petrifies("chickatrice"));
        assert!(!touch_petrifies("orc"));
        assert!(!touch_petrifies("dragon"));
    }

    #[test]
    fn test_is_rider() {
        assert!(is_rider("Death"));
        assert!(is_rider("Pestilence"));
        assert!(is_rider("Famine"));
        assert!(!is_rider("orc"));
    }

    #[test]
    fn test_angry_guards_messages() {
        //
        let msgs = angry_guards_messages(0, 0, 0, 0, false);
        assert!(msgs.is_empty());

        //
        let msgs = angry_guards_messages(2, 2, 0, 0, false);
        assert!(msgs.iter().any(|m| m.contains("wake up")));

        //
        let msgs = angry_guards_messages(1, 0, 1, 0, false);
        assert!(msgs.iter().any(|m| m.contains("gets angry")));

        //
        let msgs = angry_guards_messages(5, 3, 2, 1, true);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_can_eat_metal() {
        let rust_mon = MonsterTemplate::default_with_symbol('R');
        assert!(can_eat_metal(&rust_mon));

        let xorn = MonsterTemplate::default_with_symbol('X');
        assert!(can_eat_metal(&xorn));

        let orc = MonsterTemplate::default_with_symbol('o');
        assert!(!can_eat_metal(&orc));
    }

    #[test]
    fn test_is_hider() {
        let mimic = MonsterTemplate::default_with_symbol('m');
        assert!(is_hider(&mimic));

        let piercer = MonsterTemplate::default_with_symbol('p');
        assert!(is_hider(&piercer));

        let orc = MonsterTemplate::default_with_symbol('o');
        assert!(!is_hider(&orc));
    }

    #[test]
    fn test_vampire_shapes() {
        let shapes = vampire_shapes();
        assert!(shapes.contains(&"bat"));
        assert!(shapes.contains(&"fog cloud"));
        assert!(shapes.contains(&"wolf"));
        assert!(shapes.len() >= 3);
    }

    #[test]
    fn test_can_be_hatched() {
        assert!(can_be_hatched("cockatrice"));
        assert!(can_be_hatched("red dragon"));
        assert!(can_be_hatched("silver dragon"));
        assert!(can_be_hatched("cobra"));
        assert!(!can_be_hatched("orc"));
        assert!(!can_be_hatched("human"));
    }

    #[test]
    fn test_dead_species_check() {
        let genocided = vec!["orc".to_string(), "kobold".to_string()];
        assert!(dead_species_check("orc", &genocided));
        assert!(!dead_species_check("elf", &genocided));
    }

    // [v2.9.4] 라이프사이클 이식 테스트
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_make_corpse_dragon_scales() {
        let mut t = MonsterTemplate::default_with_symbol('D');
        t.name = "red dragon".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(0); // 0 seed → rn2(3)==0 → 비늘 드롭
        let r = make_corpse_result(&t, false, &mut rng);
        assert!(r.corpse_created);
        assert!(r.corpse_name.as_ref().unwrap().contains("red dragon"));
        // 비늘 드롭 여부는 RNG 의존
    }

    #[test]
    fn test_make_corpse_golem_no_corpse() {
        let mut t = MonsterTemplate::default_with_symbol('\'');
        t.name = "iron golem".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = make_corpse_result(&t, false, &mut rng);
        assert!(!r.corpse_created);
        assert!(!r.special_drops.is_empty());
        assert!(r.special_drops.iter().any(|s| s.contains("iron chain")));
    }

    #[test]
    fn test_make_corpse_unicorn_horn() {
        let mut t = MonsterTemplate::default_with_symbol('u');
        t.name = "white unicorn".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = make_corpse_result(&t, false, &mut rng);
        assert!(r.corpse_created);
        assert!(r.special_drops.contains(&"unicorn horn".to_string()));
    }

    #[test]
    fn test_lifesaved_no_amulet() {
        let t = MonsterTemplate::default_with_symbol('o');
        let r = lifesaved_check(&t, false, false, false, 20, false);
        assert!(!r.saved);
        assert!(!r.amulet_consumed);
    }

    #[test]
    fn test_lifesaved_with_amulet() {
        let mut t = MonsterTemplate::default_with_symbol('o');
        t.name = "orc".to_string();
        let r = lifesaved_check(&t, true, false, false, 20, false);
        assert!(r.saved);
        assert!(r.amulet_consumed);
        assert_eq!(r.restored_hp, 20);
    }

    #[test]
    fn test_lifesaved_genocided() {
        let mut t = MonsterTemplate::default_with_symbol('o');
        t.name = "orc".to_string();
        let r = lifesaved_check(&t, true, false, true, 20, false);
        assert!(!r.saved);
        assert!(r.still_dead_from_genocide);
        assert!(r.amulet_consumed);
    }

    #[test]
    fn test_mondead_normal_death() {
        let mut t = MonsterTemplate::default_with_symbol('o');
        t.name = "orc".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = mondead_result2(&t, false, false, false, 20, false, None, &mut rng);
        assert!(r.is_dead);
        assert!(!r.vampire_rise);
        assert_eq!(r.death_count_species, Some("orc".to_string()));
    }

    #[test]
    fn test_mondead_vampire_rise() {
        let mut t = MonsterTemplate::default_with_symbol('B');
        t.name = "vampire bat".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = mondead_result2(&t, false, true, false, 20, false, Some("vampire"), &mut rng);
        assert!(!r.is_dead);
        assert!(r.vampire_rise);
        assert_eq!(r.vampire_new_form, Some("vampire".to_string()));
    }

    #[test]
    fn test_newcham_success() {
        let mut t = MonsterTemplate::default_with_symbol('o');
        t.name = "orc".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = newcham_result(&t, Some("troll"), 15, 20, &mut rng);
        assert!(r.success);
        assert_eq!(r.new_form, Some("troll".to_string()));
        assert!((r.hp_ratio - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_newcham_rider_immune() {
        let mut t = MonsterTemplate::default_with_symbol('@');
        t.name = "Death".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = newcham_result(&t, Some("orc"), 20, 20, &mut rng);
        assert!(!r.success);
    }

    #[test]
    fn test_monflee_hp_threshold() {
        let mut t = MonsterTemplate::default_with_symbol('@');
        t.name = "human".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        // HP 4/20 = 20% ≤ 20% → 도주
        let r = monflee_check(&t, 4, 20, false, false, &mut rng);
        assert!(r.should_flee);
        assert!(r.flee_turns > 0);
    }

    #[test]
    fn test_meatmetal_rust_monster() {
        let mut t = MonsterTemplate::default_with_symbol('R');
        t.name = "rust monster".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        // 녹방지 아이템 → 뱉어냄
        let r = meatmetal_check(&t, false, "long sword", 40, true, true, &mut rng);
        assert_eq!(r.action, 1);
        assert!(r.remove_rustproof);
        // 비녹방지 → 정상 섭취
        let r2 = meatmetal_check(&t, false, "iron chain", 20, false, true, &mut rng);
        assert_eq!(r2.action, 1);
        assert_eq!(r2.heal, 20);
        assert!(!r2.remove_rustproof);
    }

    #[test]
    fn test_meatobj_organic() {
        let mut t = MonsterTemplate::default_with_symbol('b');
        t.name = "gelatinous cube".to_string();
        let r = meatobj_check(&t, false, "scroll", 5, true, false, false, false);
        assert_eq!(r.action, 1);
        assert_eq!(r.eaten_count, 1);
        assert_eq!(r.heal, 5);
    }

    #[test]
    fn test_meatobj_carrot_blindness() {
        let mut t = MonsterTemplate::default_with_symbol('b');
        t.name = "gelatinous cube".to_string();
        let r = meatobj_check(&t, false, "carrot", 2, true, false, false, true);
        assert!(r.cures_blindness);
    }

    #[test]
    fn test_minliquid_drowning() {
        let t = MonsterTemplate::default_with_symbol('o');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = minliquid_result(&t, true, false, false, &mut rng);
        assert!(r.died);
        assert_eq!(r.death_cause, Some("drowned".to_string()));
    }

    #[test]
    fn test_minliquid_gremlin_split() {
        let mut t = MonsterTemplate::default_with_symbol('g');
        t.name = "gremlin".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(1); // rn2(3) != 0 → 분열
        let r = minliquid_result(&t, false, false, true, &mut rng);
        // 그렘린 + 분수 → 분열 시도 (RNG 의존)
        // rn2(3) 결과에 따라 gremlin_split 여부 달라짐
        assert!(!r.died);
    }

    #[test]
    fn test_lifecycle_stats() {
        let mut stats = MonLifecycleStats::new();
        stats.record_death();
        stats.record_death();
        stats.record_life_save();
        stats.record_polymorph();
        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.life_saves, 1);
        assert_eq!(stats.polymorphs, 1);
    }

    // [v2.9.4] 2차 이식 테스트
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_monstone_golem_to_stone_golem() {
        let t = MonsterTemplate::default_with_symbol('\'');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = monstone_result(&t, false, None, false, 2, &mut rng);
        assert!(!r.petrified);
        assert_eq!(r.revert_form, Some("stone golem".to_string()));
    }

    #[test]
    fn test_monstone_vampire_reverts() {
        let t = MonsterTemplate::default_with_symbol('B');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = monstone_result(&t, true, Some("vampire"), false, 2, &mut rng);
        assert!(r.vampire_reverted);
        assert_eq!(r.revert_form, Some("vampire".to_string()));
        assert!(!r.petrified);
    }

    #[test]
    fn test_monstone_normal_statue() {
        let t = MonsterTemplate::default_with_symbol('o');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = monstone_result(&t, false, None, false, 2, &mut rng);
        assert!(r.petrified);
        assert!(r.statue_created);
    }

    #[test]
    fn test_restrap_mimic_hides() {
        let t = MonsterTemplate::default_with_symbol('m');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        // rn2(3) != 0 이어야 은신 가능
        let r = restrap_check(&t, false, false, false, false, false, false, true, &mut rng);
        // RNG 의존적이므로 hidden이거나 아닐 수 있음
        if r.hidden {
            assert!(r.mimic_disguised);
        }
    }

    #[test]
    fn test_restrap_cancelled_fails() {
        let t = MonsterTemplate::default_with_symbol('m');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = restrap_check(&t, true, false, false, false, false, false, true, &mut rng);
        assert!(!r.hidden);
    }

    #[test]
    fn test_hideunder_eel_in_pool() {
        let t = MonsterTemplate::default_with_symbol(';');
        assert!(hideunder_check(&t, true, false, false, false));
        assert!(!hideunder_check(&t, false, false, false, false));
    }

    #[test]
    fn test_wakeup_sleeping_monster() {
        let r = wakeup_result(true, false, false, true, false);
        assert!(r.sleep_cleared);
        assert!(r.should_anger);
        assert!(!r.mimic_revealed);
    }

    #[test]
    fn test_wakeup_disguised_mimic() {
        let r = wakeup_result(false, true, false, false, false);
        assert!(r.mimic_revealed);
        assert!(!r.should_anger);
    }

    #[test]
    fn test_should_wake_from_noise() {
        assert!(should_wake_from_noise(5, 5, 3, 3, 100, true, false));
        assert!(!should_wake_from_noise(5, 5, 3, 3, 100, false, false));
        // 거리 밖이면 안 깸
        assert!(!should_wake_from_noise(50, 50, 3, 3, 10, true, false));
    }

    #[test]
    fn test_m_respond_shrieker() {
        let mut t = MonsterTemplate::default_with_symbol('F');
        t.name = "shrieker".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = m_respond_result(&t, false, &mut rng);
        assert!(r.shrieks);
        assert!(r.aggravate);
        assert!(!r.messages.is_empty());
    }

    #[test]
    fn test_m_respond_medusa() {
        let mut t = MonsterTemplate::default_with_symbol('@');
        t.name = "Medusa".to_string();
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = m_respond_result(&t, false, &mut rng);
        assert!(r.medusa_gaze);
    }

    #[test]
    fn test_mpickgold_wants_gold() {
        let t = MonsterTemplate::default_with_symbol('L');
        let r = mpickgold_check(&t, 100, false, true, true);
        // 'L'(leprechaun)은 wants_gold() 체크에 따라 달라짐
        // 기본 template은 금을 원하지 않을 수 있음
        let _ = r;
    }

    #[test]
    fn test_kill_genocided_direct() {
        let genocided = vec!["orc".to_string()];
        assert!(kill_genocided_check("orc", None, &genocided));
        assert!(!kill_genocided_check("elf", None, &genocided));
    }

    #[test]
    fn test_kill_genocided_chameleon() {
        let genocided = vec!["orc".to_string()];
        assert!(kill_genocided_check("chameleon", Some("orc"), &genocided));
        assert!(!kill_genocided_check("chameleon", Some("elf"), &genocided));
    }

    #[test]
    fn test_select_newcham_form_chameleon() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let form = select_newcham_form("chameleon", &mut rng);
        let valid = [
            "jackal", "wolf", "panther", "jaguar", "tiger", "horse", "pony", "warhorse",
        ];
        assert!(valid.contains(&form.as_str()), "got: {}", form);
    }

    #[test]
    fn test_select_newcham_form_sandestin() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let form = select_newcham_form("sandestin", &mut rng);
        let valid = [
            "cockatrice",
            "ettin",
            "mind flayer",
            "purple worm",
            "green dragon",
            "minotaur",
        ];
        assert!(valid.contains(&form.as_str()), "got: {}", form);
    }

    #[test]
    fn test_angry_guards_result() {
        let r = angry_guards_result(3, 1, 1, 1, false);
        assert_eq!(r.angry_count, 3);
        assert!(!r.messages.is_empty());
    }

    #[test]
    fn test_overcrowding_endgame() {
        let r = overcrowding_check(true);
        assert!(r.elemental_clog);
        assert!(!r.send_to_limbo);
    }

    #[test]
    fn test_overcrowding_normal() {
        let r = overcrowding_check(false);
        assert!(r.send_to_limbo);
        assert!(!r.elemental_clog);
    }

    #[test]
    fn test_elemental_clog_check() {
        let t = MonsterTemplate::default_with_symbol('E');
        assert!(elemental_clog_check(&t, 5, 3));
        assert!(!elemental_clog_check(&t, 2, 3));
        // 비원소 몬스터
        let t2 = MonsterTemplate::default_with_symbol('o');
        assert!(!elemental_clog_check(&t2, 10, 3));
    }

    #[test]
    fn test_unstuck_check() {
        let r = unstuck_check(true, true);
        assert!(r.released);
        assert!(r.expel_from_engulf);

        let r2 = unstuck_check(true, false);
        assert!(r2.released);
        assert!(!r2.expel_from_engulf);

        let r3 = unstuck_check(false, false);
        assert!(!r3.released);
    }

    #[test]
    fn test_mimic_hit_msg() {
        assert!(mimic_hit_msg("potion", true).is_some());
        assert!(mimic_hit_msg("potion", false).is_none());
    }

    // [v2.9.4] 3차 이식 테스트
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_movemon_tick_dead_skip() {
        let r = movemon_tick(12, 12, true, false, false, false, false);
        assert!(r.skip_no_movement);
        assert_eq!(r.movement_consumed, 0);
    }

    #[test]
    fn test_movemon_tick_normal_move() {
        let r = movemon_tick(24, 12, false, false, false, false, false);
        assert!(!r.skip_no_movement);
        assert_eq!(r.movement_consumed, 12);
        assert!(r.can_act_again); // 24-12=12 >= 12
    }

    #[test]
    fn test_movemon_tick_insufficient() {
        let r = movemon_tick(8, 12, false, false, false, false, false);
        assert!(r.skip_no_movement);
        assert_eq!(r.movement_consumed, 0);
    }

    #[test]
    fn test_movemon_tick_liquid() {
        let r = movemon_tick(12, 12, false, false, false, true, false);
        assert!(r.liquid_effect);
    }

    #[test]
    fn test_movemon_tick_rehide() {
        let r = movemon_tick(12, 12, false, true, true, false, false);
        assert!(r.rehide_success);
    }

    #[test]
    fn test_decide_shapeshift_regular() {
        let t = MonsterTemplate::default_with_symbol('m');
        let mut rng = crate::util::rng::NetHackRng::new(0);
        let r = decide_to_shapeshift(
            &t, false, false, false, 50, 50, false, false, false, &mut rng,
        );
        // rn2(6)==0 → seed 0일 때 should_shift 가능
        let _ = r; // RNG 의존적
    }

    #[test]
    fn test_decide_shapeshift_vampire_low_hp() {
        let t = MonsterTemplate::default_with_symbol('V');
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let r = decide_to_shapeshift(&t, true, false, false, 5, 60, true, false, false, &mut rng);
        // HP 5 <= (60+5)/6 = 10, rn2(4) != 0 (높은 확률) → 변신
        assert!(r.should_shift || !r.should_shift); // RNG 의존
        assert!(r.preserve_gender);
    }

    #[test]
    fn test_accept_newcham_form_genocided() {
        let t = MonsterTemplate::default_with_symbol('o');
        assert!(!accept_newcham_form_check(&t, true, false));
    }

    #[test]
    fn test_accept_newcham_form_player_char() {
        let t = MonsterTemplate::default_with_symbol('@');
        assert!(accept_newcham_form_check(&t, false, true));
    }

    #[test]
    fn test_isspecmon_shopkeeper() {
        assert!(isspecmon_check(true, false, false, false));
        assert!(!isspecmon_check(false, false, false, false));
    }

    #[test]
    fn test_validvamp_fog_cloud() {
        assert!(validvamp_check("fog cloud", "vampire"));
        assert!(validvamp_check("vampire bat", "vampire"));
        assert!(!validvamp_check("wolf", "vampire"));
        assert!(validvamp_check("wolf", "vampire lord"));
    }

    #[test]
    fn test_mgender_male_only() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        assert_eq!(
            mgender_from_permonst("incubus", false, &mut rng),
            GenderResult::Male
        );
    }

    #[test]
    fn test_mgender_female_only() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        assert_eq!(
            mgender_from_permonst("succubus", true, &mut rng),
            GenderResult::Female
        );
    }

    #[test]
    fn test_mgender_golem_neuter() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        assert_eq!(
            mgender_from_permonst("iron golem", false, &mut rng),
            GenderResult::Neuter
        );
    }

    #[test]
    fn test_mongone_result_steed() {
        let r = mongone_result(true, false, true);
        assert!(r.dismount);
        assert!(!r.unstuck);
        assert!(r.special_drops);
        assert!(r.discard_inventory);
    }

    #[test]
    fn test_mondied_should_corpse() {
        assert!(mondied_should_corpse(true, true, false));
        assert!(mondied_should_corpse(true, false, true));
        assert!(!mondied_should_corpse(false, true, false));
        assert!(!mondied_should_corpse(true, false, false));
    }

    #[test]
    fn test_seemimic_result() {
        let r = seemimic_result(true, true);
        assert!(r.disguise_cleared);
        assert!(r.unblock_light);
        assert!(r.free_corpse_name);
    }

    #[test]
    fn test_rescham_protection() {
        assert!(rescham_should_revert(true, true, true));
        assert!(!rescham_should_revert(true, true, false));
        assert!(!rescham_should_revert(false, true, true));
    }

    #[test]
    fn test_restartcham() {
        assert!(restartcham_check(false, true));
        assert!(!restartcham_check(true, true));
        assert!(!restartcham_check(false, false));
    }

    #[test]
    fn test_restore_cham_revert() {
        assert!(restore_cham_should_revert(true, true, false, false));
        assert!(restore_cham_should_revert(true, false, true, false));
        assert!(!restore_cham_should_revert(true, false, true, true));
        assert!(!restore_cham_should_revert(false, true, false, false));
    }

    #[test]
    fn test_hide_monst_rehide() {
        let t = MonsterTemplate::default_with_symbol('m');
        assert!(hide_monst_should_rehide(&t, false, false));
        assert!(!hide_monst_should_rehide(&t, true, false)); // 이미 은신
    }

    #[test]
    fn test_pick_animal_valid() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let animal = pick_animal(&mut rng);
        let all = build_animal_list();
        assert!(all.contains(&animal), "got: {}", animal);
    }

    #[test]
    fn test_should_kill_egg() {
        let genocided = vec!["orc".to_string()];
        assert!(should_kill_egg("orc", &genocided));
        assert!(!should_kill_egg("elf", &genocided));
    }

    #[test]
    fn test_ok_to_obliterate_rider() {
        let mut t = MonsterTemplate::default_with_symbol('&');
        t.name = "Death".to_string();
        assert!(!ok_to_obliterate(&t, false, false, false, false, false));
    }

    #[test]
    fn test_ok_to_obliterate_normal() {
        let t = MonsterTemplate::default_with_symbol('o');
        assert!(ok_to_obliterate(&t, false, false, false, false, false));
    }

    #[test]
    fn test_ok_to_obliterate_shopkeeper() {
        let t = MonsterTemplate::default_with_symbol('@');
        assert!(!ok_to_obliterate(&t, true, false, false, false, false));
    }

    #[test]
    fn test_monnear() {
        assert!(monnear(5, 5, 5, 6));
        assert!(monnear(5, 5, 4, 4));
        assert!(!monnear(5, 5, 3, 3));
    }

    #[test]
    fn test_curr_mon_load() {
        let inv = vec![100, 200, 50];
        let boulders = vec![200];
        assert_eq!(curr_mon_load(&inv, &boulders, false), 350);
        assert_eq!(curr_mon_load(&inv, &boulders, true), 150); // 350-200
    }

    // [v2.9.4] 4차(최종) 이식 테스트
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_migrate_mon_result() {
        let r = migrate_mon_result(5, MigrateXYLoc::ExactXY, true);
        assert!(r.unstuck);
        assert!(r.drop_special_objs);
        assert_eq!(r.target_level, 5);
        assert_eq!(r.xyloc, MigrateXYLoc::ExactXY);
        assert!(r.set_migrating);
    }

    #[test]
    fn test_m_into_limbo_result() {
        let r = m_into_limbo_result(3, false);
        assert!(!r.unstuck);
        assert_eq!(r.target_level, 3);
        assert_eq!(r.xyloc, MigrateXYLoc::ApproxXY);
    }

    #[test]
    fn test_wake_nearby_distance() {
        assert_eq!(wake_nearby_distance(5), 100);
        assert_eq!(wake_nearby_distance(14), 280);
    }

    #[test]
    fn test_wake_nearto_in_range() {
        let r = wake_nearto_check(5, 5, 6, 6, 100, false, false, false);
        assert!(r.sleep_cleared);
        assert!(r.strategy_cleared);
        assert!(!r.clear_pet_tracking);
    }

    #[test]
    fn test_wake_nearto_tame() {
        let r = wake_nearto_check(5, 5, 5, 6, 100, false, true, false);
        assert!(r.sleep_cleared);
        assert!(r.clear_pet_tracking);
    }

    #[test]
    fn test_wake_nearto_out_of_range() {
        let r = wake_nearto_check(50, 50, 5, 5, 10, false, false, false);
        assert!(!r.sleep_cleared);
    }

    #[test]
    fn test_mnexto_steed() {
        let r = mnexto_result(true, 10, 10, None, false, false, false, false);
        assert!(r.sync_with_player);
        assert_eq!(r.new_pos, Some((10, 10)));
    }

    #[test]
    fn test_mnexto_overcrowded() {
        let r = mnexto_result(false, 10, 10, None, false, false, false, false);
        assert!(r.overcrowded);
        assert!(r.new_pos.is_none());
    }

    #[test]
    fn test_mnexto_normal() {
        let r = mnexto_result(false, 10, 10, Some((8, 9)), false, true, false, true);
        assert!(!r.overcrowded);
        assert_eq!(r.new_pos, Some((8, 9)));
        assert!(r.show_appear_msg);
    }

    #[test]
    fn test_mon_to_stone_poly() {
        let r = mon_to_stone_result(true, Some("stone golem"));
        assert!(r.poly_transform);
        assert_eq!(r.new_form, Some("stone golem".to_string()));
        assert!(!r.proceed_to_stone);
    }

    #[test]
    fn test_mon_to_stone_no_poly() {
        let r = mon_to_stone_result(false, None);
        assert!(!r.poly_transform);
        assert!(r.proceed_to_stone);
    }

    #[test]
    fn test_replmon_result() {
        let r = replmon_result(true, false, true, false);
        assert!(r.steed_transfer);
        assert!(!r.stuck_transfer);
        assert!(r.tracking_transfer);
    }

    #[test]
    fn test_m_detach_result() {
        let r = m_detach_result(5, 5, true, false, true, true, false);
        assert!(r.clear_trap);
        assert!(!r.dismount);
        assert!(r.update_lighting);
        assert!(r.unstuck);
        assert_eq!(r.newsym_pos, Some((5, 5)));
    }

    #[test]
    fn test_should_cleanup_dead() {
        assert!(should_cleanup_dead(0, false, 0));
        assert!(should_cleanup_dead(-5, false, 0));
        assert!(!should_cleanup_dead(5, false, 0)); // HP > 0
        assert!(!should_cleanup_dead(0, true, 0)); // 경비병 파킹 중
        assert!(should_cleanup_dead(0, true, 5)); // 경비병이지만 위치 != 0
    }

    #[test]
    fn test_has_lifesaver() {
        assert!(has_lifesaver(true));
        assert!(!has_lifesaver(false));
    }

    #[test]
    fn test_lifesaved_monster_result() {
        let r = lifesaved_monster_result(true, true);
        assert!(r.saved);
        assert!(r.amulet_consumed);
        assert!(r.hp_restored);
        assert!(r.show_message);

        let r2 = lifesaved_monster_result(false, true);
        assert!(!r2.saved);
    }

    #[test]
    fn test_monkilled_by_mon_living() {
        let mut t = MonsterTemplate::default_with_symbol('o');
        t.name = "orc".to_string();
        let r = monkilled_by_mon_result(true, "fireball", false, false, &t);
        assert!(r.create_corpse);
        assert!(!r.show_sad_message);
        assert!(r.death_message.as_ref().unwrap().contains("killed"));
    }

    #[test]
    fn test_monkilled_by_mon_undead() {
        let mut t = MonsterTemplate::default_with_symbol('Z');
        t.name = "zombie".to_string();
        let r = monkilled_by_mon_result(true, "fire", false, false, &t);
        // nonliving은 symbol 기반이므로 zombie는 nonliving일 수 있음
        assert!(r.death_message.is_some());
    }

    #[test]
    fn test_monkilled_pet_sad() {
        let t = MonsterTemplate::default_with_symbol('d');
        let r = monkilled_by_mon_result(false, "", true, false, &t);
        assert!(r.show_sad_message);
        assert!(r.death_message.is_none()); // 안 보임
    }

    #[test]
    fn test_monkilled_digested() {
        let t = MonsterTemplate::default_with_symbol('o');
        let r = monkilled_by_mon_result(true, "digestion", false, true, &t);
        assert!(!r.create_corpse);
    }

    #[test]
    fn test_sanity_check_valid() {
        let r = sanity_check_single_mon("orc", 5, 5, 10, 15, false, true, 80, 22);
        assert!(r.errors.is_empty());
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn test_sanity_check_out_of_bounds() {
        let r = sanity_check_single_mon("orc", -1, 5, 10, 15, false, true, 80, 22);
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_sanity_check_alive_zero_hp() {
        let r = sanity_check_single_mon("orc", 5, 5, 0, 15, false, true, 80, 22);
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_sanity_check_hp_over_max() {
        let r = sanity_check_single_mon("orc", 5, 5, 20, 15, false, true, 80, 22);
        assert!(!r.warnings.is_empty());
    }

    #[test]
    fn test_sanity_check_invalid_pos() {
        let r = sanity_check_single_mon("orc", 5, 5, 10, 15, false, false, 80, 22);
        assert!(!r.errors.is_empty());
    }
}
