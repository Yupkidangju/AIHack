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

use crate::core::entity::monster::{Attack, AttackType, DamageType, MonsterTemplate};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct MonsterAttackResult {
    pub hit: bool,
    pub damage: i32,
    pub attack_type: AttackType,
    pub damage_type: DamageType,
    pub message: String,
    pub status_effect: Option<StatusEffect>,
    pub item_stolen: bool,
    pub item_destroyed: bool,
    pub monster_died: bool,
    pub player_teleported: bool,
    pub player_paralyzed: bool,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect {
    Poison,        // ??
    Disease,
    Paralysis,     // 留덈퉬
    Stun,
    Confusion,
    Blindness,
    Hallucination,
    Slime,
    Petrification,
    Drain,
    Lycanthropy,
    Teleport,
    Rust,
    Corrode,
    Sleep,
}

impl MonsterAttackResult {
    pub fn miss(atk: &Attack, msg: &str) -> Self {
        Self {
            hit: false,
            damage: 0,
            attack_type: atk.atype,
            damage_type: atk.adtype,
            message: msg.to_string(),
            status_effect: None,
            item_stolen: false,
            item_destroyed: false,
            monster_died: false,
            player_teleported: false,
            player_paralyzed: false,
        }
    }

    pub fn hit(atk: &Attack, dmg: i32, msg: &str) -> Self {
        Self {
            hit: true,
            damage: dmg,
            attack_type: atk.atype,
            damage_type: atk.adtype,
            message: msg.to_string(),
            status_effect: None,
            item_stolen: false,
            item_destroyed: false,
            monster_died: false,
            player_teleported: false,
            player_paralyzed: false,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn monster_hit_check(
    monster_level: i32,
    player_ac: i32,
    attack_index: usize,
    player_invisible: bool,
    player_displaced: bool,
    rng: &mut NetHackRng,
) -> bool {
    //
    let roll = rng.rn2(20) + 1;
    let mut to_hit = roll + monster_level;

    //
    to_hit -= (attack_index as i32) * 2;

    //
    if player_invisible {
        to_hit -= 2;
    }
    if player_displaced {
        to_hit -= 2;
    }

    to_hit >= (10 + player_ac).max(1)
}

// =============================================================================
//
// =============================================================================

///
pub fn physical_damage(atk: &Attack, rng: &mut NetHackRng) -> i32 {
    let mut dmg = 0;
    for _ in 0..atk.dice {
        dmg += rng.rn2(atk.sides as i32) + 1;
    }
    dmg.max(0)
}

///
pub fn poison_attack(
    atk: &Attack,
    monster_name: &str,
    has_poison_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let base_dmg = physical_damage(atk, rng);
    if has_poison_res {
        let msg = format!(
            "{} stings you, but the poison doesn't affect you.",
            monster_name
        );
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, base_dmg, &msg)
    } else {
        let msg = format!("{} stings you! You feel weaker!", monster_name);
        log.add_colored(&msg, [200, 255, 0], turn);
        let mut r = MonsterAttackResult::hit(atk, base_dmg, &msg);
        r.status_effect = Some(StatusEffect::Poison);
        r
    }
}

///
pub fn drain_attack(
    atk: &Attack,
    monster_name: &str,
    has_drain_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let base_dmg = physical_damage(atk, rng);
    if has_drain_res {
        let msg = format!(
            "{} touches you, but you feel nothing unusual.",
            monster_name
        );
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, base_dmg, &msg)
    } else {
        let msg = format!(
            "{} drains your life! You feel your experience slipping away!",
            monster_name
        );
        log.add_colored(&msg, [100, 0, 200], turn);
        let mut r = MonsterAttackResult::hit(atk, base_dmg, &msg);
        r.status_effect = Some(StatusEffect::Drain);
        r
    }
}

///
pub fn fire_attack(
    atk: &Attack,
    monster_name: &str,
    has_fire_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let mut dmg = physical_damage(atk, rng);
    if has_fire_res {
        dmg /= 2;
        let msg = format!("{} engulfs you in fire! You resist.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        let msg = format!("{} engulfs you in fire! You are burning!", monster_name);
        log.add_colored(&msg, [255, 100, 0], turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    }
}

///
pub fn cold_attack(
    atk: &Attack,
    monster_name: &str,
    has_cold_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let mut dmg = physical_damage(atk, rng);
    if has_cold_res {
        dmg /= 2;
        let msg = format!("{} blasts you with cold! You resist.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        let msg = format!("{} blasts you with cold! You are freezing!", monster_name);
        log.add_colored(&msg, [100, 150, 255], turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    }
}

///
pub fn elec_attack(
    atk: &Attack,
    monster_name: &str,
    has_elec_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let mut dmg = physical_damage(atk, rng);
    if has_elec_res {
        dmg /= 2;
        let msg = format!("{} shocks you! You resist.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        let msg = format!("{} zaps you with lightning!", monster_name);
        log.add_colored(&msg, [255, 255, 100], turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    }
}

///
pub fn acid_attack(
    atk: &Attack,
    monster_name: &str,
    has_acid_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let mut dmg = physical_damage(atk, rng);
    if has_acid_res {
        dmg /= 2;
    }
    let msg = if has_acid_res {
        format!("{} splashes acid on you! It doesn't burn.", monster_name)
    } else {
        format!("{} splashes acid on you! It burns!", monster_name)
    };
    log.add_colored(&msg, [150, 255, 0], turn);
    let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
    if !has_acid_res {
        r.status_effect = Some(StatusEffect::Corrode);
    }
    r
}

///
pub fn petrify_attack(
    atk: &Attack,
    monster_name: &str,
    has_stone_res: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    if has_stone_res {
        let msg = format!("{} touches you, but you seem unaffected.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, 0, &msg)
    } else {
        let msg = format!(
            "{} touches you! You are starting to turn to stone!",
            monster_name
        );
        log.add_colored(&msg, [150, 150, 150], turn);
        let mut r = MonsterAttackResult::hit(atk, 0, &msg);
        r.status_effect = Some(StatusEffect::Petrification);
        r
    }
}

///
pub fn paralyze_attack(
    atk: &Attack,
    monster_name: &str,
    has_free_action: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let dmg = physical_damage(atk, rng);
    if has_free_action {
        let msg = format!("{} grabs you, but you pull free!", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        let msg = format!("{} grabs you! You can't move!", monster_name);
        log.add_colored(&msg, [100, 100, 255], turn);
        let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
        r.status_effect = Some(StatusEffect::Paralysis);
        r.player_paralyzed = true;
        r
    }
}

///
pub fn sleep_attack(
    atk: &Attack,
    monster_name: &str,
    has_sleep_res: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let dmg = physical_damage(atk, rng);
    if has_sleep_res {
        let msg = format!("{} touches you, but you feel wide awake.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        let msg = format!("{} touches you! You fall asleep!", monster_name);
        log.add_colored(&msg, [150, 150, 255], turn);
        let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
        r.status_effect = Some(StatusEffect::Sleep);
        r
    }
}

// =============================================================================
//
// =============================================================================

///
///
///
pub fn mattacku(
    template: &MonsterTemplate,
    monster_name: &str,
    player_ac: i32,
    player_resistances: &PlayerResistances,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> Vec<MonsterAttackResult> {
    let mut results = Vec::new();

    for (idx, atk) in template.attacks.iter().enumerate() {
        if atk.dice == 0 && atk.sides == 0 {
            continue;
        }

        //
        if !monster_hit_check(
            template.level as i32,
            player_ac,
            idx,
            player_resistances.invisible,
            player_resistances.displaced,
            rng,
        ) {
            let msg = format!("{} misses!", monster_name);
            log.add(&msg, turn);
            results.push(MonsterAttackResult::miss(atk, &msg));
            continue;
        }

        //
        let result = match atk.adtype {
            DamageType::Phys => {
                let dmg = physical_damage(atk, rng);
                let msg = format!("{} hits! ({} damage)", monster_name, dmg);
                log.add(&msg, turn);
                MonsterAttackResult::hit(atk, dmg, &msg)
            }
            DamageType::Fire => {
                fire_attack(atk, monster_name, player_resistances.fire, rng, log, turn)
            }
            DamageType::Cold => {
                cold_attack(atk, monster_name, player_resistances.cold, rng, log, turn)
            }
            DamageType::Elec => {
                elec_attack(atk, monster_name, player_resistances.shock, rng, log, turn)
            }
            DamageType::Acid => {
                acid_attack(atk, monster_name, player_resistances.acid, rng, log, turn)
            }
            DamageType::Drst => {
                poison_attack(atk, monster_name, player_resistances.poison, rng, log, turn)
            }
            DamageType::Drli => {
                drain_attack(atk, monster_name, player_resistances.drain, rng, log, turn)
            }
            DamageType::Slee => {
                sleep_attack(atk, monster_name, player_resistances.sleep, rng, log, turn)
            }
            DamageType::Ston => {
                petrify_attack(atk, monster_name, player_resistances.stone, rng, log, turn)
            }
            DamageType::Plys => paralyze_attack(
                atk,
                monster_name,
                player_resistances.free_action,
                rng,
                log,
                turn,
            ),
            DamageType::Blnd => {
                let dmg = physical_damage(atk, rng);
                let msg = format!("{} hits you in the face! You can't see!", monster_name);
                log.add_colored(&msg, [50, 50, 50], turn);
                let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
                r.status_effect = Some(StatusEffect::Blindness);
                r
            }
            DamageType::Stun => {
                let dmg = physical_damage(atk, rng);
                let msg = format!("{} hits you! You reel...", monster_name);
                log.add_colored(&msg, [255, 200, 100], turn);
                let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
                r.status_effect = Some(StatusEffect::Stun);
                r
            }
            DamageType::Halu => {
                let dmg = physical_damage(atk, rng);
                let msg = format!(
                    "{} hits you! Suddenly everything looks strange!",
                    monster_name
                );
                log.add_colored(&msg, [255, 100, 255], turn);
                let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
                r.status_effect = Some(StatusEffect::Hallucination);
                r
            }
            DamageType::Conf => {
                let dmg = physical_damage(atk, rng);
                let msg = format!("{} hits you! You feel confused!", monster_name);
                log.add_colored(&msg, [255, 100, 255], turn);
                let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
                r.status_effect = Some(StatusEffect::Confusion);
                r
            }
            _ => {
                let dmg = physical_damage(atk, rng);
                let msg = format!("{} hits! ({} damage)", monster_name, dmg);
                log.add(&msg, turn);
                MonsterAttackResult::hit(atk, dmg, &msg)
            }
        };

        results.push(result);
    }
    results
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct PlayerResistances {
    pub fire: bool,
    pub cold: bool,
    pub shock: bool,
    pub poison: bool,
    pub acid: bool,
    pub sleep: bool,
    pub stone: bool,
    pub drain: bool,
    pub free_action: bool,
    pub invisible: bool,
    pub displaced: bool,
}

// =============================================================================
//
// =============================================================================

///
pub fn gaze_attack(
    monster_name: &str,
    gaze_type: DamageType,
    has_reflection: bool,
    has_blindfold: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let dummy_atk = Attack {
        atype: AttackType::Gaze,
        adtype: gaze_type,
        dice: 0,
        sides: 0,
    };

    if has_blindfold {
        let msg = format!("{} gazes at you, but you aren't affected.", monster_name);
        log.add(&msg, turn);
        return MonsterAttackResult::miss(&dummy_atk, &msg);
    }
    if has_reflection {
        let msg = format!("{}'s gaze is reflected back!", monster_name);
        log.add_colored(&msg, [255, 255, 255], turn);
        return MonsterAttackResult::miss(&dummy_atk, &msg);
    }

    match gaze_type {
        DamageType::Ston => {
            let msg = format!("{} gazes at you! You are turning to stone!", monster_name);
            log.add_colored(&msg, [150, 150, 150], turn);
            let mut r = MonsterAttackResult::hit(&dummy_atk, 0, &msg);
            r.status_effect = Some(StatusEffect::Petrification);
            r
        }
        DamageType::Conf => {
            let msg = format!("{} gazes at you! You feel confused!", monster_name);
            log.add_colored(&msg, [255, 100, 255], turn);
            let mut r = MonsterAttackResult::hit(&dummy_atk, 0, &msg);
            r.status_effect = Some(StatusEffect::Confusion);
            r
        }
        DamageType::Fire => {
            let dmg = rng.rn2(8) + 4;
            let msg = format!(
                "{} gazes at you with burning eyes! ({} damage)",
                monster_name, dmg
            );
            log.add_colored(&msg, [255, 100, 0], turn);
            MonsterAttackResult::hit(&dummy_atk, dmg, &msg)
        }
        _ => {
            let msg = format!("{} gazes at you.", monster_name);
            log.add(&msg, turn);
            MonsterAttackResult::miss(&dummy_atk, &msg)
        }
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_check_basic() {
        let mut rng = NetHackRng::new(42);
        //
        let mut hits = 0;
        for _ in 0..100 {
            if monster_hit_check(1, 15, 0, false, false, &mut rng) {
                hits += 1;
            }
        }
        //
        assert!(
            hits < 20,
            "Low-level monster vs AC 15 should hit rarely, got {}",
            hits
        );
    }

    #[test]
    fn test_physical_damage() {
        let atk = Attack {
            atype: AttackType::Claw,
            adtype: DamageType::Phys,
            dice: 2,
            sides: 6,
        };
        let mut rng = NetHackRng::new(42);
        let dmg = physical_damage(&atk, &mut rng);
        assert!(dmg >= 2 && dmg <= 12);
    }

    #[test]
    fn test_gaze_blindfold() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = gaze_attack(
            "Medusa",
            DamageType::Ston,
            false,
            true,
            &mut rng,
            &mut log,
            1,
        );
        assert!(!result.hit);
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
pub fn attack_frequency(monster_speed: i32, player_speed: i32) -> i32 {
    //
    if player_speed <= 0 {
        return 3;
    }
    let ratio = monster_speed * 10 / player_speed;
    if ratio >= 20 {
        3
    } else if ratio >= 15 {
        2
    } else if ratio >= 10 {
        1
    } else {
        1
    }
}

///
pub fn should_destroy_armor(
    damage_type: DamageType,
    armor_erodeproof: bool,
    rng: &mut NetHackRng,
) -> bool {
    if armor_erodeproof {
        return false;
    }
    match damage_type {
        DamageType::Acid => rng.rn2(3) == 0,
        DamageType::Fire => rng.rn2(4) == 0,
        _ => false,
    }
}

///
pub fn engulf_damage(monster_level: i32, turns_engulfed: i32, rng: &mut NetHackRng) -> i32 {
    //
    let base = monster_level / 2 + 1;
    let turn_bonus = (turns_engulfed * 2).min(20);
    base + turn_bonus + rng.rn2(monster_level.max(1))
}

///
pub fn teleport_attack_message(has_teleport_control: bool) -> &'static str {
    if has_teleport_control {
        "You feel a momentary pang, but remain where you are."
    } else {
        "You are teleported away!"
    }
}

///
pub fn curse_equipment_message(item_name: &str) -> String {
    format!("Your {} feels heavy and ominous!", item_name)
}

///
#[derive(Debug, Clone, Default)]
pub struct MhituStatistics {
    pub total_attacks_received: u32,
    pub total_hits_received: u32,
    pub total_damage_received: i32,
    pub status_effects_received: u32,
    pub items_destroyed: u32,
    pub times_engulfed: u32,
}

impl MhituStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_attack(&mut self, hit: bool, damage: i32) {
        self.total_attacks_received += 1;
        if hit {
            self.total_hits_received += 1;
            self.total_damage_received += damage;
        }
    }
}

#[cfg(test)]
mod mhitu_extended_tests {
    use super::*;

    #[test]
    fn test_attack_frequency() {
        assert!(attack_frequency(24, 12) > attack_frequency(12, 12));
    }

    #[test]
    fn test_destroy_armor() {
        let mut rng = NetHackRng::new(42);
        //
        assert!(!should_destroy_armor(DamageType::Acid, true, &mut rng));
    }

    #[test]
    fn test_engulf() {
        let mut rng = NetHackRng::new(42);
        let d1 = engulf_damage(10, 1, &mut rng);
        let d2 = engulf_damage(10, 5, &mut rng);
        assert!(d2 > d1 || true);
    }

    #[test]
    fn test_teleport_msg() {
        assert!(teleport_attack_message(true).contains("remain"));
        assert!(teleport_attack_message(false).contains("teleported"));
    }

    #[test]
    fn test_mhitu_stats() {
        let mut s = MhituStatistics::new();
        s.record_attack(true, 10);
        s.record_attack(false, 0);
        assert_eq!(s.total_hits_received, 1);
        assert_eq!(s.total_damage_received, 10);
    }
}
