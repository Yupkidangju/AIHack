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

use crate::core::entity::capability::MonsterCapability;
use crate::core::entity::monster::MonsterTemplate;
use crate::core::entity::object::{ItemClass, ItemTemplate, Material};
use crate::core::entity::player::{Player, PlayerClass};
use crate::core::events::GameEvent;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct PlayerAttackResult {
    pub hit: bool,
    pub damage: i32,
    pub critical: bool,
    pub message: String,
    pub monster_killed: bool,
    pub monster_fled: bool,
    pub weapon_broke: bool,
    pub weapon_eroded: bool,
    pub player_took_damage: i32,
    pub silver_bonus: bool,
    pub artifact_invoked: bool,
}

impl PlayerAttackResult {
    pub fn miss(msg: &str) -> Self {
        Self {
            hit: false,
            damage: 0,
            critical: false,
            message: msg.to_string(),
            monster_killed: false,
            monster_fled: false,
            weapon_broke: false,
            weapon_eroded: false,
            player_took_damage: 0,
            silver_bonus: false,
            artifact_invoked: false,
        }
    }

    pub fn hit(dmg: i32, msg: &str) -> Self {
        Self {
            hit: true,
            damage: dmg,
            critical: false,
            message: msg.to_string(),
            monster_killed: false,
            monster_fled: false,
            weapon_broke: false,
            weapon_eroded: false,
            player_took_damage: 0,
            silver_bonus: false,
            artifact_invoked: false,
        }
    }

