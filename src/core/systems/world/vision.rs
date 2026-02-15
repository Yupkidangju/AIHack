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
use crate::core::dungeon::tile::{Tile, TileType};
use crate::core::dungeon::{Grid, COLNO, ROWNO};

///
pub const COULD_SEE: u8 = 0x01;
pub const IN_SIGHT: u8 = 0x02;
pub const TEMP_LIT: u8 = 0x04;
pub const MEMORIZED: u8 = 0x08;

pub struct VisionSystem {
    ///
    pub viz_array: [[u8; ROWNO]; COLNO],
    ///
    pub viz_clear: [[bool; ROWNO]; COLNO],
}

impl VisionSystem {
    pub fn new() -> Self {
        Self {
            viz_array: [[0; ROWNO]; COLNO],
            viz_clear: [[false; ROWNO]; COLNO],
        }
    }

    ///
    pub fn does_block(tile: &Tile) -> bool {
        match tile.typ {
            TileType::Stone | TileType::Tree => true,
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
            | TileType::DbWall => true,
            TileType::Door => true,
            _ => false,
        }
    }

    ///
    pub fn update_clear_zones(&mut self, grid: &Grid) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if let Some(tile) = grid.get_tile(x, y) {
                    self.viz_clear[x][y] = !Self::does_block(tile);
                }
            }
        }
    }

    ///
    pub fn reset_sight(&mut self) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.viz_array[x][y] &= !IN_SIGHT;
            }
        }
    }

    ///
    pub fn apply_vision(&mut self, grid: &Grid, lx: usize, ly: usize, radius: i32) {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let tx = lx as i32 + dx;
                    let ty = ly as i32 + dy;
                    if tx >= 0 && tx < COLNO as i32 && ty >= 0 && ty < ROWNO as i32 {
                        let ux = tx as usize;
                        let uy = ty as usize;
                        if Self::has_line_of_sight(grid, lx, ly, ux, uy) {
                            self.viz_array[ux][uy] |= IN_SIGHT | MEMORIZED;
                        }
                    }
                }
            }
        }
    }

    ///
    pub fn recalc(&mut self, grid: &Grid, player_x: usize, player_y: usize, radius: i32) {
        self.reset_sight();

        //
        //
        let mut current_room_no = 0;
        if let Some(tile) = grid.get_tile(player_x, player_y) {
            if tile
                .flags
                .contains(crate::core::dungeon::tile::TileFlags::LIT)
                && tile.typ == TileType::Room
            {
                current_room_no = tile.roomno;
            }
        }

        if current_room_no > 0 {
            for x in 0..COLNO {
                for y in 0..ROWNO {
                    if let Some(t) = grid.get_tile(x, y) {
                        if t.roomno == current_room_no {
                            self.viz_array[x][y] |= IN_SIGHT | MEMORIZED;
                        }
                    }
                }
            }
        }

        self.apply_vision(grid, player_x, player_y, radius);
    }

    pub fn has_line_of_sight(grid: &Grid, x0: usize, y0: usize, x1: usize, y1: usize) -> bool {
        let mut x = x0 as i32;
        let mut y = y0 as i32;
        let dx = (x1 as i32 - x).abs();
        let dy = (y1 as i32 - y).abs();
        let sx = if x < x1 as i32 { 1 } else { -1 };
        let sy = if y < y1 as i32 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x == x1 as i32 && y == y1 as i32 {
                return true;
            }

            //
            if x != x0 as i32 || y != y0 as i32 {
                if let Some(tile) = grid.get_tile(x as usize, y as usize) {
                    if Self::does_block(tile) {
                        return false;
                    }
                }
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    ///
    pub fn magic_map(&mut self) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.viz_array[x][y] |= MEMORIZED;
            }
        }
    }

    // =============================================================================
    // [v2.3.1
    //
    //
    //
    // =============================================================================

    ///
    pub fn apply_temp_light(&mut self, x: usize, y: usize, radius: i32) {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let tx = x as i32 + dx;
                    let ty = y as i32 + dy;
                    if tx >= 0 && tx < COLNO as i32 && ty >= 0 && ty < ROWNO as i32 {
                        self.viz_array[tx as usize][ty as usize] |= TEMP_LIT;
                    }
                }
            }
        }
    }

    ///
    pub fn clear_temp_light(&mut self) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.viz_array[x][y] &= !TEMP_LIT;
            }
        }
    }

    ///
    ///
    pub fn recalc_blind(&mut self, player_x: usize, player_y: usize) {
        self.reset_sight();
        //
        if player_x < COLNO && player_y < ROWNO {
            self.viz_array[player_x][player_y] |= IN_SIGHT | MEMORIZED;
        }
    }

    ///
    ///
    pub fn infravision_detect(
        &self,
        grid: &Grid,
        player_x: usize,
        player_y: usize,
        monster_x: usize,
        monster_y: usize,
        infra_range: i32,
    ) -> bool {
        let dx = (monster_x as i32 - player_x as i32).abs();
        let dy = (monster_y as i32 - player_y as i32).abs();
        if dx * dx + dy * dy > infra_range * infra_range {
            return false;
        }
        //
        Self::has_line_of_sight(grid, player_x, player_y, monster_x, monster_y)
    }

    ///
    ///
    pub fn telepathy_detect(
        player_x: usize,
        player_y: usize,
        monster_x: usize,
        monster_y: usize,
        range: i32,
        monster_mindless: bool,
    ) -> bool {
        if monster_mindless {
            return false;
        }
        if range <= 0 {
            //
            return true;
        }
        let dx = (monster_x as i32 - player_x as i32).abs();
        let dy = (monster_y as i32 - player_y as i32).abs();
        dx * dx + dy * dy <= range * range
    }

    ///
    ///
    pub fn can_see_invisible(
        &self,
        monster_x: usize,
        monster_y: usize,
        has_see_invis: bool,
    ) -> bool {
        if !has_see_invis {
            return false;
        }
        //
        self.viz_array[monster_x][monster_y] & IN_SIGHT != 0
    }

    ///
    pub fn effective_vision_radius(
        base_radius: i32,
        is_dark_level: bool,
        has_night_vision: bool,
        carrying_light: bool,
        light_radius: i32,
    ) -> i32 {
        let mut radius = base_radius;

        if is_dark_level {
            //
            radius = 1;
        }

        if has_night_vision {
            radius = radius.max(3);
        }

        if carrying_light {
            radius = radius.max(light_radius);
        }

        radius.clamp(1, 15)
    }

    ///
    ///
    pub fn clairvoyance(&mut self, center_x: usize, center_y: usize, range: i32) {
        for dx in -range..=range {
            for dy in -range..=range {
                let tx = center_x as i32 + dx;
                let ty = center_y as i32 + dy;
                if tx >= 0 && tx < COLNO as i32 && ty >= 0 && ty < ROWNO as i32 {
                    self.viz_array[tx as usize][ty as usize] |= IN_SIGHT | MEMORIZED | TEMP_LIT;
                }
            }
        }
    }

    ///
    pub fn debug_vision_stats(&self) -> (usize, usize, usize) {
        let mut in_sight_count = 0;
        let mut memorized_count = 0;
        let mut temp_lit_count = 0;
        for x in 0..COLNO {
            for y in 0..ROWNO {
                let v = self.viz_array[x][y];
                if v & IN_SIGHT != 0 {
                    in_sight_count += 1;
                }
                if v & MEMORIZED != 0 {
                    memorized_count += 1;
                }
                if v & TEMP_LIT != 0 {
                    temp_lit_count += 1;
                }
            }
        }
        (in_sight_count, memorized_count, temp_lit_count)
    }
}

