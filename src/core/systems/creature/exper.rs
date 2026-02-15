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
use crate::core::entity::player::{Player, PlayerClass};
use crate::core::systems::role::{
    exp_for_level, get_role_data, level_from_exp, levelup_energy, levelup_hp,
};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
///
pub fn experience(template: &MonsterTemplate, _player_level: i32) -> u64 {
    let mlev = template.level as i64;
    let diff = template.difficulty as i64;

    //
    let mut tmp = 1 + mlev * mlev;

    //
    for atk in &template.attacks {
        if atk.dice > 0 {
            tmp += atk.dice as i64 * atk.sides as i64;
        }
    }

    //
    if diff > 0 {
        tmp += diff * diff;
    }

    //
    if template.mr > 0 {
        tmp += template.mr as i64;
    }

    // 理쒖냼 1
    tmp.max(1) as u64
}

///
pub fn gain_experience(
    player: &mut Player,
    exp_gained: u64,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) {
    let old_level = player.exp_level;
    player.experience += exp_gained;

    //
    let new_level = level_from_exp(player.experience);
    if new_level > old_level {
        for lv in (old_level + 1)..=new_level {
            gain_level(player, lv, rng, log, turn);
        }
    }
}

// =============================================================================
//
// =============================================================================

///
fn gain_level(
    player: &mut Player,
    new_level: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) {
    player.exp_level = new_level;

    //
    let hp_gain = calculate_hp_gain(player.role, player.race, new_level, rng);
    player.hp_max += hp_gain;
    player.hp += hp_gain;

    //
    let en_gain = calculate_en_gain(player.role, player.race, new_level, rng);
    player.energy_max += en_gain;
    player.energy += en_gain;

    log.add_colored(
        &format!("Welcome to experience level {}!", new_level),
        [255, 255, 100],
        turn,
    );

    //
    match player.role {
        PlayerClass::Monk => {
            //
            if new_level % 2 == 0 {
                player.ac -= 1;
                log.add("Your skills improve!", turn);
            }
        }
        PlayerClass::Wizard => {
            //
            if new_level == 12 || new_level == 18 {
                player.energy_max += 5;
                player.energy += 5;
                log.add("You feel a surge of magical energy!", turn);
            }
        }
        _ => {}
    }
}

///
fn calculate_hp_gain(
    role: PlayerClass,
    race: crate::core::entity::player::Race,
    level: i32,
    rng: &mut NetHackRng,
) -> i32 {
    let base = levelup_hp(role, race, level);
    //
    let rd = get_role_data(role);
    let var = rd.hp_adv.level_var;
    let random_bonus = if var > 0 { rng.rn2(var + 1) } else { 0 };
    (base + random_bonus).max(1)
}

///
fn calculate_en_gain(
    role: PlayerClass,
    race: crate::core::entity::player::Race,
    level: i32,
    rng: &mut NetHackRng,
) -> i32 {
    let base = levelup_energy(role, race, level);
    let rd = get_role_data(role);
    let var = rd.en_adv.level_var;
    let random_bonus = if var > 0 { rng.rn2(var + 1) } else { 0 };
    (base + random_bonus).max(0)
}

// =============================================================================
//
// =============================================================================

///
pub fn lose_level(player: &mut Player, rng: &mut NetHackRng, log: &mut GameLog, turn: u64) {
    if player.exp_level <= 1 {
        //
        log.add_colored("You die...", [255, 0, 0], turn);
        player.hp = 0;
        return;
    }

    let old_level = player.exp_level;
    player.exp_level -= 1;

    //
    let hp_loss = calculate_hp_gain(player.role, player.race, old_level, rng);
    player.hp_max = (player.hp_max - hp_loss).max(1);
    if player.hp > player.hp_max {
        player.hp = player.hp_max;
    }

    //
    let en_loss = calculate_en_gain(player.role, player.race, old_level, rng);
    player.energy_max = (player.energy_max - en_loss).max(0);
    if player.energy > player.energy_max {
        player.energy = player.energy_max;
    }

    //
    let min_exp = exp_for_level(player.exp_level);
    if player.experience > min_exp {
        player.experience = min_exp;
    }

    log.add_colored(
        &format!("Goodbye level {}.", old_level),
        [200, 100, 100],
        turn,
    );
}