    ///
    ///
    ///
    ///
    pub fn to_events(&self, target_name: &str, weapon_name: &str) -> Vec<GameEvent> {
        let mut events = Vec::new();

        if self.hit {
            //
            events.push(GameEvent::DamageDealt {
                attacker: "player".to_string(),
                defender: target_name.to_string(),
                amount: self.damage,
                source: if weapon_name.is_empty() {
                    "unarmed".to_string()
                } else {
                    weapon_name.to_string()
                },
            });
        } else {
            //
            events.push(GameEvent::AttackMissed {
                attacker: "player".to_string(),
                defender: target_name.to_string(),
            });
        }

        events
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn find_roll_to_hit(
    player: &Player,
    _weapon: Option<&ItemTemplate>,
    weapon_spe: i8,
    _target_ac: i32,
    is_thrown: bool,
    is_ranged: bool,
) -> i32 {
    let mut to_hit: i32 = 1;

    //
    to_hit += player.level;

    //
    to_hit += str_bonus(player.str.base);

    //
    to_hit += dex_bonus(player.dex.base);

    //
    to_hit += weapon_spe as i32;

    //
    match player.role {
        PlayerClass::Barbarian
        | PlayerClass::Knight
        | PlayerClass::Samurai
        | PlayerClass::Valkyrie => to_hit += 2,
        PlayerClass::Ranger => {
            if is_ranged {
                to_hit += 3;
            }
        }
        PlayerClass::Rogue => to_hit += 1,
        _ => {}
    }

    //
    if is_thrown {
        to_hit -= 2;
    }

    to_hit
}

///
pub fn player_hit_check(to_hit: i32, target_ac: i32, rng: &mut NetHackRng) -> bool {
    let roll = rng.rn2(20) + 1;
    roll + to_hit >= (10 + target_ac).max(1)
}

// =============================================================================
//
// =============================================================================

///
///
pub fn weapon_damage(
    template: &ItemTemplate,
    spe: i8,
    target_large: bool,
    rng: &mut NetHackRng,
) -> i32 {
    //
    let sides = if target_large {
        template.wldam as i32
    } else {
        template.wsdam as i32
    };

    //
    let dmg = rng.rn2(sides.max(1)) + 1;

    //
    (dmg + spe as i32).max(1)
}

///
pub fn barehanded_damage(player: &Player, rng: &mut NetHackRng) -> i32 {
    let base = match player.role {
        PlayerClass::Monk => {
            //
            let dice = 1 + player.level / 4;
            let sides = 4 + player.level / 3;
            let mut dmg = 0;
            for _ in 0..dice {
                dmg += rng.rn2(sides) + 1;
            }
            dmg
        }
        _ => {
            //
            rng.rn2(2) + 1
        }
    };

    //
    base + str_damage_bonus(player.str.base)
}

///
fn str_bonus(str_val: i32) -> i32 {
    match str_val {
        0..=3 => -3,
        4..=5 => -2,
        6..=7 => -1,
        8..=15 => 0,
        16 => 1,
        17 => 2,
        18 => 3,
        _ => (str_val - 18) / 3 + 3,
    }
}

///
fn dex_bonus(dex_val: i32) -> i32 {
    match dex_val {
        0..=3 => -3,
        4..=5 => -2,
        6..=7 => -1,
        8..=14 => 0,
        15 => 1,
        16..=17 => 2,
        18 => 3,
        _ => (dex_val - 18) / 3 + 3,
    }
}

///
fn str_damage_bonus(str_val: i32) -> i32 {
    match str_val {
        0..=5 => -1,
        6..=15 => 0,
        16 => 1,
        17 => 3,
        18 => 5,
        _ => (str_val - 18) + 5,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn silver_bonus(weapon_material: Material, target: &MonsterTemplate) -> i32 {
    if weapon_material != Material::Silver {
        return 0;
    }
    //
    let flags2 = target.flags2;
    let is_demon = flags2 & 0x00000100 != 0;
    let is_undead = flags2 & 0x00000002 != 0;
    let is_were = flags2 & 0x00000004 != 0;
    if is_demon || is_undead || is_were {
        20
    } else {
        0
    }
}

///
pub fn blessed_bonus(blessed: bool, target: &MonsterTemplate) -> i32 {
    if !blessed {
        return 0;
    }
    let flags2 = target.flags2;
    let is_demon = flags2 & 0x00000100 != 0;
    let is_undead = flags2 & 0x00000002 != 0;
    if is_demon || is_undead {
        4
    } else {
        0
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn player_attack(
    player: &Player,
    target: &MonsterTemplate,
    target_hp: i32,
    target_ac: i32,
    weapon: Option<(&ItemTemplate, i8, bool, Material)>,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> PlayerAttackResult {
    let (weapon_tmpl, weapon_spe, weapon_blessed, weapon_mat) = match weapon {
        Some((t, s, b, m)) => (Some(t), s, b, m),
        None => (None, 0, false, Material::Flesh),
    };

    //
    let to_hit = find_roll_to_hit(player, weapon_tmpl, weapon_spe, target_ac, false, false);
    if !player_hit_check(to_hit, target_ac, rng) {
        let msg = format!("You miss {}.", target.name);
        log.add(&msg, turn);
        return PlayerAttackResult::miss(&msg);
    }

    //
    let base_dmg = if let Some(tmpl) = weapon_tmpl {
        let is_large = target.msize >= 4;
        weapon_damage(tmpl, weapon_spe, is_large, rng)
    } else {
        barehanded_damage(player, rng)
    };

    //
    let silver = silver_bonus(weapon_mat, target);
    let blessed = blessed_bonus(weapon_blessed, target);
    let str_dmg = str_damage_bonus(player.str.base);

    let total_dmg = (base_dmg + silver + blessed + str_dmg).max(1);

    //
    let critical = rng.rn2(20) == 0;
    let final_dmg = if critical { total_dmg * 2 } else { total_dmg };

    //
    let killed = final_dmg >= target_hp;

    //
    let msg = if killed {
        format!(
            "You {} {}!",
            if critical {
                "critically hit and destroy"
            } else {
                "destroy"
            },
            target.name
        )
    } else if critical {
        format!(
            "Critical hit! You {} {} for {} damage!",
            attack_verb(weapon_tmpl),
            target.name,
            final_dmg
        )
    } else {
        format!(
            "You {} {} for {} damage.",
            attack_verb(weapon_tmpl),
            target.name,
            final_dmg
        )
    };

    if silver > 0 {
        log.add_colored(
            &format!("Your silver weapon sears {}!", target.name),
            [200, 200, 255],
            turn,
        );
    }
    log.add(&msg, turn);

    let mut result = PlayerAttackResult::hit(final_dmg, &msg);
    result.critical = critical;
    result.monster_killed = killed;
    result.silver_bonus = silver > 0;

    //
    if target.has_capability(MonsterCapability::Acid) {
        if weapon_tmpl.is_some() && weapon_mat != Material::Glass && weapon_mat != Material::Mineral
        {
            if rng.rn2(3) == 0 {
                log.add("Your weapon is corroded by acid!", turn);
                result.weapon_eroded = true;
            }
        }
    }

    result
}

///
fn attack_verb(weapon: Option<&ItemTemplate>) -> &'static str {
    match weapon {
        None => "hit",
        Some(tmpl) => match tmpl.class {
            ItemClass::Weapon => {
                if tmpl.name.contains("sword") || tmpl.name.contains("katana") {
                    "slash"
                } else if tmpl.name.contains("axe") || tmpl.name.contains("mattock") {
                    "cleave"
                } else if tmpl.name.contains("mace") || tmpl.name.contains("hammer") {
                    "smash"
                } else if tmpl.name.contains("dagger") || tmpl.name.contains("knife") {
                    "stab"
                } else if tmpl.name.contains("spear") || tmpl.name.contains("lance") {
                    "thrust"
                } else if tmpl.name.contains("whip") {
                    "lash"
                } else {
                    "hit"
                }
            }
            _ => "hit",
        },
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn passive_damage(
    target: &MonsterTemplate,
    player_has_gloves: bool,
    weapon_used: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> i32 {
    //
    if target.has_capability(MonsterCapability::Acid) {
        if !weapon_used && !player_has_gloves {
            let dmg = rng.rn2(4) + 1;
            log.add_colored(
                &format!(
                    "{} is covered in acid! ({} damage to you)",
                    target.name, dmg
                ),
                [150, 255, 0],
                turn,
            );
            return dmg;
        }
    }

    //
    if target.resists_elec() {
        if !weapon_used {
            let dmg = rng.rn2(6) + 1;
            log.add_colored(
                &format!("You get zapped by {}! ({} damage)", target.name, dmg),
                [255, 255, 100],
                turn,
            );
            return dmg;
        }
    }

    0
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_bonus() {
        assert!(str_bonus(18) > str_bonus(10));
        assert!(str_bonus(3) < 0);
    }

    #[test]
    fn test_dex_bonus() {
        assert!(dex_bonus(18) > dex_bonus(10));
        assert!(dex_bonus(3) < 0);
    }

    #[test]
    fn test_str_damage() {
        assert!(str_damage_bonus(18) > str_damage_bonus(10));
    }

    #[test]
    fn test_attack_verb() {
        assert_eq!(attack_verb(None), "hit");
    }

    #[test]
    fn test_silver_no_bonus_on_human() {
        let tmpl = MonsterTemplate {
            name: "human".into(),
            symbol: '@',
            level: 1,
            movement: 12,
            ac: 10,
            mr: 0,
            alignment: 0,
            geno: 0,
            weight: 150,
            nutrition: 400,
            msound: 0,
            msize: 2,
            resists: 0,
            conveys: 0,
            attacks: vec![],
            flags1: 0,
            flags2: 0,
            flags3: 0,
            difficulty: 1,
            color: 0,
        };
        assert_eq!(silver_bonus(Material::Silver, &tmpl), 0);
    }
}
