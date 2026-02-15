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
//

use crate::core::entity::monster::DamageType;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionType {
    Dark,
    Noxious,
    Muddy,
    Wet,
    Magical,
    Fiery,
    Frosty,
}

impl ExplosionType {
    ///
    pub fn from_damage_type(dt: DamageType) -> Self {
        match dt {
            DamageType::Fire => Self::Fiery,
            DamageType::Cold => Self::Frosty,
            DamageType::Elec => Self::Magical,
            DamageType::Acid => Self::Muddy,
            DamageType::Drst | DamageType::Dise => Self::Noxious,
            DamageType::Deth => Self::Dark,
            _ => Self::Magical,
        }
    }

    ///
    pub fn color(&self) -> [u8; 3] {
        match self {
            Self::Dark => [80, 0, 80],
            Self::Noxious => [0, 200, 0],
            Self::Muddy => [150, 100, 0],
            Self::Wet => [0, 100, 255],
            Self::Magical => [255, 255, 0],
            Self::Fiery => [255, 80, 0],
            Self::Frosty => [200, 200, 255],
        }
    }

    ///
    pub fn description(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Noxious => "noxious",
            Self::Muddy => "muddy",
            Self::Wet => "wet",
            Self::Magical => "magical",
            Self::Fiery => "fiery",
            Self::Frosty => "frosty",
        }
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn explosion_offsets() -> Vec<(i32, i32)> {
    vec![
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ]
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct ExplosionResult {
    pub center: (i32, i32),
    pub explosion_type: ExplosionType,
    pub damage_per_cell: Vec<((i32, i32), i32)>,
    pub total_damage: i32,
    pub messages: Vec<String>,
}

///
pub fn explode(
    center_x: i32,
    center_y: i32,
    damage_type: DamageType,
    dice: i32,
    sides: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ExplosionResult {
    let expl_type = ExplosionType::from_damage_type(damage_type);

    log.add_colored(
        &format!("A {} explosion erupts!", expl_type.description()),
        expl_type.color(),
        turn,
    );

    let offsets = explosion_offsets();
    let mut damage_per_cell = Vec::new();
    let mut total = 0;

    for &(dx, dy) in &offsets {
        let x = center_x + dx;
        let y = center_y + dy;

        //
        let dmg = roll_damage(dice, sides, rng);
        damage_per_cell.push(((x, y), dmg));
        total += dmg;
    }

    ExplosionResult {
        center: (center_x, center_y),
        explosion_type: expl_type,
        damage_per_cell,
        total_damage: total,
        messages: vec![format!("A {} explosion erupts!", expl_type.description())],
    }
}

///
fn roll_damage(dice: i32, sides: i32, rng: &mut NetHackRng) -> i32 {
    if sides <= 0 || dice <= 0 {
        return 0;
    }
    let mut total = 0;
    for _ in 0..dice {
        total += rng.rn2(sides) + 1;
    }
    total
}

// =============================================================================
//
// =============================================================================

///
pub fn resist_explosion(base_damage: i32, has_resistance: bool) -> i32 {
    if has_resistance {
        base_damage / 2
    } else {
        base_damage
    }
}

///
pub fn magic_resist_explosion(
    base_damage: i32,
    magic_resistance: i32,
    rng: &mut NetHackRng,
) -> i32 {
    //
    if magic_resistance > 0 && rng.rn2(100) < magic_resistance {
        base_damage / 2
    } else {
        base_damage
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyableType {
    Potion,
    Scroll,
    Spellbook,
    Ring,
    Wand,
}

///
pub fn vulnerable_items(expl_type: ExplosionType) -> Vec<DestroyableType> {
    match expl_type {
        ExplosionType::Fiery => vec![
            DestroyableType::Scroll,
            DestroyableType::Spellbook,
            DestroyableType::Potion,
        ],
        ExplosionType::Frosty => vec![DestroyableType::Potion],
        ExplosionType::Magical => vec![DestroyableType::Wand],
        _ => vec![],
    }
}

///
pub fn item_destroy_chance(expl_type: ExplosionType, rng: &mut NetHackRng) -> bool {
    match expl_type {
        ExplosionType::Fiery => rng.rn2(3) == 0,
        ExplosionType::Frosty => rng.rn2(5) == 0,
        ExplosionType::Magical => rng.rn2(10) == 0,
        _ => false,
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explosion_type_from_damage() {
        assert_eq!(
            ExplosionType::from_damage_type(DamageType::Fire),
            ExplosionType::Fiery
        );
        assert_eq!(
            ExplosionType::from_damage_type(DamageType::Cold),
            ExplosionType::Frosty
        );
        assert_eq!(
            ExplosionType::from_damage_type(DamageType::Elec),
            ExplosionType::Magical
        );
    }

    #[test]
    fn test_explosion_offsets() {
        let offsets = explosion_offsets();
        assert_eq!(offsets.len(), 9);
        assert!(offsets.contains(&(0, 0)));
    }

    #[test]
    fn test_explode() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(50);
        let result = explode(5, 5, DamageType::Fire, 3, 6, &mut rng, &mut log, 1);
        assert_eq!(result.center, (5, 5));
        assert_eq!(result.damage_per_cell.len(), 9);
        assert!(result.total_damage > 0);
    }

    #[test]
    fn test_resist_explosion() {
        assert_eq!(resist_explosion(20, false), 20);
        assert_eq!(resist_explosion(20, true), 10);
    }

    #[test]
    fn test_vulnerable_items() {
        let fire_vuln = vulnerable_items(ExplosionType::Fiery);
        assert!(fire_vuln.contains(&DestroyableType::Scroll));
        assert!(fire_vuln.contains(&DestroyableType::Spellbook));

        let frost_vuln = vulnerable_items(ExplosionType::Frosty);
        assert!(frost_vuln.contains(&DestroyableType::Potion));
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn explosion_offsets_5x5() -> Vec<(i32, i32)> {
    let mut offsets = Vec::new();
    for dy in -2..=2 {
        for dx in -2..=2 {
            offsets.push((dx, dy));
        }
    }
    offsets
}

///
pub fn explosion_radius(dice: i32) -> i32 {
    if dice >= 8 {
        2 // 5횞5
    } else if dice >= 4 {
        1 // 3횞3
    } else {
        0
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn can_chain_explode(damage_type: DamageType) -> bool {
    matches!(damage_type, DamageType::Fire | DamageType::Magm)
}

///
pub fn chain_explode_chance(damage_type: DamageType, intensity: i32, rng: &mut NetHackRng) -> bool {
    if !can_chain_explode(damage_type) {
        return false;
    }
    //
    let threshold = (intensity * 5).min(50);
    rng.rn2(100) < threshold
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionDefense {
    None,
    Resistance,
    HalfDamage,
    Globe,
    Reflection,
}

///
pub fn apply_explosion_defense(base_damage: i32, defense: ExplosionDefense) -> i32 {
    match defense {
        ExplosionDefense::None => base_damage,
        ExplosionDefense::Resistance => base_damage / 2,
        ExplosionDefense::HalfDamage => base_damage / 2,
        ExplosionDefense::Globe => 0,
        ExplosionDefense::Reflection => 0,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainChange {
    None,
    BurnFloor,
    FreezeWater,
    MeltIce,
    EvaporatePool,
    DigFloor,
}

///
pub fn explosion_terrain_effect(expl_type: ExplosionType) -> TerrainChange {
    match expl_type {
        ExplosionType::Fiery => TerrainChange::BurnFloor,
        ExplosionType::Frosty => TerrainChange::FreezeWater,
        ExplosionType::Magical => TerrainChange::DigFloor,
        _ => TerrainChange::None,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn explosion_sound(expl_type: ExplosionType, distance: i32) -> &'static str {
    if distance > 10 {
        return "You hear a distant explosion.";
    }
    if distance > 5 {
        return "You hear an explosion!";
    }

    match expl_type {
        ExplosionType::Fiery => "KABOOM! A ball of fire erupts!",
        ExplosionType::Frosty => "CRACK! A blast of cold!",
        ExplosionType::Magical => "WHOOSH! A magical explosion!",
        ExplosionType::Muddy => "SPLAT! An acidic burst!",
        ExplosionType::Wet => "SPLASH! A watery explosion!",
        ExplosionType::Dark => "BOOM! A dark explosion!",
        ExplosionType::Noxious => "HISS! A noxious burst!",
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct ExplosionStatistics {
    pub total_explosions: u32,
    pub total_damage: u64,
    pub chain_explosions: u32,
    pub items_destroyed: u32,
    pub terrain_changes: u32,
    pub by_type: [u32; 7],
}

impl ExplosionStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_explosion(&mut self, expl_type: ExplosionType, damage: i32) {
        self.total_explosions += 1;
        self.total_damage += damage as u64;
        let idx = match expl_type {
            ExplosionType::Dark => 0,
            ExplosionType::Noxious => 1,
            ExplosionType::Muddy => 2,
            ExplosionType::Wet => 3,
            ExplosionType::Magical => 4,
            ExplosionType::Fiery => 5,
            ExplosionType::Frosty => 6,
        };
        self.by_type[idx] += 1;
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod explode_extended_tests {
    use super::*;

    #[test]
    fn test_5x5_offsets() {
        let offsets = explosion_offsets_5x5();
        assert_eq!(offsets.len(), 25);
    }

    #[test]
    fn test_explosion_radius() {
        assert_eq!(explosion_radius(10), 2);
        assert_eq!(explosion_radius(5), 1);
        assert_eq!(explosion_radius(2), 0);
    }

    #[test]
    fn test_chain_explode() {
        assert!(can_chain_explode(DamageType::Fire));
        assert!(!can_chain_explode(DamageType::Phys));
    }

    #[test]
    fn test_explosion_defense() {
        assert_eq!(apply_explosion_defense(20, ExplosionDefense::None), 20);
        assert_eq!(
            apply_explosion_defense(20, ExplosionDefense::Resistance),
            10
        );
        assert_eq!(apply_explosion_defense(20, ExplosionDefense::Globe), 0);
    }

    #[test]
    fn test_terrain_effect() {
        assert_eq!(
            explosion_terrain_effect(ExplosionType::Fiery),
            TerrainChange::BurnFloor
        );
        assert_eq!(
            explosion_terrain_effect(ExplosionType::Frosty),
            TerrainChange::FreezeWater
        );
    }

    #[test]
    fn test_explosion_sound() {
        assert!(explosion_sound(ExplosionType::Fiery, 3).contains("KABOOM"));
        assert!(explosion_sound(ExplosionType::Fiery, 15).contains("distant"));
    }

    #[test]
    fn test_explosion_stats() {
        let mut stats = ExplosionStatistics::new();
        stats.record_explosion(ExplosionType::Fiery, 30);
        stats.record_explosion(ExplosionType::Frosty, 20);
        assert_eq!(stats.total_explosions, 2);
        assert_eq!(stats.total_damage, 50);
    }
}
