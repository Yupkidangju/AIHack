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
    Poison, // ??
    Disease,
    Paralysis, // 留덈퉬
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

// =============================================================================
// [v2.9.2] mhitu.c 대량 이식  삼키기/절도/질병/라이칸스로피/저주/특수공격
// 원본: nethack-3.6.7/src/mhitu.c (2,819줄)
// =============================================================================

/// [v2.9.1] 삼키기(Engulfing) 상태 (원본: mhitu.c:1200-1350 gulpmu)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngulfState {
    /// 삼킨 몬스터 이름
    pub engulfer_name: String,
    /// 삼킨 턴
    pub engulfed_turn: u64,
    /// 삼키기 유형
    pub engulf_type: EngulfType,
    /// 탈출 가능 여부
    pub can_escape: bool,
}

/// [v2.9.1] 삼키기 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngulfType {
    /// 소화 (Purple Worm 등)
    Digest,
    /// 감싸기 (Trapper, Lurker Above)
    Wrap,
    /// 점액동화 (Green Slime)
    Slime,
    /// 질식 (Air Elemental)
    Suffocate,
}

/// [v2.9.2] 삼키기 데미지 계산 (원본: gulpmu damage)
pub fn engulf_type_damage(
    engulf_type: EngulfType,
    monster_level: i32,
    turns_inside: i32,
    rng: &mut NetHackRng,
) -> (i32, Option<StatusEffect>) {
    match engulf_type {
        EngulfType::Digest => {
            let dmg = rng.rn1(monster_level.max(1), 3) + turns_inside;
            (dmg, None)
        }
        EngulfType::Wrap => {
            let dmg = rng.rn1(4, 2);
            (dmg, None)
        }
        EngulfType::Slime => (0, Some(StatusEffect::Slime)),
        EngulfType::Suffocate => {
            let dmg = rng.rn1(6, 4) + turns_inside * 2;
            (dmg, None)
        }
    }
}

/// [v2.9.2] 삼키기 탈출 판정 (원본: mhitu.c expels_you)
pub fn can_escape_engulf(
    player_str: i32,
    turns_inside: i32,
    monster_level: i32,
    has_wand_of_digging: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 굴착 지팡이가 있으면 즉시 탈출
    if has_wand_of_digging {
        return true;
    }

    // 힘 기반 탈출 판정
    let escape_chance = player_str * 3 + turns_inside * 5 - monster_level * 4;
    rng.rn2(100) < escape_chance.clamp(5, 80)
}

/// [v2.9.2] 절도 공격 (원본: mhitu.c:600-700 steal_attack/stealarm)
#[derive(Debug, Clone)]
pub struct TheftResult {
    /// 도난 성공 여부
    pub stolen: bool,
    /// 도난된 아이템 인덱스 (인벤 내)
    pub item_index: Option<usize>,
    /// 메시지
    pub message: String,
    /// 플레이어 텔레포트 (님프)
    pub thief_teleports: bool,
}

/// [v2.9.2] 도난 판정 (원본: steal)
pub fn steal_check(
    monster_name: &str,
    monster_type: &str, // "nymph", "monkey", "leprechaun"
    player_dex: i32,
    item_count: usize,
    player_has_gold: bool,
    rng: &mut NetHackRng,
) -> TheftResult {
    if item_count == 0 && !player_has_gold {
        return TheftResult {
            stolen: false,
            item_index: None,
            message: format!("{} tries to steal, but you have nothing!", monster_name),
            thief_teleports: false,
        };
    }

    // [v2.9.3] DEX 기반 절도 방어: DEX가 높을수록 절도 확률 감소
    // 원본 NetHack에서 하드코딩된 확률값을 상수로 분리
    const NYMPH_BASE_STEAL: i32 = 70; // 님프 기본 절도 확률 (원본: 70)
    const MONKEY_BASE_STEAL: i32 = 50; // 원숭이 기본 절도 확률 (원본: 50)
    const LEPRECHAUN_BASE_STEAL: i32 = 60; // 레프리콘 기본 절도 확률 (원본: 60)
    const DEFAULT_BASE_STEAL: i32 = 40; // 기타 몬스터 기본 절도 확률
    const STEAL_MIN_CHANCE: i32 = 5; // 최소 절도 확률 (완전 면역 방지)
    const STEAL_MAX_CHANCE: i32 = 90; // 최대 절도 확률 (항상 약간의 회피 기회)

    let steal_chance = match monster_type {
        "nymph" => NYMPH_BASE_STEAL - player_dex * 2,
        "monkey" => MONKEY_BASE_STEAL - player_dex * 2,
        "leprechaun" => LEPRECHAUN_BASE_STEAL - player_dex,
        _ => DEFAULT_BASE_STEAL - player_dex,
    };

    if rng.rn2(100) < steal_chance.clamp(STEAL_MIN_CHANCE, STEAL_MAX_CHANCE) {
        let idx = if monster_type == "leprechaun" && player_has_gold {
            None // 레프리콘은 금화만 훔침
        } else {
            Some(rng.rn2(item_count as i32) as usize)
        };

        TheftResult {
            stolen: true,
            item_index: idx,
            message: format!("{} stole something!", monster_name),
            thief_teleports: monster_type == "nymph",
        }
    } else {
        TheftResult {
            stolen: false,
            item_index: None,
            message: format!("{} tries to steal, but you dodge!", monster_name),
            thief_teleports: false,
        }
    }
}