///
pub fn light_source_radius(item_name: &str) -> i32 {
    let lower = item_name.to_lowercase();
    if lower.contains("candle") {
        1
    } else if lower.contains("lamp") || lower.contains("lantern") {
        3
    } else if lower.contains("magic lamp") || lower.contains("brass lantern") {
        4
    } else if lower.contains("sunsword") || lower.contains("Sun Sword") {
        3
    } else {
        0
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    NormalSight,
    Infravision,
    Telepathy,
    SeeInvisible,
    TrembleSense,
    Warning,
    None,
}

///
pub fn detect_monster(
    in_sight: bool,
    monster_invisible: bool,
    monster_mindless: bool,
    has_see_invis: bool,
    has_infravision: bool,
    has_telepathy: bool,
    has_warning: bool,
    in_infra_range: bool,
    in_telepathy_range: bool,
) -> DetectionMethod {
    //
    if in_sight && !monster_invisible {
        return DetectionMethod::NormalSight;
    }

    //
    if in_sight && monster_invisible && has_see_invis {
        return DetectionMethod::SeeInvisible;
    }

    //
    if has_infravision && in_infra_range && !monster_invisible {
        return DetectionMethod::Infravision;
    }

    //
    if has_telepathy && !monster_mindless && in_telepathy_range {
        return DetectionMethod::Telepathy;
    }

    //
    if has_warning {
        return DetectionMethod::Warning;
    }

    DetectionMethod::None
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Full,
    Partial,
    None,
}

///
pub fn classify_block(tile_type: TileType) -> BlockType {
    match tile_type {
        TileType::Stone | TileType::DbWall => BlockType::Full,
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
        | TileType::TrWall => BlockType::Full,
        TileType::Door => BlockType::Full,
        TileType::Tree => BlockType::Partial,
        _ => BlockType::None,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSourceType {
    Candle,
    OilLamp,
    BrassLantern,
    MagicLamp,
    SunSword,
    WandOfLight,
    SpellLight,
}

///
pub fn light_radius(source: LightSourceType) -> i32 {
    match source {
        LightSourceType::Candle => 1,
        LightSourceType::OilLamp => 2,
        LightSourceType::BrassLantern => 3,
        LightSourceType::MagicLamp => 4,
        LightSourceType::SunSword => 3,
        LightSourceType::WandOfLight => 5,
        LightSourceType::SpellLight => 4,
    }
}

///
pub fn light_duration(source: LightSourceType) -> Option<u32> {
    match source {
        LightSourceType::Candle => Some(400),
        LightSourceType::OilLamp => Some(1500),
        LightSourceType::BrassLantern => Some(3000),
        LightSourceType::MagicLamp => None,
        LightSourceType::SunSword => None,
        LightSourceType::WandOfLight => Some(50),
        LightSourceType::SpellLight => Some(100),
    }
}

///
pub fn light_flicker_message(remaining_pct: f32) -> Option<&'static str> {
    if remaining_pct <= 0.0 {
        Some("Your light goes out!")
    } else if remaining_pct <= 0.1 {
        Some("Your light is about to go out!")
    } else if remaining_pct <= 0.25 {
        Some("Your light is getting dim.")
    } else if remaining_pct <= 0.5 {
        Some("Your light flickers.")
    } else {
        None
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct VisionStatistics {
    pub recalc_count: u32,
    pub blind_turns: u32,
    pub magic_maps_used: u32,
    pub clairvoyance_uses: u32,
    pub monsters_detected_telepathy: u32,
    pub monsters_detected_infravision: u32,
    pub light_sources_used: u32,
}

impl VisionStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_recalc(&mut self) {
        self.recalc_count += 1;
    }
    pub fn record_blind(&mut self) {
        self.blind_turns += 1;
    }
    pub fn record_magic_map(&mut self) {
        self.magic_maps_used += 1;
    }
    pub fn record_clairvoyance(&mut self) {
        self.clairvoyance_uses += 1;
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod vision_extended_tests {
    use super::*;

    #[test]
    fn test_detect_normal() {
        let m = detect_monster(true, false, false, false, false, false, false, false, false);
        assert_eq!(m, DetectionMethod::NormalSight);
    }

    #[test]
    fn test_detect_invisible() {
        let m = detect_monster(true, true, false, true, false, false, false, false, false);
        assert_eq!(m, DetectionMethod::SeeInvisible);
    }

    #[test]
    fn test_detect_telepathy() {
        let m = detect_monster(false, false, false, false, false, true, false, false, true);
        assert_eq!(m, DetectionMethod::Telepathy);
    }

    #[test]
    fn test_detect_mindless_telepathy() {
        let m = detect_monster(false, false, true, false, false, true, false, false, true);
        //
        assert_ne!(m, DetectionMethod::Telepathy);
    }

    #[test]
    fn test_classify_block() {
        assert_eq!(classify_block(TileType::Stone), BlockType::Full);
        assert_eq!(classify_block(TileType::Room), BlockType::None);
        assert_eq!(classify_block(TileType::Tree), BlockType::Partial);
    }

    #[test]
    fn test_light_radius() {
        assert_eq!(light_radius(LightSourceType::Candle), 1);
        assert_eq!(light_radius(LightSourceType::MagicLamp), 4);
    }

    #[test]
    fn test_light_duration() {
        assert!(light_duration(LightSourceType::MagicLamp).is_none());
        assert!(light_duration(LightSourceType::Candle).is_some());
    }

    #[test]
    fn test_flicker_message() {
        assert!(light_flicker_message(0.05).is_some());
        assert!(light_flicker_message(0.8).is_none());
    }

    #[test]
    fn test_vision_stats() {
        let mut stats = VisionStatistics::new();
        stats.record_recalc();
        stats.record_blind();
        stats.record_magic_map();
        assert_eq!(stats.recalc_count, 1);
        assert_eq!(stats.blind_turns, 1);
        assert_eq!(stats.magic_maps_used, 1);
    }
}
