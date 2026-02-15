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
use crate::core::entity::player::Player;
use crate::ui::log::GameLog;
use legion::world::SubWorld;
use legion::IntoQuery;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeIndex {
    Str = 0,
    Int = 1,
    Wis = 2,
    Dex = 3,
    Con = 4,
    Cha = 5,
}

const PLUSATTR: [&str; 6] = ["strong", "smart", "wise", "agile", "tough", "charismatic"];
const MINUSATTR: [&str; 6] = [
    "weak",
    "stupid",
    "foolish",
    "clumsy",
    "fragile",
    "repulsive",
];

const ATTRMIN: i32 = 3;

///
///
///
///
pub fn adjattrib(
    player: &mut Player,
    ndx: AttributeIndex,
    incr: i32,
    msg_flg: i32,
    log: &mut GameLog,
    turn: u64,
) -> bool {
    if incr == 0 {
        return false;
    }

    let attr = match ndx {
        AttributeIndex::Str => &mut player.str,
        AttributeIndex::Int => &mut player.int,
        AttributeIndex::Wis => &mut player.wis,
        AttributeIndex::Dex => &mut player.dex,
        AttributeIndex::Con => &mut player.con,
        AttributeIndex::Cha => &mut player.cha,
    };

    let old_base = attr.base;

    attr.base += incr;

    let attr_idx = ndx as usize;
    let mut changed = false;

    if incr > 0 {
        let cap = match ndx {
            AttributeIndex::Str => match player.race {
                crate::core::entity::player::Race::Human
                | crate::core::entity::player::Race::Dwarf
                | crate::core::entity::player::Race::Orc => 118,
                _ => 18,
            },
            AttributeIndex::Int => match player.race {
                crate::core::entity::player::Race::Elf => 20,
                _ => 18,
            },
            AttributeIndex::Wis => match player.race {
                crate::core::entity::player::Race::Elf => 20,
                _ => 18,
            },
            AttributeIndex::Dex => match player.race {
                crate::core::entity::player::Race::Elf => 20,
                _ => 18,
            },
            AttributeIndex::Con => match player.race {
                crate::core::entity::player::Race::Dwarf
                | crate::core::entity::player::Race::Orc => 20,
                _ => 18,
            },
            AttributeIndex::Cha => 18,
        };

        if attr.base > attr.max {
            attr.max = attr.base;
            if attr.max > cap {
                attr.base = cap;
                attr.max = cap;
            }
        }
        if attr.base > cap {
            attr.base = cap;
        }
        if attr.base != old_base {
            changed = true;
            if msg_flg <= 0 {
                log.add(
                    format!(
                        "You feel {}{}!",
                        if incr.abs() > 1 { "very " } else { "" },
                        PLUSATTR[attr_idx]
                    ),
                    turn,
                );
            }
        }
    } else {
        if attr.base < ATTRMIN {
            //
            //
            attr.base = ATTRMIN;
        }
        if attr.base != old_base {
            changed = true;
            if msg_flg <= 0 {
                log.add(
                    format!(
                        "You feel {}{}!",
                        if incr.abs() > 1 { "very " } else { "" },
                        MINUSATTR[attr_idx]
                    ),
                    turn,
                );
            }
        }
    }

    if !changed && msg_flg == 0 {
        log.add(
            format!(
                "You're already as {} as you can get.",
                if incr > 0 {
                    PLUSATTR[attr_idx]
                } else {
                    MINUSATTR[attr_idx]
                }
            ),
            turn,
        );
    }

    changed
}

///
pub fn losestr(player: &mut Player, mut num: i32, log: &mut GameLog, turn: u64) {
    //
    while player.str.base - num < 3 {
        num -= 1;
        player.hp -= 6;
        player.hp_max -= 6;
        log.add("You feel much weaker!", turn);
        if player.hp <= 0 {
            return;
        }
    }
    adjattrib(player, AttributeIndex::Str, -num, 1, log, turn);
}

///
pub fn exercise(player: &mut Player, ndx: AttributeIndex, positive: bool) {
    let i = ndx as usize;
    if positive {
        if player.exercise[i] < 50 {
            player.exercise[i] += 1;
        }
    } else {
        if player.exercise[i] > -50 {
            player.exercise[i] -= 1;
        }
    }
}

