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

use crate::core::entity::player::{Alignment, HungerState, Player};
use crate::core::systems::role::{get_role_data, rank_of};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct StatusLine1 {
    pub role_name: String,
    pub rank: String,
    pub dungeon_name: String,
    pub depth: i32,
    pub gold: u64,
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub ac: i32,
    pub level: i32,
    pub experience: u64,
}

///
#[derive(Debug, Clone)]
pub struct StatusLine2 {
    pub str_val: String,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,                         // 留ㅻ젰
    pub alignment: Alignment,
    pub hunger: HungerState,
    pub conditions: Vec<StatusCondition>,
    pub turn: u64,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCondition {
    Blind,
    Confused,
    Stunned,
    Hallucinating,
    Slimed,
    Stoning,
    Strangled,
    FoodPoisoned,
    TerminallyIll, // 吏덈퀝
    Levitating,
    Flying,
    Riding,
}

// =============================================================================
//
// =============================================================================

///
pub fn generate_status_line1(player: &Player, dungeon_name: &str, depth: i32) -> StatusLine1 {
    let rd = get_role_data(player.role);
    StatusLine1 {
        role_name: rd.name.to_string(),
        rank: rank_of(player.role, player.exp_level).to_string(),
        dungeon_name: dungeon_name.to_string(),
        depth,
        gold: player.gold,
        hp: player.hp,
        hp_max: player.hp_max,
        energy: player.energy,
        energy_max: player.energy_max,
        ac: player.ac,
        level: player.exp_level,
        experience: player.experience,
    }
}

///
pub fn generate_status_line2(
    player: &Player,
    conditions: &[StatusCondition],
    turn: u64,
) -> StatusLine2 {
    StatusLine2 {
        str_val: format_strength(player.str.base),
        dex: player.dex.base,
        con: player.con.base,
        int: player.int.base,
        wis: player.wis.base,
        cha: player.cha.base,
        alignment: player.alignment,
        hunger: player.hunger,
        conditions: conditions.to_vec(),
        turn,
    }
}

///
fn format_strength(str_val: i32) -> String {
    if str_val > 18 && str_val <= 118 {
        //
        let excess = str_val - 18;
        if excess >= 100 {
            "18/**".to_string()
        } else {
            format!("18/{:02}", excess)
        }
    } else {
        format!("{}", str_val)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn format_line1(s: &StatusLine1) -> String {
    format!(
        "{} the {} St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
        s.rank, s.role_name, s.hp, s.hp_max, s.energy, s.energy_max, s.ac, s.level,
    )
}

///
pub fn format_status_compact(s1: &StatusLine1, s2: &StatusLine2) -> (String, String) {
    let line1 = format!(
        "{} the {} - {}:{}  $:{}  HP:{}/{}  Pw:{}/{}  AC:{}  Xp:{}",
        s1.rank,
        s1.role_name,
        s1.dungeon_name,
        s1.depth,
        s1.gold,
        s1.hp,
        s1.hp_max,
        s1.energy,
        s1.energy_max,
        s1.ac,
        s1.level,
    );

    let mut line2 = format!(
        "St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}  {}  {}",
        s2.str_val,
        s2.dex,
        s2.con,
        s2.int,
        s2.wis,
        s2.cha,
        alignment_str(s2.alignment),
        hunger_str(s2.hunger),
    );

    //
    for cond in &s2.conditions {
        line2.push_str(&format!(" {}", condition_str(*cond)));
    }

    line2.push_str(&format!(" T:{}", s2.turn));

    (line1, line2)
}

///
fn alignment_str(align: Alignment) -> &'static str {
    match align {
        Alignment::Lawful => "Lawful",
        Alignment::Neutral => "Neutral",
        Alignment::Chaotic => "Chaotic",
    }
}

///
fn hunger_str(hunger: HungerState) -> &'static str {
    match hunger {
        HungerState::Satiated => "Satiated",
        HungerState::NotHungry => "",
        HungerState::Hungry => "Hungry",
        HungerState::Weak => "Weak",
        HungerState::Fainting => "Fainting",
        HungerState::Starved => "Starved",
    }
}

///
fn condition_str(cond: StatusCondition) -> &'static str {
    match cond {
        StatusCondition::Blind => "Blind",
        StatusCondition::Confused => "Conf",
        StatusCondition::Stunned => "Stun",
        StatusCondition::Hallucinating => "Hallu",
        StatusCondition::Slimed => "Slime",
        StatusCondition::Stoning => "Stone",
        StatusCondition::Strangled => "Strngl",
        StatusCondition::FoodPoisoned => "FoodPois",
        StatusCondition::TerminallyIll => "Ill",
        StatusCondition::Levitating => "Lev",
        StatusCondition::Flying => "Fly",
        StatusCondition::Riding => "Ride",
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn hp_color(hp: i32, hp_max: i32) -> [u8; 3] {
    if hp_max <= 0 {
        return [255, 0, 0];
    }
    let ratio = (hp as f32) / (hp_max as f32);
    if ratio <= 0.0 {
        [255, 0, 0]
    } else if ratio <= 0.15 {
        [255, 50, 50]
    } else if ratio <= 0.33 {
        [255, 150, 50]
    } else if ratio <= 0.50 {
        [255, 255, 50]
    } else if ratio <= 0.75 {
        [100, 255, 100]
    } else {
        [50, 255, 50]
    }
}

///
pub fn pw_color(pw: i32, pw_max: i32) -> [u8; 3] {
    if pw_max <= 0 {
        return [100, 100, 255];
    }
    let ratio = (pw as f32) / (pw_max as f32);
    if ratio <= 0.0 {
        [50, 50, 100]
    } else if ratio <= 0.25 {
        [100, 100, 200]
    } else if ratio <= 0.50 {
        [130, 130, 255]
    } else {
        [100, 150, 255]
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusField {
    Title = 0,
    Strength = 1,
    Dexterity = 2,
    Constitution = 3,
    Intelligence = 4,
    Wisdom = 5,
    Charisma = 6,
    Alignment = 7,
    Score = 8,
    Cap = 9,
    Gold = 10,
    EnergyNow = 11,
    EnergyMax = 12,
    HpNow = 13,
    HpMax = 14,
    ArmorClass = 15,
    Hd = 16,
    Time = 17,
    Hunger = 18,
    Exp = 19,
    Condition = 20,
}

///
pub const BL_MASK_STONE: u32 = 0x00000001;
pub const BL_MASK_SLIME: u32 = 0x00000002;
pub const BL_MASK_STRNGL: u32 = 0x00000004;
pub const BL_MASK_FOODPOIS: u32 = 0x00000008;
pub const BL_MASK_TERMILL: u32 = 0x00000010;
pub const BL_MASK_BLIND: u32 = 0x00000020;
pub const BL_MASK_DEAF: u32 = 0x00000040;
pub const BL_MASK_STUN: u32 = 0x00000080;
pub const BL_MASK_CONF: u32 = 0x00000100;
pub const BL_MASK_HALLU: u32 = 0x00000200;
pub const BL_MASK_LEV: u32 = 0x00000400;
pub const BL_MASK_FLY: u32 = 0x00000800;
pub const BL_MASK_RIDE: u32 = 0x00001000;

///
pub fn conditions_from_mask(mask: u32) -> Vec<StatusCondition> {
    let mut conds = Vec::new();
    if mask & BL_MASK_STONE != 0 {
        conds.push(StatusCondition::Stoning);
    }
    if mask & BL_MASK_SLIME != 0 {
        conds.push(StatusCondition::Slimed);
    }
    if mask & BL_MASK_STRNGL != 0 {
        conds.push(StatusCondition::Strangled);
    }
    if mask & BL_MASK_FOODPOIS != 0 {
        conds.push(StatusCondition::FoodPoisoned);
    }
    if mask & BL_MASK_TERMILL != 0 {
        conds.push(StatusCondition::TerminallyIll);
    }
    if mask & BL_MASK_BLIND != 0 {
        conds.push(StatusCondition::Blind);
    }
    if mask & BL_MASK_STUN != 0 {
        conds.push(StatusCondition::Stunned);
    }
    if mask & BL_MASK_CONF != 0 {
        conds.push(StatusCondition::Confused);
    }
    if mask & BL_MASK_HALLU != 0 {
        conds.push(StatusCondition::Hallucinating);
    }
    if mask & BL_MASK_LEV != 0 {
        conds.push(StatusCondition::Levitating);
    }
    if mask & BL_MASK_FLY != 0 {
        conds.push(StatusCondition::Flying);
    }
    if mask & BL_MASK_RIDE != 0 {
        conds.push(StatusCondition::Riding);
    }
    conds
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncumbranceLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

///
pub fn encumbrance_str(level: EncumbranceLevel) -> &'static str {
    match level {
        EncumbranceLevel::Unencumbered => "",
        EncumbranceLevel::Burdened => "Burdened",
        EncumbranceLevel::Stressed => "Stressed",
        EncumbranceLevel::Strained => "Strained",
        EncumbranceLevel::Overtaxed => "Overtaxed",
        EncumbranceLevel::Overloaded => "Overloaded",
    }
}

///
pub fn weight_cap(str_val: i32, _con_val: i32) -> i32 {
    //
    let base = 25 * str_val + 50;
    base.max(100) // 理쒖냼 100
}

///
pub fn near_capacity(carried_weight: i32, max_weight: i32) -> EncumbranceLevel {
    if max_weight <= 0 {
        return EncumbranceLevel::Overloaded;
    }
    let ratio = (carried_weight * 100) / max_weight;
    match ratio {
        0..=50 => EncumbranceLevel::Unencumbered,
        51..=67 => EncumbranceLevel::Burdened,
        68..=83 => EncumbranceLevel::Stressed,
        84..=93 => EncumbranceLevel::Strained,
        94..=100 => EncumbranceLevel::Overtaxed,
        _ => EncumbranceLevel::Overloaded,
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_strength() {
        assert_eq!(format_strength(10), "10");
        assert_eq!(format_strength(18), "18");
        assert_eq!(format_strength(19), "18/01");
        assert_eq!(format_strength(68), "18/50");
        assert_eq!(format_strength(118), "18/**");
    }

    #[test]
    fn test_hp_color() {
        let color = hp_color(100, 100);
        assert_eq!(color[1], 255);
        let color = hp_color(1, 100);
        assert!(color[0] > 200);
    }

    #[test]
    fn test_conditions_from_mask() {
        let mask = BL_MASK_BLIND | BL_MASK_CONF;
        let conds = conditions_from_mask(mask);
        assert!(conds.contains(&StatusCondition::Blind));
        assert!(conds.contains(&StatusCondition::Confused));
        assert!(!conds.contains(&StatusCondition::Stunned));
    }

    #[test]
    fn test_near_capacity() {
        assert_eq!(near_capacity(100, 400), EncumbranceLevel::Unencumbered);
        assert_eq!(near_capacity(300, 400), EncumbranceLevel::Stressed);
        assert_eq!(near_capacity(500, 400), EncumbranceLevel::Overloaded);
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn hp_status_text(hp: i32, hp_max: i32) -> &'static str {
    if hp_max <= 0 {
        return "Dead";
    }
    let pct = (hp * 100) / hp_max;
    match pct {
        ..=0 => "Dead",
        1..=10 => "Critical",
        11..=25 => "Badly Wounded",
        26..=50 => "Wounded",
        51..=75 => "Hurt",
        76..=90 => "Scratched",
        91..=99 => "Almost Healthy",
        _ => "Healthy",
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpWarningLevel {
    None,
    Minor,
    Moderate,
    Serious,
    Critical,
    NearDeath,
}

///
pub fn hp_warning_level(prev_hp: i32, curr_hp: i32, hp_max: i32) -> HpWarningLevel {
    if curr_hp >= prev_hp {
        return HpWarningLevel::None;
    }
    if hp_max <= 0 {
        return HpWarningLevel::Critical;
    }
    let pct = (curr_hp * 100) / hp_max;
    match pct {
        ..=5 => HpWarningLevel::NearDeath,
        6..=15 => HpWarningLevel::Critical,
        16..=30 => HpWarningLevel::Serious,
        31..=50 => HpWarningLevel::Moderate,
        _ => HpWarningLevel::Minor,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn experience_for_level(level: i32) -> u64 {
    //
    match level {
        1 => 0,
        2 => 20,
        3 => 40,
        4 => 80,
        5 => 160,
        6 => 320,
        7 => 640,
        8 => 1_280,
        9 => 2_560,
        10 => 5_120,
        11 => 10_000,
        12 => 20_000,
        13 => 40_000,
        14 => 80_000,
        15 => 160_000,
        16 => 320_000,
        17 => 640_000,
        18 => 1_280_000,
        19 => 2_560_000,
        20 => 5_120_000,
        21 => 10_000_000,
        22 => 20_000_000,
        23 => 40_000_000,
        24 => 80_000_000,
        25 => 160_000_000,
        26 => 320_000_000,
        27 => 640_000_000,
        28 => 1_280_000_000,
        29 => 2_560_000_000,
        30 => 5_000_000_000,
        _ => 10_000_000_000,
    }
}

///
pub fn level_for_experience(exp: u64) -> i32 {
    for lv in (1..=30).rev() {
        if exp >= experience_for_level(lv) {
            return lv;
        }
    }
    1
}

///
pub fn exp_to_next_level(current_exp: u64, current_level: i32) -> u64 {
    let next = experience_for_level(current_level + 1);
    if current_exp >= next {
        0
    } else {
        next - current_exp
    }
}

///
pub fn exp_progress(current_exp: u64, current_level: i32) -> f32 {
    let curr_threshold = experience_for_level(current_level);
    let next_threshold = experience_for_level(current_level + 1);
    if next_threshold <= curr_threshold {
        return 1.0;
    }
    let progress = current_exp.saturating_sub(curr_threshold) as f32;
    let range = (next_threshold - curr_threshold) as f32;
    (progress / range).min(1.0)
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, PartialEq)]
pub enum StatusChangeEvent {
    HpChanged { old: i32, new: i32 },
    PwChanged { old: i32, new: i32 },
    LevelUp { old_level: i32, new_level: i32 },
    LevelDown { old_level: i32, new_level: i32 },
    GoldChanged { old: u64, new: u64 },
    AcChanged { old: i32, new: i32 },
    HungerChanged { old: HungerState, new: HungerState },
    ConditionAdded(StatusCondition),
    ConditionRemoved(StatusCondition),
    StrChanged { old: i32, new: i32 },
}

///
pub fn detect_status_changes(
    old_s1: &StatusLine1,
    new_s1: &StatusLine1,
    old_s2: &StatusLine2,
    new_s2: &StatusLine2,
) -> Vec<StatusChangeEvent> {
    let mut events = Vec::new();

    if old_s1.hp != new_s1.hp {
        events.push(StatusChangeEvent::HpChanged {
            old: old_s1.hp,
            new: new_s1.hp,
        });
    }
    if old_s1.energy != new_s1.energy {
        events.push(StatusChangeEvent::PwChanged {
            old: old_s1.energy,
            new: new_s1.energy,
        });
    }
    if new_s1.level > old_s1.level {
        events.push(StatusChangeEvent::LevelUp {
            old_level: old_s1.level,
            new_level: new_s1.level,
        });
    }
    if new_s1.level < old_s1.level {
        events.push(StatusChangeEvent::LevelDown {
            old_level: old_s1.level,
            new_level: new_s1.level,
        });
    }
    if old_s1.gold != new_s1.gold {
        events.push(StatusChangeEvent::GoldChanged {
            old: old_s1.gold,
            new: new_s1.gold,
        });
    }
    if old_s1.ac != new_s1.ac {
        events.push(StatusChangeEvent::AcChanged {
            old: old_s1.ac,
            new: new_s1.ac,
        });
    }
    if old_s2.hunger != new_s2.hunger {
        events.push(StatusChangeEvent::HungerChanged {
            old: old_s2.hunger,
            new: new_s2.hunger,
        });
    }

    //
    for cond in &new_s2.conditions {
        if !old_s2.conditions.contains(cond) {
            events.push(StatusChangeEvent::ConditionAdded(*cond));
        }
    }
    for cond in &old_s2.conditions {
        if !new_s2.conditions.contains(cond) {
            events.push(StatusChangeEvent::ConditionRemoved(*cond));
        }
    }

    events
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct FullStatusSnapshot {
    pub line1: StatusLine1,
    pub line2: StatusLine2,
    pub encumbrance: EncumbranceLevel,
    pub hp_status: String,
    pub exp_progress: f32,
    pub exp_to_next: u64,
}

///
pub fn generate_full_snapshot(
    player: &Player,
    dungeon_name: &str,
    depth: i32,
    conditions: &[StatusCondition],
    carried_weight: i32,
    max_weight: i32,
    turn: u64,
) -> FullStatusSnapshot {
    let line1 = generate_status_line1(player, dungeon_name, depth);
    let line2 = generate_status_line2(player, conditions, turn);
    let enc = near_capacity(carried_weight, max_weight);
    let hp_text = hp_status_text(player.hp, player.hp_max).to_string();
    let progress = exp_progress(player.experience, player.exp_level);
    let to_next = exp_to_next_level(player.experience, player.exp_level);

    FullStatusSnapshot {
        line1,
        line2,
        encumbrance: enc,
        hp_status: hp_text,
        exp_progress: progress,
        exp_to_next: to_next,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn estimate_score(gold: u64, experience: u64, depth: i32, hp_max: i32, amulet: bool) -> u64 {
    let mut score = gold;
    score += experience / 10;
    score += (depth as u64) * 100;
    score += (hp_max as u64) * 5;
    if amulet {
        score += 50000;
    }
    score
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn ac_description(ac: i32) -> &'static str {
    match ac {
        ..=-10 => "Impervious",
        -9..=-5 => "Heavily Armored",
        -4..=-1 => "Well Protected",
        0..=2 => "Lightly Armored",
        3..=5 => "Poorly Protected",
        6..=8 => "Barely Protected",
        _ => "Defenseless",
    }
}

///
pub fn ac_color(ac: i32) -> [u8; 3] {
    match ac {
        ..=-10 => [50, 200, 255],
        -9..=-5 => [100, 255, 100],
        -4..=-1 => [200, 255, 100],
        0..=2 => [255, 255, 100],
        3..=5 => [255, 200, 50],
        6..=8 => [255, 100, 50],
        _ => [255, 50, 50],
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================
#[cfg(test)]
mod botl_extended_tests {
    use super::*;

    #[test]
    fn test_hp_status_text() {
        assert_eq!(hp_status_text(100, 100), "Healthy");
        assert_eq!(hp_status_text(50, 100), "Wounded");
        assert_eq!(hp_status_text(10, 100), "Critical");
        assert_eq!(hp_status_text(0, 100), "Dead");
    }

    #[test]
    fn test_hp_warning_level() {
        assert_eq!(hp_warning_level(100, 100, 100), HpWarningLevel::None);
        assert_eq!(hp_warning_level(100, 50, 100), HpWarningLevel::Moderate);
        assert_eq!(hp_warning_level(100, 3, 100), HpWarningLevel::NearDeath);
    }

    #[test]
    fn test_experience_table() {
        assert_eq!(experience_for_level(1), 0);
        assert_eq!(experience_for_level(2), 20);
        assert_eq!(experience_for_level(10), 5120);
        assert!(experience_for_level(30) > experience_for_level(29));
    }

    #[test]
    fn test_level_for_experience() {
        assert_eq!(level_for_experience(0), 1);
        assert_eq!(level_for_experience(25), 2);
        assert_eq!(level_for_experience(100_000_000), 24);
    }

    #[test]
    fn test_exp_progress() {
        let p = exp_progress(10, 1);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_ac_description() {
        assert_eq!(ac_description(-15), "Impervious");
        assert_eq!(ac_description(0), "Lightly Armored");
        assert_eq!(ac_description(10), "Defenseless");
    }

    #[test]
    fn test_score_estimation() {
        let score1 = estimate_score(1000, 5000, 10, 50, false);
        let score2 = estimate_score(1000, 5000, 10, 50, true);
        assert!(score2 > score1);
        assert!(score2 - score1 == 50000);
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpDanger {
    Critical,
    Danger,
    Warning,
    Caution,
    Healthy,
}

///
pub fn hp_danger_level(current: i32, max: i32) -> HpDanger {
    if max <= 0 {
        return HpDanger::Critical;
    }
    let ratio = (current * 100) / max;
    if ratio <= 10 {
        HpDanger::Critical
    } else if ratio <= 25 {
        HpDanger::Danger
    } else if ratio <= 50 {
        HpDanger::Warning
    } else if ratio <= 75 {
        HpDanger::Caution
    } else {
        HpDanger::Healthy
    }
}

///
pub fn hp_danger_color(danger: HpDanger) -> [u8; 3] {
    match danger {
        HpDanger::Critical => [255, 0, 0],
        HpDanger::Danger => [255, 80, 80],
        HpDanger::Warning => [255, 255, 0],
        HpDanger::Caution => [180, 255, 80],
        HpDanger::Healthy => [0, 255, 0],
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyLevel {
    Empty,  // 0
    Low,
    Medium,
    High,
    Full,   // 75%+
}

///
pub fn energy_level(current: i32, max: i32) -> EnergyLevel {
    if current <= 0 {
        return EnergyLevel::Empty;
    }
    if max <= 0 {
        return EnergyLevel::Empty;
    }
    let ratio = (current * 100) / max;
    if ratio <= 25 {
        EnergyLevel::Low
    } else if ratio <= 50 {
        EnergyLevel::Medium
    } else if ratio <= 75 {
        EnergyLevel::High
    } else {
        EnergyLevel::Full
    }
}

///
pub fn xp_for_next_level(current_level: i32) -> u64 {
    match current_level {
        1 => 20,
        2 => 40,
        3 => 80,
        4 => 160,
        5 => 320,
        6 => 640,
        7 => 1280,
        8 => 2560,
        9 => 5120,
        10 => 10000,
        11 => 20000,
        12 => 40000,
        13 => 80000,
        14 => 160000,
        15 => 320000,
        16 => 640000,
        17 => 1280000,
        18 => 2560000,
        19 => 5120000,
        20 => 10000000,
        21 => 20000000,
        22 => 40000000,
        23 => 80000000,
        24 => 160000000,
        25 => 320000000,
        26 => 640000000,
        27 => 1280000000,
        28 => 2560000000,
        29 => 5120000000,
        _ => u64::MAX,
    }
}

///
pub fn xp_progress_percent(current_xp: u64, level: i32) -> u8 {
    let needed = xp_for_next_level(level);
    if needed == u64::MAX {
        return 100;
    }
    let prev = if level > 1 {
        xp_for_next_level(level - 1)
    } else {
        0
    };
    let range = needed - prev;
    if range == 0 {
        return 100;
    }
    let progress = current_xp.saturating_sub(prev);
    ((progress * 100) / range).min(100) as u8
}

///
pub fn dungeon_danger_label(depth: i32, player_level: i32) -> &'static str {
    let diff = depth - player_level;
    if diff >= 10 {
        "SUICIDAL"
    } else if diff >= 5 {
        "DEADLY"
    } else if diff >= 2 {
        "DANGEROUS"
    } else if diff >= 0 {
        "MODERATE"
    } else {
        "SAFE"
    }
}

///
pub fn condition_severity(condition: &str) -> i32 {
    let l = condition.to_lowercase();
    if l.contains("stoned") || l.contains("slimed") {
        return 10;
    }
    if l.contains("strangled") || l.contains("food poison") {
        return 9;
    }
    if l.contains("ill") {
        return 8;
    }
    if l.contains("paralyzed") || l.contains("sleeping") {
        return 7;
    }
    if l.contains("stunned") || l.contains("confused") {
        return 5;
    }
    if l.contains("blind") {
        return 4;
    }
    if l.contains("hallucinating") {
        return 3;
    }
    if l.contains("hungry") || l.contains("weak") {
        return 2;
    }
    if l.contains("burdened") || l.contains("stressed") {
        return 1;
    }
    0
}

///
pub fn condition_expire_message(condition: &str) -> &'static str {
    let l = condition.to_lowercase();
    if l.contains("blind") {
        return "You can see again.";
    }
    if l.contains("confused") {
        return "You feel less confused now.";
    }
    if l.contains("stunned") {
        return "You feel steady again.";
    }
    if l.contains("hallucinating") {
        return "Everything looks SO boring now.";
    }
    if l.contains("paralyzed") {
        return "You can move again!";
    }
    if l.contains("sleeping") {
        return "You wake up.";
    }
    if l.contains("levitating") {
        return "You float gently to the ground.";
    }
    ""
}

///
#[derive(Debug, Clone, Default)]
pub struct BotlStatistics {
    pub updates: u32,
    pub critical_warnings: u32,
    pub conditions_gained: u32,
    pub conditions_expired: u32,
    pub level_ups: u32,
}

impl BotlStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_update(&mut self) {
        self.updates += 1;
    }
    pub fn record_critical(&mut self) {
        self.critical_warnings += 1;
    }
    pub fn record_levelup(&mut self) {
        self.level_ups += 1;
    }
}

#[cfg(test)]
mod botl_advanced_tests {
    use super::*;

    #[test]
    fn test_hp_danger() {
        assert_eq!(hp_danger_level(5, 100), HpDanger::Critical);
        assert_eq!(hp_danger_level(80, 100), HpDanger::Healthy);
        assert_eq!(hp_danger_level(30, 100), HpDanger::Warning);
    }

    #[test]
    fn test_energy() {
        assert_eq!(energy_level(0, 50), EnergyLevel::Empty);
        assert_eq!(energy_level(50, 50), EnergyLevel::Full);
    }

    #[test]
    fn test_xp_table() {
        assert_eq!(xp_for_next_level(1), 20);
        assert!(xp_for_next_level(10) > xp_for_next_level(5));
    }

    #[test]
    fn test_xp_progress() {
        let p = xp_progress_percent(30, 1);
        assert!(p > 0 && p <= 100);
    }

    #[test]
    fn test_dungeon_danger() {
        assert_eq!(dungeon_danger_label(20, 5), "SUICIDAL");
        assert_eq!(dungeon_danger_label(5, 10), "SAFE");
    }

    #[test]
    fn test_condition_severity() {
        assert!(condition_severity("stoned") > condition_severity("hungry"));
    }

    #[test]
    fn test_condition_expire() {
        let m = condition_expire_message("blind");
        assert!(m.contains("see"));
    }

    #[test]
    fn test_botl_stats() {
        let mut s = BotlStatistics::new();
        s.record_update();
        s.record_critical();
        s.record_levelup();
        assert_eq!(s.updates, 1);
        assert_eq!(s.level_ups, 1);
    }
}