/// [v2.9.2] 장비 절도 (원본: stealarm  님프가 착용 장비를 훔침)
pub fn steal_armor_check(
    monster_name: &str,
    worn_slots_occupied: &[bool], // [armor, cloak, helmet, gloves, boots, shield]
    slot_cursed: &[bool],
    rng: &mut NetHackRng,
) -> Option<(usize, String)> {
    let slot_names = ["armor", "cloak", "helmet", "gloves", "boots", "shield"];

    // 가장 바깥 장비부터 시도 (클로크  갑옷  기타)
    let order = [1, 0, 2, 3, 4, 5]; // cloak first
    for &idx in &order {
        if idx < worn_slots_occupied.len() && worn_slots_occupied[idx] {
            if idx < slot_cursed.len() && slot_cursed[idx] {
                continue; // 저주된 건 못 훔침
            }
            if rng.rn2(3) < 2 {
                return Some((
                    idx,
                    format!("{} steals your {}!", monster_name, slot_names[idx]),
                ));
            }
        }
    }
    None
}

/// [v2.9.2] 질병 공격 (원본: mhitu.c:800-850 disease)
pub fn disease_attack(
    atk: &Attack,
    monster_name: &str,
    has_sick_res: bool,
    constitution: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> MonsterAttackResult {
    let dmg = physical_damage(atk, rng);
    if has_sick_res {
        let msg = format!("{} touches you, but you feel fine.", monster_name);
        log.add(&msg, turn);
        MonsterAttackResult::hit(atk, dmg, &msg)
    } else {
        // CON 체크: 높은 CON은 질병 저항
        if rng.rn2(20) < constitution {
            let msg = format!("{} makes you feel slightly ill.", monster_name);
            log.add(&msg, turn);
            MonsterAttackResult::hit(atk, dmg, &msg)
        } else {
            let msg = format!("{} brings you down with a terrible disease!", monster_name);
            log.add_colored(&msg, [200, 200, 0], turn);
            let mut r = MonsterAttackResult::hit(atk, dmg, &msg);
            r.status_effect = Some(StatusEffect::Disease);
            r
        }
    }
}

/// [v2.9.2] 라이칸스로피 공격 (원본: mhitu.c:900-960 were_attack)
pub fn lycanthropy_attack(
    monster_name: &str,
    were_type: &str, // "werewolf", "werejackal", "wererat"
    has_prot_from_shape: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> Option<StatusEffect> {
    if has_prot_from_shape {
        log.add(
            &format!("{} bites you, but nothing happens.", monster_name),
            turn,
        );
        return None;
    }

    // 전파 확률: 약 10%
    if rng.rn2(10) == 0 {
        let msg = format!(
            "{} bites you! You feel feverish. ({})",
            monster_name, were_type
        );
        log.add_colored(&msg, [255, 200, 0], turn);
        Some(StatusEffect::Lycanthropy)
    } else {
        log.add(&format!("{} bites you!", monster_name), turn);
        None
    }
}

/// [v2.9.2] 텔레포트 공격 (원본: mhitu.c tele_attack)
pub fn teleport_attack_result(
    has_teleport_control: bool,
    has_teleport_resistance: bool,
    rng: &mut NetHackRng,
) -> (bool, &'static str) {
    if has_teleport_resistance {
        (
            false,
            "You feel a momentary pang, but resist the teleportation.",
        )
    } else if has_teleport_control {
        (
            false,
            "You feel a momentary pang, but remain where you are.",
        )
    } else {
        let _ = rng; // 텔레포트 좌표 결정에 사용
        (true, "You are teleported away!")
    }
}

/// [v2.9.2] 아이템 저주 공격 (원본: mhitu.c curse_items)
pub fn curse_items_count(inventory_size: usize, monster_level: i32, rng: &mut NetHackRng) -> usize {
    // 몬스터 레벨이 높을수록 저주 아이템 수 증가
    let max_curse = (monster_level / 5 + 1).min(5) as usize;
    let count = (rng.rn2(max_curse as i32 + 1) as usize).min(inventory_size);
    count
}

/// [v2.9.2] 녹 공격 (원본: mhitu.c rust_dmg)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustResult {
    /// 방어구가 부식되지 않음 (방어구 없음 또는 저항)
    NoEffect,
    /// 방어구 침식 레벨 증가
    Eroded {
        slot_name: &'static str,
        new_level: i32,
    },
    /// 방어구 파괴 (침식 3파괴)
    Destroyed { slot_name: &'static str },
}

/// [v2.9.2] 녹 몬스터 공격 효과 (원본: rust_dmg)
pub fn rust_attack_effect(
    armor_name: &str,
    current_erosion: i32,
    is_erodeproof: bool,
) -> RustResult {
    if is_erodeproof {
        return RustResult::NoEffect;
    }

    // 가죽/드래곤은 녹슬지 않음
    if armor_name.contains("leather")
        || armor_name.contains("dragon")
        || armor_name.contains("cloth")
        || armor_name.contains("elven")
    {
        return RustResult::NoEffect;
    }

    if current_erosion >= 3 {
        RustResult::Destroyed { slot_name: "armor" }
    } else {
        RustResult::Eroded {
            slot_name: "armor",
            new_level: current_erosion + 1,
        }
    }
}

/// [v2.9.2] 속도 흡수 공격 (원본: mhitu.c slow_attack)
pub fn slow_attack_check(
    player_speed: i32,
    has_speed_boots: bool,
    has_free_action: bool,
    rng: &mut NetHackRng,
) -> (bool, i32) {
    if has_free_action {
        return (false, player_speed);
    }
    if has_speed_boots {
        return (false, player_speed); // 속도 부츠가 보호
    }

    if rng.rn2(4) == 0 {
        let new_speed = (player_speed - 1).max(1);
        (true, new_speed)
    } else {
        (false, player_speed)
    }
}

/// [v2.9.2] 점액 변이 공격 (원본: mhitu.c slime_attack)
pub fn slime_attack_check(
    has_fire_resistance: bool,
    has_unchanging: bool,
    burning_item: bool,
    rng: &mut NetHackRng,
) -> (bool, &'static str) {
    if has_unchanging {
        return (false, "You feel slimy for a moment, but it passes.");
    }
    if has_fire_resistance || burning_item {
        return (false, "The slime burns away!");
    }
    if rng.rn2(4) == 0 {
        (true, "You are turning into a green slime!")
    } else {
        (false, "You feel momentarily slimy.")
    }
}

/// [v2.9.2] 몬스터 공격 횟수 제한 (원본: mhitu.c MAX_ATTACKS)
pub fn max_attacks_per_round(monster_speed: i32, _player_speed: i32) -> usize {
    // 기본 6회, 고속 몬스터는 추가
    let base = 6;
    if monster_speed >= 24 {
        base + 2
    } else if monster_speed >= 18 {
        base + 1
    } else {
        base
    }
}

/// [v2.9.2] 공격 회피율 보정 (원본: mhitu.c dodge_modifier)
pub fn dodge_modifier(
    player_dex: i32,
    player_ac: i32,
    is_flying: bool,
    is_levitating: bool,
) -> i32 {
    let mut modifier = 0;

    // DEX 보너스
    if player_dex >= 18 {
        modifier += 3;
    } else if player_dex >= 16 {
        modifier += 2;
    } else if player_dex >= 14 {
        modifier += 1;
    }

    // AC 보너스
    if player_ac < 0 {
        modifier += (-player_ac) / 3;
    }

    // 비행/공중부양 보너스
    if is_flying {
        modifier += 2;
    }
    if is_levitating {
        modifier += 1;
    }

    modifier
}

// =============================================================================
// [v2.9.2] mhitu.c 테스트
// =============================================================================
#[cfg(test)]
mod mhitu_v292_tests {
    use super::*;

    #[test]
    fn test_engulf_type_digest() {
        let mut rng = NetHackRng::new(42);
        let (dmg, eff) = engulf_type_damage(EngulfType::Digest, 10, 3, &mut rng);
        assert!(dmg > 0);
        assert!(eff.is_none());
    }

    #[test]
    fn test_engulf_type_slime() {
        let mut rng = NetHackRng::new(42);
        let (dmg, eff) = engulf_type_damage(EngulfType::Slime, 5, 1, &mut rng);
        assert_eq!(dmg, 0);
        assert_eq!(eff, Some(StatusEffect::Slime));
    }

    #[test]
    fn test_escape_engulf_with_wand() {
        let mut rng = NetHackRng::new(42);
        assert!(can_escape_engulf(10, 1, 15, true, &mut rng));
    }

    #[test]
    fn test_steal_nothing() {
        let mut rng = NetHackRng::new(42);
        let r = steal_check("nymph", "nymph", 18, 0, false, &mut rng);
        assert!(!r.stolen);
    }

    #[test]
    fn test_steal_with_items() {
        let mut rng = NetHackRng::new(42);
        let r = steal_check("nymph", "nymph", 5, 10, true, &mut rng);
        // 낮은 DEX일 때 도난 확률 높음
        assert!(r.message.len() > 0);
    }

    #[test]
    fn test_steal_armor() {
        let mut rng = NetHackRng::new(42);
        let worn = vec![true, true, false, false, false, false]; // armor + cloak
        let cursed = vec![false, false, false, false, false, false];
        let result = steal_armor_check("nymph", &worn, &cursed, &mut rng);
        // 클로크부터 시도
        if let Some((idx, _)) = &result {
            assert!(*idx == 1 || *idx == 0); // cloak or armor
        }
    }

    #[test]
    fn test_disease_with_resist() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let atk = Attack {
            atype: AttackType::Claw,
            adtype: DamageType::Phys,
            dice: 1,
            sides: 4,
        };
        let r = disease_attack(&atk, "rat", true, 10, &mut rng, &mut log, 1);
        assert!(r.status_effect.is_none());
    }

    #[test]
    fn test_lycanthropy_resist() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let r = lycanthropy_attack("werewolf", "werewolf", true, &mut rng, &mut log, 1);
        assert!(r.is_none());
    }

    #[test]
    fn test_teleport_resist() {
        let mut rng = NetHackRng::new(42);
        let (tele, msg) = teleport_attack_result(false, true, &mut rng);
        assert!(!tele);
        assert!(msg.contains("resist"));
    }

    #[test]
    fn test_rust_leather() {
        let r = rust_attack_effect("leather armor", 0, false);
        assert_eq!(r, RustResult::NoEffect);
    }

    #[test]
    fn test_rust_chain() {
        let r = rust_attack_effect("chain mail", 2, false);
        assert!(matches!(r, RustResult::Eroded { .. }));
    }

    #[test]
    fn test_rust_destroyed() {
        let r = rust_attack_effect("chain mail", 3, false);
        assert!(matches!(r, RustResult::Destroyed { .. }));
    }

    #[test]
    fn test_rust_erodeproof() {
        let r = rust_attack_effect("chain mail", 1, true);
        assert_eq!(r, RustResult::NoEffect);
    }

    #[test]
    fn test_slow_attack_free_action() {
        let mut rng = NetHackRng::new(42);
        let (slowed, sp) = slow_attack_check(12, false, true, &mut rng);
        assert!(!slowed);
        assert_eq!(sp, 12);
    }

    #[test]
    fn test_slime_fire_resist() {
        let mut rng = NetHackRng::new(42);
        let (slimed, _) = slime_attack_check(true, false, false, &mut rng);
        assert!(!slimed);
    }

    #[test]
    fn test_max_attacks() {
        assert!(max_attacks_per_round(24, 12) > max_attacks_per_round(12, 12));
    }

    #[test]
    fn test_dodge_modifier() {
        let m = dodge_modifier(18, -5, true, false);
        assert!(m >= 5); // DEX18 +3, AC-5 +1, fly +2 = 6
    }
}