///
pub fn exerchk(player: &mut Player, log: &mut GameLog, turn: u64) {
    for i in 0..6 {
        let exercise_val = player.exercise[i];
        if exercise_val > 0 {
            //
            let chance = exercise_val; // 1~50
            let r = crate::util::rng::NetHackRng::new(turn + i as u64).rn2(100);
            if r < chance {
                adjattrib(
                    player,
                    match i {
                        0 => AttributeIndex::Str,
                        1 => AttributeIndex::Int,
                        2 => AttributeIndex::Wis,
                        3 => AttributeIndex::Dex,
                        4 => AttributeIndex::Con,
                        _ => AttributeIndex::Cha,
                    },
                    1,
                    -1,
                    log,
                    turn,
                );
            }
        } else if exercise_val < 0 {
            //
            let chance = -exercise_val;
            let r = crate::util::rng::NetHackRng::new(turn + i as u64).rn2(100);
            if r < chance {
                adjattrib(
                    player,
                    match i {
                        0 => AttributeIndex::Str,
                        1 => AttributeIndex::Int,
                        2 => AttributeIndex::Wis,
                        3 => AttributeIndex::Dex,
                        4 => AttributeIndex::Con,
                        _ => AttributeIndex::Cha,
                    },
                    -1,
                    -1,
                    log,
                    turn,
                );
            }
        }
        player.exercise[i] = 0;
    }
}

///
pub fn restore_attrib(player: &mut Player, log: &mut GameLog, turn: u64) {
    //
    if turn > 0 && turn % 1000 == 0 {
        exerchk(player, log, turn);
    }

    //
    if turn % 100 == 0 {
        let attrs = [
            &mut player.str,
            &mut player.int,
            &mut player.wis,
            &mut player.dex,
            &mut player.con,
            &mut player.cha,
        ];

        let mut any_recovered = false;
        for attr in attrs {
            if attr.base < attr.max {
                attr.base += 1;
                any_recovered = true;
            }
        }

        if any_recovered {
            log.add("You feel better.", turn);
        }
    }
}

