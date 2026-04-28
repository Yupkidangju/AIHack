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

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{COLNO, ROWNO};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigTool {
    Pickaxe,
    DwarvenMatk,
    Shovel,
    WandDigging,
    DrumEarthq,
    Hands,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigDirection {
    North,
    South,
    East,
    West,
    Down,
    Up,
}

///
#[derive(Debug, Clone)]
pub struct DigResult {
    pub success: bool,
    pub progress: i32,
    pub completed: bool,
    pub tile_changed: bool,
    pub new_tile: Option<TileType>,
    pub target_x: i32,
    pub target_y: i32,
    pub message: String,
    pub turns_needed: i32,
}

impl DigResult {
    pub fn fail(msg: &str) -> Self {
        Self {
            success: false,
            progress: 0,
            completed: false,
            tile_changed: false,
            new_tile: None,
            target_x: 0,
            target_y: 0,
            message: msg.to_string(),
            turns_needed: 0,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn can_dig(tile: TileType, direction: DigDirection) -> bool {
    match direction {
        DigDirection::Down => {
            //
            matches!(tile, TileType::Room | TileType::Corr)
        }
        DigDirection::Up => {
            //
            matches!(tile, TileType::Room | TileType::Corr)
        }
        _ => {
            //
            matches!(
                tile,
                TileType::VWall
                    | TileType::HWall
                    | TileType::TlCorner
                    | TileType::TrCorner
                    | TileType::BlCorner
                    | TileType::BrCorner
                    | TileType::CrossWall
                    | TileType::TuWall
                    | TileType::TdWall
                    | TileType::TlWall
                    | TileType::TrWall
                    | TileType::SDoor
                    | TileType::SCorr
                    | TileType::DbWall
                    | TileType::Door
            )
        }
    }
}

///
pub fn dig_speed(tool: DigTool) -> i32 {
    match tool {
        DigTool::Pickaxe => 15,
        DigTool::DwarvenMatk => 25,
        DigTool::Shovel => 10,
        DigTool::WandDigging => 100,
        DigTool::DrumEarthq => 100,
        DigTool::Hands => 5,
    }
}

///
pub fn wall_hardness(tile: TileType) -> i32 {
    match tile {
        TileType::VWall | TileType::HWall => 100,
        TileType::TlCorner | TileType::TrCorner | TileType::BlCorner | TileType::BrCorner => 120,
        TileType::CrossWall => 150,
        TileType::DbWall => 200,
        TileType::Door => 50,
        TileType::SDoor => 60,
        TileType::SCorr => 80,
        _ => 100,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn dig_horizontal(
    tile: TileType,
    tool: DigTool,
    current_progress: i32,
    player_str: i32,
    enchantment: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> DigResult {
    if !can_dig(tile, DigDirection::East) {
        let msg = "This spot cannot be dug into.";
        log.add(msg, turn);
        return DigResult::fail(msg);
    }

    let speed = dig_speed(tool);
    let hardness = wall_hardness(tile);

    //
    let str_bonus = (player_str - 10).max(0);
    let ench_bonus = enchantment.max(0) * 5;
    let progress_gain = speed + str_bonus + ench_bonus + rng.rn2(10);

    let new_progress = current_progress + progress_gain;

    if new_progress >= hardness {
        //
        let new_tile = match tile {
            TileType::SDoor | TileType::Door => TileType::OpenDoor,
            _ => TileType::Corr,
        };
        let msg = dig_complete_message(tool, tile);
        log.add(&msg, turn);
        DigResult {
            success: true,
            progress: hardness,
            completed: true,
            tile_changed: true,
            new_tile: Some(new_tile),
            target_x: 0,
            target_y: 0,
            message: msg,
            turns_needed: 0,
        }
    } else {
        //
        let remaining = hardness - new_progress;
        let est_turns = remaining / progress_gain.max(1);
        let msg = dig_progress_message(tool, new_progress, hardness);
        log.add(&msg, turn);
        DigResult {
            success: true,
            progress: new_progress,
            completed: false,
            tile_changed: false,
            new_tile: None,
            target_x: 0,
            target_y: 0,
            message: msg,
            turns_needed: est_turns,
        }
    }
}

///
pub fn dig_downward(
    tile: TileType,
    tool: DigTool,
    current_progress: i32,
    player_str: i32,
    enchantment: i32,
    is_bottom_level: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> DigResult {
    if !can_dig(tile, DigDirection::Down) {
        return DigResult::fail("The floor here is too hard to dig.");
    }
    if is_bottom_level {
        let msg = "The floor here is too hard to dig.";
        log.add(msg, turn);
        return DigResult::fail(msg);
    }

    let speed = dig_speed(tool);
    let hardness = 150;
    let str_bonus = (player_str - 10).max(0);
    let ench_bonus = enchantment.max(0) * 5;
    let progress_gain = speed + str_bonus + ench_bonus + rng.rn2(10);
    let new_progress = current_progress + progress_gain;

    if new_progress >= hardness {
        let msg = "You dig a hole in the floor!";
        log.add(msg, turn);
        DigResult {
            success: true,
            progress: hardness,
            completed: true,
            tile_changed: true,
            new_tile: Some(TileType::Hole),
            target_x: 0,
            target_y: 0,
            message: msg.to_string(),
            turns_needed: 0,
        }
    } else {
        let msg = format!(
            "You continue digging downward. ({}/{})",
            new_progress, hardness
        );
        log.add(&msg, turn);
        DigResult {
            success: true,
            progress: new_progress,
            completed: false,
            tile_changed: false,
            new_tile: None,
            target_x: 0,
            target_y: 0,
            message: msg,
            turns_needed: (hardness - new_progress) / progress_gain.max(1),
        }
    }
}

///
pub fn dig_upward(
    tile: TileType,
    tool: DigTool,
    current_progress: i32,
    player_str: i32,
    enchantment: i32,
    is_top_level: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> DigResult {
    if !can_dig(tile, DigDirection::Up) {
        return DigResult::fail("You can't dig up here.");
    }
    if is_top_level {
        let msg = "You hit the ceiling but nothing happens.";
        log.add(msg, turn);
        return DigResult::fail(msg);
    }
    if tool == DigTool::Shovel {
        let msg = "You can't dig upward with a shovel.";
        log.add(msg, turn);
        return DigResult::fail(msg);
    }

    let speed = dig_speed(tool);
    let hardness = 200;
    let str_bonus = (player_str - 10).max(0);
    let ench_bonus = enchantment.max(0) * 5;
    let progress_gain = speed + str_bonus + ench_bonus + rng.rn2(10);
    let new_progress = current_progress + progress_gain;

    if new_progress >= hardness {
        let msg = "You dig a passage upward!";
        log.add(msg, turn);
        DigResult {
            success: true,
            progress: hardness,
            completed: true,
            tile_changed: true,
            new_tile: Some(TileType::Hole),
            target_x: 0,
            target_y: 0,
            message: msg.to_string(),
            turns_needed: 0,
        }
    } else {
        let msg = format!(
            "You continue digging upward. ({}/{})",
            new_progress, hardness
        );
        log.add(&msg, turn);
        DigResult {
            success: true,
            progress: new_progress,
            completed: false,
            tile_changed: false,
            new_tile: None,
            target_x: 0,
            target_y: 0,
            message: msg,
            turns_needed: (hardness - new_progress) / progress_gain.max(1),
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn zap_dig(
    grid: &mut crate::core::dungeon::Grid,
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    range: i32,
    log: &mut GameLog,
    turn: u64,
) -> Vec<(i32, i32)> {
    let mut changed = Vec::new();

    if dx == 0 && dy == 0 {
        //
        let ux = start_x as usize;
        let uy = start_y as usize;
        if ux < COLNO && uy < ROWNO {
            if matches!(grid.locations[ux][uy].typ, TileType::Room | TileType::Corr) {
                grid.locations[ux][uy].typ = TileType::Hole;
                changed.push((start_x, start_y));
                log.add("You dig a hole in the floor!", turn);
            }
        }
        return changed;
    }

    //
    let mut x = start_x;
    let mut y = start_y;

    for _ in 0..range {
        x += dx;
        y += dy;
        if x < 0 || y < 0 || x as usize >= COLNO || y as usize >= ROWNO {
            break;
        }
        let ux = x as usize;
        let uy = y as usize;
        let tile = grid.locations[ux][uy].typ;

        if can_dig(
            tile,
            if dx != 0 {
                DigDirection::East
            } else {
                DigDirection::North
            },
        ) {
            grid.locations[ux][uy].typ = match tile {
                TileType::SDoor | TileType::Door => TileType::OpenDoor,
                _ => TileType::Corr,
            };
            grid.locations[ux][uy].seenv = 0xFF;
            changed.push((x, y));
        } else if matches!(tile, TileType::Room | TileType::Corr | TileType::OpenDoor) {
            //
            continue;
        } else {
            //
            break;
        }
    }

    if !changed.is_empty() {
        log.add(
            &format!(
                "The beam of digging blasts through {} wall{}!",
                changed.len(),
                if changed.len() != 1 { "s" } else { "" }
            ),
            turn,
        );
    }
    changed
}

// =============================================================================
//
// =============================================================================

///
pub fn break_boulder(tool: DigTool, rng: &mut NetHackRng, log: &mut GameLog, turn: u64) -> bool {
    let success = match tool {
        DigTool::Pickaxe | DigTool::DwarvenMatk => rng.rn2(3) != 0,
        DigTool::WandDigging => true,
        _ => false,
    };

    if success {
        log.add("The boulder crumbles to pieces.", turn);
    } else {
        log.add("You chip away at the boulder.", turn);
    }
    success
}

// =============================================================================
//
// =============================================================================

///
pub fn dig_up_grave(rng: &mut NetHackRng, log: &mut GameLog, turn: u64) -> GraveContents {
    //
    let roll = rng.rn2(10);
    if roll < 3 {
        log.add_colored("A zombie rises from the grave!", [100, 200, 100], turn);
        GraveContents::Zombie
    } else if roll < 6 {
        log.add("You unearth some treasure!", turn);
        GraveContents::Treasure
    } else if roll < 8 {
        log.add("You unearth a corpse.", turn);
        GraveContents::Corpse
    } else {
        log.add("The grave is empty.", turn);
        GraveContents::Empty
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraveContents {
    Zombie,
    Treasure,
    Corpse,
    Empty,
}

// =============================================================================
//
// =============================================================================

///
pub fn do_earthquake(
    grid: &mut crate::core::dungeon::Grid,
    center_x: i32,
    center_y: i32,
    intensity: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> i32 {
    let mut destroyed = 0;
    let range = intensity.min(5);

    log.add_colored("The ground shakes violently!", [255, 150, 50], turn);

    for dx in -range..=range {
        for dy in -range..=range {
            let x = center_x + dx;
            let y = center_y + dy;
            if x < 1 || y < 1 || x as usize >= COLNO - 1 || y as usize >= ROWNO - 1 {
                continue;
            }
            let ux = x as usize;
            let uy = y as usize;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > range * range {
                continue;
            }

            //
            let chance = (range * range - dist_sq + 1).max(1);
            if rng.rn2(chance * 2) != 0 {
                continue;
            }

            let tile = grid.locations[ux][uy].typ;
            match tile {
                TileType::VWall
                | TileType::HWall
                | TileType::TlCorner
                | TileType::TrCorner
                | TileType::BlCorner
                | TileType::BrCorner => {
                    grid.locations[ux][uy].typ = TileType::Corr;
                    grid.locations[ux][uy].seenv = 0xFF;
                    destroyed += 1;
                }
                TileType::Door | TileType::SDoor => {
                    grid.locations[ux][uy].typ = TileType::OpenDoor;
                    destroyed += 1;
                }
                _ => {}
            }
        }
    }

    if destroyed > 0 {
        log.add(
            &format!(
                "{} wall{} crumble{}!",
                destroyed,
                if destroyed != 1 { "s" } else { "" },
                if destroyed != 1 { "" } else { "s" }
            ),
            turn,
        );
    }
    destroyed
}

// =============================================================================
//
// =============================================================================

///
fn dig_complete_message(tool: DigTool, tile: TileType) -> String {
    let tool_name = match tool {
        DigTool::Pickaxe => "pick-axe",
        DigTool::DwarvenMatk => "mattock",
        DigTool::Shovel => "shovel",
        DigTool::WandDigging => "beam of digging",
        DigTool::DrumEarthq => "earthquake",
        DigTool::Hands => "hands",
    };
    let what = match tile {
        TileType::VWall | TileType::HWall => "wall",
        TileType::Door | TileType::SDoor => "door",
        TileType::SCorr => "passage",
        _ => "obstacle",
    };
    format!(
        "You succeed in digging through the {} with your {}.",
        what, tool_name
    )
}

///
fn dig_progress_message(tool: DigTool, progress: i32, max: i32) -> String {
    let pct = progress * 100 / max.max(1);
    if pct < 30 {
        "You make some progress.".to_string()
    } else if pct < 60 {
        "You continue digging.".to_string()
    } else if pct < 90 {
        "The wall is weakening.".to_string()
    } else {
        "Almost through!".to_string()
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_dig() {
        assert!(can_dig(TileType::VWall, DigDirection::East));
        assert!(can_dig(TileType::HWall, DigDirection::North));
        assert!(!can_dig(TileType::Room, DigDirection::East));
        assert!(can_dig(TileType::Room, DigDirection::Down));
    }

    #[test]
    fn test_dig_speed() {
        assert!(dig_speed(DigTool::DwarvenMatk) > dig_speed(DigTool::Pickaxe));
        assert_eq!(dig_speed(DigTool::WandDigging), 100);
    }

    #[test]
    fn test_wall_hardness() {
        assert!(wall_hardness(TileType::DbWall) > wall_hardness(TileType::VWall));
        assert!(wall_hardness(TileType::Door) < wall_hardness(TileType::VWall));
    }

    #[test]
    fn test_break_boulder() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        //
        assert!(break_boulder(DigTool::WandDigging, &mut rng, &mut log, 1));
    }

    #[test]
    fn test_grave() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let contents = dig_up_grave(&mut rng, &mut log, 1);
        assert!(matches!(
            contents,
            GraveContents::Zombie
                | GraveContents::Treasure
                | GraveContents::Corpse
                | GraveContents::Empty
        ));
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
pub fn dig_direction_delta(dir: DigDirection) -> (i32, i32) {
    match dir {
        DigDirection::North => (0, -1),
        DigDirection::South => (0, 1),
        DigDirection::East => (1, 0),
        DigDirection::West => (-1, 0),
        DigDirection::Down | DigDirection::Up => (0, 0),
    }
}

///
pub fn terrain_after_dig(tile: &str) -> &'static str {
    match tile {
        "wall" | "stone" => "corridor",
        "door" => "floor",
        "tree" => "floor",
        "drawbridge" => "moat",
        _ => "pit",
    }
}

///
pub fn tunnel_stability(depth: i32, adjacent_walls: i32) -> bool {
    //
    if depth >= 20 {
        adjacent_walls >= 3
    } else {
        adjacent_walls >= 2
    }
}

///
pub fn mineral_discovery_chance(depth: i32, tool: DigTool) -> i32 {
    let base = match tool {
        DigTool::Pickaxe => 5,
        DigTool::DwarvenMatk => 10,
        DigTool::WandDigging => 2,
        _ => 0,
    };
    base + depth / 5
}

///
pub fn dig_exhaustion(consecutive_digs: i32) -> i32 {
    //
    (consecutive_digs * 2).min(20)
}

///
pub fn mineral_found_message(rng: &mut NetHackRng) -> &'static str {
    let roll = rng.rn2(20);
    if roll == 0 {
        "You found a gem!"
    } else if roll < 3 {
        "You found some gold!"
    } else if roll < 5 {
        "You found a rock."
    } else {
        ""
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct DigStatistics {
    pub walls_dug: u32,
    pub floors_dug: u32,
    pub boulders_smashed: u32,
    pub minerals_found: u32,
    pub graves_dug: u32,
    pub total_turns_digging: u32,
}

impl DigStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_dig(&mut self, is_wall: bool) {
        if is_wall {
            self.walls_dug += 1;
        } else {
            self.floors_dug += 1;
        }
        self.total_turns_digging += 1;
    }
}

#[cfg(test)]
mod dig_extended_tests {
    use super::*;

    #[test]
    fn test_direction_delta() {
        assert_eq!(dig_direction_delta(DigDirection::North), (0, -1));
        assert_eq!(dig_direction_delta(DigDirection::Down), (0, 0));
    }

    #[test]
    fn test_terrain_after() {
        assert_eq!(terrain_after_dig("wall"), "corridor");
        assert_eq!(terrain_after_dig("door"), "floor");
    }

    #[test]
    fn test_tunnel_stability() {
        assert!(tunnel_stability(5, 2));
        assert!(!tunnel_stability(25, 2));
    }

    #[test]
    fn test_mineral_chance() {
        assert!(
            mineral_discovery_chance(20, DigTool::DwarvenMatk)
                > mineral_discovery_chance(1, DigTool::Pickaxe)
        );
    }

    #[test]
    fn test_dig_stats() {
        let mut s = DigStatistics::new();
        s.record_dig(true);
        s.record_dig(false);
        assert_eq!(s.walls_dug, 1);
        assert_eq!(s.total_turns_digging, 2);
    }
}
