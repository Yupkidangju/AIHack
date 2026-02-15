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
    result.experience = monster_experience(
        template,
        template.difficulty as i32 * 4,
    );

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
//
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
}