///
#[legion::system]
#[write_component(Player)]
pub fn attrib_maintenance(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
) {
    let mut query = <&mut Player>::query();
    for player in query.iter_mut(world) {
        restore_attrib(player, log, *turn);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn attribute_description(value: i32) -> &'static str {
    match value {
        ..=5 => "very poor",
        6..=8 => "poor",
        9..=11 => "average",
        12..=14 => "good",
        15..=17 => "very good",
        18 => "excellent",
        19..=20 => "extraordinary",
        21..=24 => "supernatural",
        _ => "godlike",
    }
}

///
pub fn str_display(base: i32) -> String {
    if base <= 18 {
        format!("{}", base)
    } else if base < 118 {
        // 18/01 ~ 18/99
        format!("18/{:02}", base - 18)
    } else if base == 118 {
        "18/**".to_string()
    } else {
        format!("{}", base - 100) // 19+
    }
}

///
pub fn str_damage_bonus(str_val: i32) -> i32 {
    match str_val {
        ..=5 => -1,
        6..=15 => 0,
        16 => 1,
        17 => 2,
        18 => 3,
        19..=50 => (str_val - 18) / 5 + 3,
        51..=68 => 4,
        69..=100 => 5,
        _ => 6,
    }
}

///
pub fn str_hit_bonus(str_val: i32) -> i32 {
    match str_val {
        ..=5 => -2,
        6..=7 => -1,
        8..=16 => 0,
        17 => 1,
        18 => 2,
        19..=100 => 3,
        _ => 3,
    }
}

///
pub fn dex_ac_bonus(dex_val: i32) -> i32 {
    match dex_val {
        ..=3 => 3,
        4..=5 => 2,
        6..=7 => 1,
        8..=14 => 0,
        15 => -1,
        16 => -2,
        17 => -3,
        18 => -4,
        _ => -5,
    }
}

///
pub fn con_hp_bonus(con_val: i32) -> i32 {
    match con_val {
        ..=6 => -1,
        7..=14 => 0,
        15..=16 => 1,
        17 => 2,
        18 => 3,
        _ => 4,
    }
}

///
pub fn wis_energy_bonus(wis_val: i32) -> i32 {
    match wis_val {
        ..=6 => -1,
        7..=13 => 0,
        14..=15 => 1,
        16..=17 => 2,
        18 => 3,
        _ => 4,
    }
}

///
pub fn cha_shop_discount(cha_val: i32) -> i32 {
    //
    match cha_val {
        ..=5 => -10,
        6..=7 => -5,
        8..=10 => 0,
        11..=14 => 0,
        15..=17 => 5,
        18 => 10,
        _ => 15,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn racial_attribute_cap(race: crate::core::entity::player::Race, attr: AttributeIndex) -> i32 {
    use crate::core::entity::player::Race;
    match (race, attr) {
        (Race::Human, _) => 18,
        (Race::Elf, AttributeIndex::Int)
        | (Race::Elf, AttributeIndex::Wis)
        | (Race::Elf, AttributeIndex::Dex) => 20,
        (Race::Elf, AttributeIndex::Con) => 16,
        (Race::Elf, _) => 18,
        (Race::Dwarf, AttributeIndex::Str) | (Race::Dwarf, AttributeIndex::Con) => 20,
        (Race::Dwarf, AttributeIndex::Cha) => 16,
        (Race::Dwarf, _) => 18,
        (Race::Gnome, AttributeIndex::Int) | (Race::Gnome, AttributeIndex::Wis) => 20,
        (Race::Gnome, AttributeIndex::Str) => 16,
        (Race::Gnome, _) => 18,
        (Race::Orc, AttributeIndex::Str) | (Race::Orc, AttributeIndex::Con) => 20,
        (Race::Orc, AttributeIndex::Int) | (Race::Orc, AttributeIndex::Cha) => 14,
        (Race::Orc, _) => 18,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn potion_attribute_gain(blessed: bool, cursed: bool) -> i32 {
    if blessed {
        2
    } else if cursed {
        -1
    } else {
        1
    }
}

///
pub fn attribute_summary(player: &Player) -> String {
    format!(
        "St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
        str_display(player.str.base),
        player.dex.base,
        player.con.base,
        player.int.base,
        player.wis.base,
        player.cha.base,
    )
}

///
pub fn carry_capacity(str_val: i32) -> i32 {
    //
    let base = if str_val <= 18 {
        str_val * 25
    } else {
        450 + (str_val - 18) * 10
    };
    base.max(100) // 理쒖냼 100
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod attrib_extended_tests {
    use super::*;

    #[test]
    fn test_attribute_description() {
        assert_eq!(attribute_description(3), "very poor");
        assert_eq!(attribute_description(10), "average");
        assert_eq!(attribute_description(18), "excellent");
        assert_eq!(attribute_description(25), "godlike");
    }

    #[test]
    fn test_str_display() {
        assert_eq!(str_display(15), "15");
        assert_eq!(str_display(18), "18");
        assert_eq!(str_display(50), "18/32");
        assert_eq!(str_display(118), "18/**");
    }

    #[test]
    fn test_str_damage_bonus() {
        assert!(str_damage_bonus(3) < 0);
        assert_eq!(str_damage_bonus(10), 0);
        assert!(str_damage_bonus(18) > 0);
    }

    #[test]
    fn test_dex_ac_bonus() {
        assert!(dex_ac_bonus(3) > 0);
        assert!(dex_ac_bonus(18) < 0);
    }

    #[test]
    fn test_con_hp_bonus() {
        assert!(con_hp_bonus(3) < 0);
        assert_eq!(con_hp_bonus(10), 0);
        assert!(con_hp_bonus(18) > 0);
    }

    #[test]
    fn test_carry_capacity() {
        assert!(carry_capacity(18) > carry_capacity(10));
        assert!(carry_capacity(3) >= 100);
    }

    #[test]
    fn test_potion_gain() {
        assert_eq!(potion_attribute_gain(true, false), 2);
        assert_eq!(potion_attribute_gain(false, true), -1);
        assert_eq!(potion_attribute_gain(false, false), 1);
    }
}
