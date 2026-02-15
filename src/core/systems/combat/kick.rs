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
//!
//!

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::{PlayerTag, Position};
use crate::core::game_state::Direction;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::*;

///
///
pub fn try_kick(
    world: &mut World,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    direction: Direction,
    rng: &mut NetHackRng,
) -> bool {
    //
    if direction == Direction::Here {
        log.add("You kick at empty space.", turn);
        return true;
    }

    //
    let mut player_pos: Option<(i32, i32)> = None;
    let mut query = <(&mut crate::core::entity::player::Player, &Position)>::query()
        .filter(component::<PlayerTag>());

    for (player, pos) in query.iter_mut(world) {
        player_pos = Some((pos.x, pos.y));
        //
        crate::core::systems::attrib::exercise(
            player,
            crate::core::systems::attrib::AttributeIndex::Str,
            true,
        );
    }

    let (px, py) = match player_pos {
        Some(p) => p,
        None => {
            log.add("You don't exist!", turn);
            return false;
        }
    };

    //
    let (dx, dy) = direction.to_delta();
    let tx = px + dx;
    let ty = py + dy;

    //
    if tx < 0 || tx >= COLNO as i32 || ty < 0 || ty >= ROWNO as i32 {
        log.add("You kick at nothing.", turn);
        return true;
    }

    //
    //

    //
    if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
        match tile.typ {
            TileType::Door => {
                //
                //
                //
                let success = rng.rn2(2) == 0;

                if success {
                    //
                    tile.typ = TileType::OpenDoor;
                    tile.doormas = 2;
                    log.add("CRASH! The door breaks open!", turn);
                } else {
                    log.add("THUD! The door resists your kick.", turn);
                }
                true
            }
            TileType::OpenDoor => {
                log.add("You kick at the open doorway.", turn);
                true
            }
            TileType::Sink => {
                crate::core::systems::sink::try_kick_sink(world, grid, log, turn, rng, (tx, ty));
                true
            }
            TileType::HWall | TileType::VWall | TileType::Stone => {
                //
                log.add("Ouch! That hurts!", turn);
                //
                true
            }
            TileType::Room | TileType::Corr => {
                //
                log.add("You kick at empty space.", turn);
                true
            }
            TileType::StairsUp | TileType::StairsDown => {
                log.add("You kick at the staircase.", turn);
                true
            }
            _ => {
                log.add("You kick at nothing.", turn);
                true
            }
        }
    } else {
        log.add("You kick at nothing.", turn);
        true
    }
}

// =============================================================================
// [v2.3.1
//
//
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KickTarget {
    ///
    Monster,
    ///
    Door,
    ///
    Boulder,
    ///
    Furniture,
    ///
    Object,
    ///
    Empty,
    ///
    Wall,
}

///
#[derive(Debug, Clone)]
pub struct KickResult {
    pub target: KickTarget,
    pub damage_to_target: i32,
    pub damage_to_self: i32,
    pub message: String,
    pub success: bool,
}

///
///
pub fn is_martial(role: &str) -> bool {
    matches!(role, "Monk" | "Samurai")
}

///
pub fn kick_damage(
    player_str: i32,
    player_level: i32,
    is_martial_artist: bool,
    wearing_boots: bool,
    boots_enchant: i32,
    rng: &mut NetHackRng,
) -> i32 {
    let mut damage = rng.d(1, 4);

    //
    if player_str >= 18 {
        damage += (player_str - 15) / 3;
    } else if player_str >= 16 {
        damage += 1;
    }

    //
    damage += player_level / 5;

    //
    if is_martial_artist {
        damage += rng.d(1, 4) + (player_level / 3);
    }

    //
    if wearing_boots {
        damage += boots_enchant.max(0);
        //
        damage += 1;
    }

    damage.max(1)
}

///
pub fn door_kick_chance(player_str: i32, is_martial_artist: bool, rng: &mut NetHackRng) -> bool {
    let mut chance = player_str;
    if is_martial_artist {
        chance += 5;
    }
    //
    rng.rn2(20) < chance / 2
}

