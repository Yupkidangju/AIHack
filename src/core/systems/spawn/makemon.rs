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

use crate::core::entity::monster::MonsterTemplate;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MakeMonFlags {
    pub no_minvent: bool,
    pub sleeping: bool,
    pub peaceful: bool,
    pub hostile: bool,
    pub no_group: bool,
    pub allow_genocided: bool,
    pub adjacency: bool,
    pub angry: bool,
    pub counting: bool,
}

impl Default for MakeMonFlags {
    fn default() -> Self {
        Self {
            no_minvent: false,
            sleeping: false,
            peaceful: false,
            hostile: false,
            no_group: false,
            allow_genocided: false,
            adjacency: false,
            angry: false,
            counting: true,
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct MakeMonRequest {
    pub template_name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub flags: MakeMonFlags,
}

///
#[derive(Debug, Clone)]
pub struct MakeMonResult {
    pub template_name: String,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub level: i32,
    pub ac: i32,
    pub hostile: bool,
    pub sleeping: bool,
    pub has_inventory: bool,
    pub speed: i32,
    pub symbol: char,
}

// =============================================================================
//
// =============================================================================

///
pub fn rndmonst(
    templates: &std::collections::HashMap<String, MonsterTemplate>,
    dungeon_level: i32,
    rng: &mut NetHackRng,
) -> Option<String> {
    //
    let candidates: Vec<(&String, &MonsterTemplate)> = templates
        .iter()
        .filter(|(_, t)| {
            let diff = (t.level as i32 - dungeon_level).abs();
            diff <= 5 && t.geno & 0x0800 == 0
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    //
    let total_freq: i32 = candidates
        .iter()
        .map(|(_, t)| (t.geno & 0x07) as i32 + 1)
        .sum();

    let mut roll = rng.rn2(total_freq.max(1));
    for (name, t) in &candidates {
        let freq = (t.geno & 0x07) as i32 + 1;
        roll -= freq;
        if roll < 0 {
            return Some((*name).clone());
        }
    }

    candidates.last().map(|(n, _)| (*n).clone())
}

///
pub fn rndmonnum_by_symbol(
    templates: &std::collections::HashMap<String, MonsterTemplate>,
    symbol: char,
    rng: &mut NetHackRng,
) -> Option<String> {
    let candidates: Vec<&String> = templates
        .iter()
        .filter(|(_, t)| t.symbol == symbol)
        .map(|(name, _)| name)
        .collect();

    if candidates.is_empty() {
        return None;
    }

    let idx = rng.rn2(candidates.len() as i32) as usize;
    Some(candidates[idx].clone())
}

// =============================================================================
//
// =============================================================================

///
pub fn makemon(
    request: &MakeMonRequest,
    templates: &std::collections::HashMap<String, MonsterTemplate>,
    dungeon_level: i32,
    rng: &mut NetHackRng,
) -> Option<MakeMonResult> {
    //
    let template_name = match &request.template_name {
        Some(name) => {
            if templates.contains_key(name) {
                name.clone()
            } else {
                return None;
            }
        }
        None => rndmonst(templates, dungeon_level, rng)?,
    };

    let tmpl = templates.get(&template_name)?;

    //
    let (hp, max_hp) = newmonhp(tmpl, rng);

    //
    let hostile = if request.flags.hostile {
        true
    } else if request.flags.peaceful {
        false
    } else {
        determine_hostility(tmpl, rng)
    };

    Some(MakeMonResult {
        template_name,
        x: request.x,
        y: request.y,
        hp,
        max_hp,
        level: tmpl.level as i32,
        ac: tmpl.ac as i32,
        hostile,
        sleeping: request.flags.sleeping,
        has_inventory: !request.flags.no_minvent,
        speed: tmpl.movement as i32,
        symbol: tmpl.symbol,
    })
}

///
pub fn newmonhp(tmpl: &MonsterTemplate, rng: &mut NetHackRng) -> (i32, i32) {
    let level = tmpl.level as i32;
    //
    let dice = level.max(1);
    let mut hp = 0;
    for _ in 0..dice {
        hp += rng.rn2(8) + 1;
    }
    hp = hp.max(1);
    (hp, hp)
}

///
pub fn determine_hostility(tmpl: &MonsterTemplate, rng: &mut NetHackRng) -> bool {
    // MonsterFlags2::HOSTILE / PEACEFUL 泥댄겕
    let flags2 = tmpl.flags2;
    if flags2 & 0x00100000 != 0 {
        // HOSTILE
        return true;
    }
    if flags2 & 0x00200000 != 0 {
        // PEACEFUL
        return false;
    }
    //
    rng.rn2(2) == 0
}

// =============================================================================
//
// =============================================================================

///
pub fn group_size(tmpl: &MonsterTemplate, rng: &mut NetHackRng) -> i32 {
    //
    let geno = tmpl.geno;
    if geno & 0x0200 != 0 {
        // G_LGROUP
        rng.rn2(8) + 4 // 4~11留덈━
    } else if geno & 0x0100 != 0 {
        // G_SGROUP
        rng.rn2(4) + 2 // 2~5留덈━
    } else {
        0
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn should_have_inventory(tmpl: &MonsterTemplate) -> bool {
    //
    let flags1 = tmpl.flags1;
    let notake = flags1 & 0x00000800 != 0; // NOTAKE
    let nohands = flags1 & 0x00002000 != 0; // NOHANDS
    !notake && !nohands
}

///
pub fn initial_weapon(tmpl: &MonsterTemplate, rng: &mut NetHackRng) -> Option<String> {
    let symbol = tmpl.symbol;
    match symbol {
        'o' => {
            //
            let weapons = ["orcish short sword", "scimitar", "orcish dagger"];
            Some(weapons[rng.rn2(weapons.len() as i32) as usize].into())
        }
        'k' => {
            //
            Some("club".into())
        }
        'G' => {
            //
            let weapons = ["crossbow", "club", "thonged club"];
            Some(weapons[rng.rn2(weapons.len() as i32) as usize].into())
        }
        'O' => {
            //
            Some("battle-axe".into())
        }
        'H' => {
            //
            Some("boulder".into())
        }
        'K' => {
            // KoP
            Some("long sword".into())
        }
        '@' => {
            //
            let weapons = ["long sword", "broadsword", "short sword"];
            Some(weapons[rng.rn2(weapons.len() as i32) as usize].into())
        }
        _ => None,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn goodpos(
    x: i32,
    y: i32,
    map_width: i32,
    map_height: i32,
    occupied: &dyn Fn(i32, i32) -> bool,
) -> bool {
    if x < 1 || y < 1 || x >= map_width - 1 || y >= map_height - 1 {
        return false;
    }
    !occupied(x, y)
}

///
pub fn find_spawn_position(
    center_x: i32,
    center_y: i32,
    map_width: i32,
    map_height: i32,
    rng: &mut NetHackRng,
    occupied: &dyn Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    //
    let offsets = [
        (0, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut candidates = Vec::new();
    for (dx, dy) in &offsets {
        let nx = center_x + dx;
        let ny = center_y + dy;
        if goodpos(nx, ny, map_width, map_height, occupied) {
            candidates.push((nx, ny));
        }
    }

    if candidates.is_empty() {
        None
    } else {
        let idx = rng.rn2(candidates.len() as i32) as usize;
        Some(candidates[idx])
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn make_shopkeeper(x: i32, y: i32, shop_room: u8, rng: &mut NetHackRng) -> MakeMonResult {
    let names = [
        "Asidonhopo",
        "Izchak",
        "Shklensky",
        "Astrstrstr",
        "Kosher",
        "Mastrstrstrstr",
        "Nstrstrstr",
    ];
    let idx = rng.rn2(names.len() as i32) as usize;
    MakeMonResult {
        template_name: "shopkeeper".into(),
        x,
        y,
        hp: 200,
        max_hp: 200,
        level: 12,
        ac: 0,
        hostile: false,
        sleeping: false,
        has_inventory: true,
        speed: 18,
        symbol: '@',
    }
}

///
pub fn make_wizard(x: i32, y: i32) -> MakeMonResult {
    MakeMonResult {
        template_name: "Wizard of Yendor".into(),
        x,
        y,
        hp: 150,
        max_hp: 150,
        level: 30,
        ac: -8,
        hostile: true,
        sleeping: false,
        has_inventory: true,
        speed: 12,
        symbol: '@',
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newmonhp() {
        let tmpl = MonsterTemplate {
            name: "test".into(),
            symbol: 'a',
            level: 5,
            movement: 12,
            ac: 7,
            mr: 0,
            alignment: 0,
            geno: 0,
            weight: 100,
            nutrition: 50,
            msound: 0,
            msize: 0,
            resists: 0,
            conveys: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            difficulty: 1,
            color: 0,
            attacks: vec![],
        };
        let mut rng = NetHackRng::new(42);
        let (hp, max_hp) = newmonhp(&tmpl, &mut rng);
        assert!(hp > 0);
        assert_eq!(hp, max_hp);
    }

    #[test]
    fn test_group_size_no_group() {
        let tmpl = MonsterTemplate {
            name: "loner".into(),
            symbol: 'z',
            level: 1,
            movement: 12,
            ac: 10,
            mr: 0,
            alignment: 0,
            geno: 0,
            weight: 50,
            nutrition: 20,
            msound: 0,
            msize: 0,
            resists: 0,
            conveys: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            difficulty: 1,
            color: 0,
            attacks: vec![],
        };
        let mut rng = NetHackRng::new(42);
        assert_eq!(group_size(&tmpl, &mut rng), 0);
    }

    #[test]
    fn test_goodpos() {
        let occupied = |_x: i32, _y: i32| -> bool { false };
        assert!(goodpos(5, 5, 80, 21, &occupied));
        assert!(!goodpos(0, 0, 80, 21, &occupied));
        assert!(!goodpos(79, 20, 80, 21, &occupied));
    }

    #[test]
    fn test_make_wizard() {
        let wiz = make_wizard(10, 10);
        assert_eq!(wiz.level, 30);
        assert!(wiz.hostile);
    }
}