///
pub fn lose_experience(
    player: &mut Player,
    amount: u64,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) {
    if player.experience < amount {
        player.experience = 0;
    } else {
        player.experience -= amount;
    }

    //
    let new_level = level_from_exp(player.experience);
    while player.exp_level > new_level && player.exp_level > 1 {
        lose_level(player, rng, log, turn);
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn exp_to_next_level(player: &Player) -> u64 {
    let next_level = player.exp_level + 1;
    let needed = exp_for_level(next_level);
    if player.experience >= needed {
        0
    } else {
        needed - player.experience
    }
}

///
pub fn exp_ratio(player: &Player) -> f32 {
    let current_threshold = exp_for_level(player.exp_level);
    let next_threshold = exp_for_level(player.exp_level + 1);
    if next_threshold <= current_threshold {
        return 1.0;
    }
    let progress = player.experience.saturating_sub(current_threshold) as f32;
    let range = (next_threshold - current_threshold) as f32;
    (progress / range).min(1.0)
}

///
pub fn adjusted_experience(
    base_exp: u64,
    player_luck: i32,
    _player_level: i32,
    _monster_level: i32,
) -> u64 {
    //
    let luck_mult = if player_luck > 0 {
        1.0 + (player_luck as f64 * 0.02)
    } else if player_luck < 0 {
        1.0 + (player_luck as f64 * 0.03)
    } else {
        1.0
    };

    ((base_exp as f64) * luck_mult).max(1.0) as u64
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_calculation() {
        let tmpl = MonsterTemplate {
            name: "orc".into(),
            symbol: 'o',
            level: 3,
            movement: 9,
            ac: 6,
            mr: 0,
            alignment: -3,
            geno: 0,
            weight: 850,
            nutrition: 150,
            msound: 0,
            msize: 2,
            resists: 0,
            conveys: 0,
            attacks: vec![],
            flags1: 0,
            flags2: 0,
            flags3: 0,
            difficulty: 4,
            color: 0,
        };
        let exp = experience(&tmpl, 1);
        assert!(exp > 0);
    }

    #[test]
    fn test_exp_to_next() {
        let player = Player::new();
        let remaining = exp_to_next_level(&player);
        assert_eq!(remaining, 20);
    }

    #[test]
    fn test_exp_ratio() {
        let mut player = Player::new();
        assert_eq!(exp_ratio(&player), 0.0);
        player.experience = 10;
        assert!(exp_ratio(&player) > 0.0);
        assert!(exp_ratio(&player) < 1.0);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn boss_kill_bonus(template: &MonsterTemplate, player_level: i32) -> u64 {
    let is_unique = template.flags2 & 0x00004000 != 0; // MF2_UNIQUE
    let is_boss = template.level >= 20;

    let base = experience(template, player_level);

    if is_unique {
        base * 5
    } else if is_boss {
        base * 3
    } else {
        base
    }
}

///
pub fn kill_streak_modifier(consecutive_same_type: u32) -> f64 {
    //
    match consecutive_same_type {
        0..=2 => 1.0,
        3..=5 => 0.8,
        6..=10 => 0.5,
        11..=20 => 0.25,
        _ => 0.1,
    }
}

///
pub fn streak_adjusted_exp(base_exp: u64, streak: u32) -> u64 {
    let modifier = kill_streak_modifier(streak);
    ((base_exp as f64) * modifier).max(1.0) as u64
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn elite_monster_exp(
    template: &MonsterTemplate,
    hp_bonus: i32,
    damage_bonus: i32,
    player_level: i32,
) -> u64 {
    let base = experience(template, player_level);
    let elite_mult = 1.0 + (hp_bonus as f64 / 100.0) + (damage_bonus as f64 / 50.0);
    ((base as f64) * elite_mult).max(1.0) as u64
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpPenaltyReason {
    FriendlyFire,
    Genocide,
    Prayer,
    ArtifactBlast,
    Indirect,
}

///
pub fn penalized_experience(base_exp: u64, reason: ExpPenaltyReason) -> u64 {
    let penalty = match reason {
        ExpPenaltyReason::FriendlyFire => 0.0,
        ExpPenaltyReason::Genocide => 0.1,
        ExpPenaltyReason::Prayer => 0.5,
        ExpPenaltyReason::ArtifactBlast => 0.7,
        ExpPenaltyReason::Indirect => 0.3,
    };
    ((base_exp as f64) * penalty).max(0.0) as u64
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct LevelStats {
    pub monsters_killed: u32,
    pub total_exp_gained: u64,
    pub total_exp_lost: u64,
    pub levels_gained: u32,
    pub levels_lost: u32,
    pub max_level_reached: i32,
}

impl LevelStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_kill(&mut self, exp: u64) {
        self.monsters_killed += 1;
        self.total_exp_gained += exp;
    }

    pub fn record_level_gain(&mut self, new_level: i32) {
        self.levels_gained += 1;
        if new_level > self.max_level_reached {
            self.max_level_reached = new_level;
        }
    }

    pub fn record_level_loss(&mut self, exp_lost: u64) {
        self.levels_lost += 1;
        self.total_exp_lost += exp_lost;
    }
}

///
pub fn level_up_reward(level: i32) -> &'static str {
    match level {
        5 => "You feel more experienced.",
        10 => "You feel powerful!",
        14 => "Your skills sharpen significantly!",
        20 => "You feel like a master!",
        25 => "You feel godlike!",
        30 => "You have reached the pinnacle of experience!",
        _ => "",
    }
}

///
pub fn expected_max_hp(
    role: PlayerClass,
    race: crate::core::entity::player::Race,
    level: i32,
) -> i32 {
    let mut total = 0;
    for lv in 1..=level {
        total += levelup_hp(role, race, lv);
    }
    total
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod exper_extended_tests {
    use super::*;

    #[test]
    fn test_kill_streak_modifier() {
        assert_eq!(kill_streak_modifier(0), 1.0);
        assert_eq!(kill_streak_modifier(5), 0.8);
        assert_eq!(kill_streak_modifier(25), 0.1);
    }

    #[test]
    fn test_streak_adjusted_exp() {
        assert_eq!(streak_adjusted_exp(100, 0), 100);
        assert_eq!(streak_adjusted_exp(100, 5), 80);
    }

    #[test]
    fn test_penalized_exp() {
        assert_eq!(penalized_experience(100, ExpPenaltyReason::FriendlyFire), 0);
        assert_eq!(penalized_experience(100, ExpPenaltyReason::Genocide), 10);
        assert_eq!(penalized_experience(100, ExpPenaltyReason::Prayer), 50);
    }

    #[test]
    fn test_level_stats() {
        let mut stats = LevelStats::new();
        stats.record_kill(50);
        stats.record_kill(100);
        stats.record_level_gain(2);
        assert_eq!(stats.monsters_killed, 2);
        assert_eq!(stats.total_exp_gained, 150);
        assert_eq!(stats.max_level_reached, 2);
    }

    #[test]
    fn test_level_up_reward() {
        assert!(!level_up_reward(10).is_empty());
        assert!(level_up_reward(3).is_empty());
    }

    #[test]
    fn test_expected_max_hp() {
        let hp = expected_max_hp(
            PlayerClass::Valkyrie,
            crate::core::entity::player::Race::Human,
            10,
        );
        assert!(hp > 0);
    }
}