///
pub fn wall_kick_self_damage(wearing_boots: bool, rng: &mut NetHackRng) -> i32 {
    let base = rng.d(1, 3);
    if wearing_boots {
        (base - 1).max(1)
    } else {
        base + 1
    }
}

///
pub fn try_kick_boulder(player_str: i32, is_levitating: bool, rng: &mut NetHackRng) -> KickResult {
    if is_levitating {
        return KickResult {
            target: KickTarget::Boulder,
            damage_to_target: 0,
            damage_to_self: 0,
            message: "You can't kick while levitating.".to_string(),
            success: false,
        };
    }

    let success = rng.rn2(20) < player_str / 2;
    if success {
        KickResult {
            target: KickTarget::Boulder,
            damage_to_target: 0,
            damage_to_self: 0,
            message: "You push the boulder!".to_string(),
            success: true,
        }
    } else {
        let self_dmg = rng.d(1, 3);
        KickResult {
            target: KickTarget::Boulder,
            damage_to_target: 0,
            damage_to_self: self_dmg,
            message: "You stub your toe on the boulder!".to_string(),
            success: false,
        }
    }
}

///
pub fn kick_monster_effect(
    monster_name: &str,
    kick_dmg: i32,
    monster_is_acid: bool,
    monster_is_stone: bool,
    wearing_boots: bool,
) -> KickResult {
    let mut result = KickResult {
        target: KickTarget::Monster,
        damage_to_target: kick_dmg,
        damage_to_self: 0,
        message: format!("You kick the {}!", monster_name),
        success: true,
    };

    //
    if monster_is_acid {
        if wearing_boots {
            result.message = format!("You kick the {}! Your boots are corroded!", monster_name);
        } else {
            result.damage_to_self = 5;
            result.message = format!("You kick the {}! The acid burns your foot!", monster_name);
        }
    }

    //
    if monster_is_stone && !wearing_boots {
        result.damage_to_self = 0;
        result.message = format!("You kick the {}! You begin turning to stone!", monster_name);
    }

    result
}

///
pub fn kick_furniture_message(furniture: &str) -> &'static str {
    match furniture {
        "fountain" => "You splash water by kicking the fountain!",
        "throne" => "STRSTRSTR! You kick the throne!",
        "altar" => "You kick the altar. Bad idea!",
        "grave" => "You rudely kick the headstone.",
        "tree" => "You kick the tree. It hurts!",
        "drawbridge" => "THUD! The drawbridge doesn't budge.",
        "ladder_up" | "ladder_down" => "You kick the ladder rungs.",
        _ => "You kick at the furniture.",
    }
}

///
pub fn kicked_object_distance(item_weight: i32, player_str: i32, rng: &mut NetHackRng) -> i32 {
    let base = player_str / 4 - item_weight / 100;
    let dist = base + rng.rn2(3);
    dist.clamp(1, 7)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct HurtleResult {
    pub new_x: i32,
    pub new_y: i32,
    pub distance: i32,
    pub hit_wall: bool,
    pub hit_monster: bool,
    pub message: String,
}

///
pub fn calculate_hurtle(
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    max_distance: i32,
    map_width: i32,
    map_height: i32,
) -> HurtleResult {
    let mut x = start_x;
    let mut y = start_y;
    let mut dist = 0;
    let mut hit_wall = false;

    for _ in 0..max_distance {
        let nx = x + dx;
        let ny = y + dy;

        //
        if nx < 1 || nx >= map_width - 1 || ny < 1 || ny >= map_height - 1 {
            hit_wall = true;
            break;
        }

        x = nx;
        y = ny;
        dist += 1;
    }

    HurtleResult {
        new_x: x,
        new_y: y,
        distance: dist,
        hit_wall,
        hit_monster: false,
        message: if hit_wall {
            "You slam into a wall!".to_string()
        } else {
            format!("You slide {} spaces.", dist)
        },
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn try_kick_chest(player_str: i32, chest_locked: bool, rng: &mut NetHackRng) -> KickResult {
    if !chest_locked {
        return KickResult {
            target: KickTarget::Object,
            success: false,
            damage_to_target: 0,
            damage_to_self: 0,
            message: "The chest is already unlocked.".to_string(),
        };
    }

    //
    let chance = player_str * 3 + rng.rn2(50);
    if chance >= 60 {
        KickResult {
            target: KickTarget::Object,
            success: true,
            damage_to_target: 5,
            damage_to_self: 0,
            message: "The lock on the chest breaks open!".to_string(),
        }
    } else {
        KickResult {
            target: KickTarget::Object,
            success: false,
            damage_to_target: 0,
            damage_to_self: rng.rn2(3) + 1,
            message: "You hurt your foot trying to kick the chest open.".to_string(),
        }
    }
}

///
pub fn kick_over_trap(trap_name: &str, rng: &mut NetHackRng) -> KickResult {
    let success_chance = rng.rn2(10);

    if success_chance >= 7 {
        //
        KickResult {
            target: KickTarget::Object,
            success: true,
            damage_to_target: 0,
            damage_to_self: 0,
            message: format!("You kick the {} and it clicks harmlessly.", trap_name),
        }
    } else if success_chance >= 3 {
        //
        KickResult {
            target: KickTarget::Object,
            success: false,
            damage_to_target: 0,
            damage_to_self: 0,
            message: format!("You kick the {} but nothing happens.", trap_name),
        }
    } else {
        //
        KickResult {
            target: KickTarget::Object,
            success: false,
            damage_to_target: 0,
            damage_to_self: rng.rn2(6) + 2,
            message: format!("You trigger the {} by kicking it!", trap_name),
        }
    }
}

///
pub fn kick_fountain_effect(rng: &mut NetHackRng) -> &'static str {
    let roll = rng.rn2(20);
    match roll {
        0..=3 => "A gush of water hits you!",
        4..=7 => "The fountain spurts!",
        8..=10 => "Something splashes!",
        11..=14 => "You get your feet wet.",
        15..=17 => "The fountain bubbles.",
        _ => "You hear a loud 'clang'!",
    }
}

///
pub fn kick_sink_effect(rng: &mut NetHackRng) -> &'static str {
    let roll = rng.rn2(15);
    match roll {
        0..=3 => "You hear the pipes rattle.",
        4..=7 => "Something comes loose!",
        8..=10 => "The water runs for a moment.",
        _ => "You stub your toe on the sink.",
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootsType {
    None,
    Iron,
    Speed,
    Levitation,
    Fumble,
    Water,
    Elven,
    Kicking,
}

///
pub fn boots_kick_bonus(boots: BootsType) -> i32 {
    match boots {
        BootsType::None => 0,
        BootsType::Iron => 3,
        BootsType::Speed => 1,
        BootsType::Levitation => -2,
        BootsType::Fumble => -5,
        BootsType::Water => 0,
        BootsType::Elven => 1,
        BootsType::Kicking => 5,
    }
}

///
pub fn boots_protect_foot(boots: BootsType) -> bool {
    matches!(
        boots,
        BootsType::Iron | BootsType::Kicking | BootsType::Elven
    )
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct KickStatistics {
    pub total_kicks: u32,
    pub successful_kicks: u32,
    pub doors_broken: u32,
    pub boulders_moved: u32,
    pub self_damage_taken: u32,
    pub monsters_kicked: u32,
}

impl KickStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_kick(&mut self, success: bool, target: KickTarget) {
        self.total_kicks += 1;
        if success {
            self.successful_kicks += 1;
        }
        match target {
            KickTarget::Door => {
                if success {
                    self.doors_broken += 1;
                }
            }
            KickTarget::Boulder => {
                if success {
                    self.boulders_moved += 1;
                }
            }
            KickTarget::Monster => {
                self.monsters_kicked += 1;
            }
            _ => {}
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_kicks == 0 {
            0.0
        } else {
            (self.successful_kicks as f64) / (self.total_kicks as f64) * 100.0
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod kick_extended_tests {
    use super::*;

    #[test]
    fn test_hurtle() {
        let r = calculate_hurtle(40, 10, 1, 0, 5, 80, 25);
        assert!(r.distance > 0);
        assert!(!r.hit_wall);
    }

    #[test]
    fn test_hurtle_wall() {
        let r = calculate_hurtle(78, 10, 1, 0, 5, 80, 25);
        assert!(r.hit_wall);
    }

    #[test]
    fn test_kick_chest() {
        let mut rng = NetHackRng::new(42);
        let r = try_kick_chest(18, true, &mut rng);
        //
        assert!(!r.message.is_empty());
    }

    #[test]
    fn test_kick_trap() {
        let mut rng = NetHackRng::new(42);
        let r = kick_over_trap("bear trap", &mut rng);
        assert!(!r.message.is_empty());
    }

    #[test]
    fn test_boots_bonus() {
        assert_eq!(boots_kick_bonus(BootsType::Kicking), 5);
        assert!(boots_kick_bonus(BootsType::Fumble) < 0);
    }

    #[test]
    fn test_boots_protect() {
        assert!(boots_protect_foot(BootsType::Iron));
        assert!(!boots_protect_foot(BootsType::Fumble));
    }

    #[test]
    fn test_kick_stats() {
        let mut stats = KickStatistics::new();
        stats.record_kick(true, KickTarget::Door);
        stats.record_kick(false, KickTarget::Wall);
        assert_eq!(stats.total_kicks, 2);
        assert_eq!(stats.doors_broken, 1);
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
pub fn boulder_push_distance(strength: i32, is_levitating: bool) -> i32 {
    let base = strength / 5;
    if is_levitating {
        0
    }
    //
    else {
        base.max(1).min(5)
    }
}

///
pub fn kick_retaliation_chance(target_level: i32, player_level: i32) -> bool {
    //
    target_level > player_level
}

///
pub fn wall_kick_strength_damage(strength: i32, has_boots: bool) -> i32 {
    let base = if has_boots { 1 } else { 3 };
    //
    if strength > 18 {
        base + 3
    } else if strength > 14 {
        base + 2
    } else if strength > 10 {
        base + 1
    } else {
        base
    }
}

///
pub fn martial_arts_kick_bonus(skill_level: i32) -> i32 {
    match skill_level {
        0 => 0,
        1 => 1,
        2 => 3,
        3 => 5,
        4 => 8,
        _ => 10,
    }
}

///
pub fn max_kickable_lock_level(strength: i32, has_boots: bool) -> i32 {
    let base = strength / 3;
    let boot_bonus = if has_boots { 3 } else { 0 };
    base + boot_bonus
}

///
#[derive(Debug, Clone, Default)]
pub struct KickExtendedStats {
    pub monsters_kicked: u32,
    pub doors_kicked: u32,
    pub boulders_pushed: u32,
    pub retaliations: u32,
    pub self_injuries: u32,
    pub martial_kicks: u32,
}

impl KickExtendedStats {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_kick_monster(&mut self) {
        self.monsters_kicked += 1;
    }
    pub fn record_retaliation(&mut self) {
        self.retaliations += 1;
    }
    pub fn record_self_injury(&mut self) {
        self.self_injuries += 1;
    }
}

#[cfg(test)]
mod kick_advanced_tests {
    use super::*;

    #[test]
    fn test_boulder_push() {
        assert!(boulder_push_distance(18, false) > boulder_push_distance(8, false));
        assert_eq!(boulder_push_distance(18, true), 0);
    }

    #[test]
    fn test_retaliation() {
        assert!(kick_retaliation_chance(15, 5));
        assert!(!kick_retaliation_chance(3, 10));
    }

    #[test]
    fn test_wall_damage() {
        let with_boots = wall_kick_strength_damage(18, true);
        let without = wall_kick_strength_damage(18, false);
        assert!(without > with_boots);
    }

    #[test]
    fn test_martial_bonus() {
        assert!(martial_arts_kick_bonus(4) > martial_arts_kick_bonus(1));
    }

    #[test]
    fn test_lock_level() {
        assert!(max_kickable_lock_level(18, true) > max_kickable_lock_level(10, false));
    }

    #[test]
    fn test_kick_ext_stats() {
        let mut s = KickExtendedStats::new();
        s.record_kick_monster();
        s.record_retaliation();
        assert_eq!(s.monsters_kicked, 1);
        assert_eq!(s.retaliations, 1);
    }
}
