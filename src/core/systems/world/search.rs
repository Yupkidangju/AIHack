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
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::core::entity::{PlayerTag, Position, Trap};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

///
///
pub fn try_search(
    world: &mut SubWorld,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    rumors: &crate::core::systems::talk::Rumors,
) -> bool {
    let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
    let p_pos = p_query.iter(world).next().copied();

    if let Some(pos) = p_pos {
        let mut found_something = false;

        //
        for dy in -1..=1 {
            for dx in -1..=1 {
                let tx = pos.x + dx;
                let ty = pos.y + dy;

                if tx < 0 || tx >= 80 || ty < 0 || ty >= 21 {
                    continue;
                }

                //
                let mut trap_query = <(&Position, &mut Trap)>::query();
                for (t_pos, trap) in trap_query.iter_mut(world) {
                    if t_pos.x == tx && t_pos.y == ty && !trap.discovered {
                        trap.discovered = true;
                        log.add("You find a trap!", turn);
                        found_something = true;
                    }
                }

                //
                if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
                    if tile.typ == TileType::SDoor {
                        tile.typ = TileType::Door;
                        log.add("You find a secret door!", turn);
                        found_something = true;
                    }
                }
            }
        }

        //
        if !found_something && rng.rn2(5) == 0 {
            let msg = rumors.get_random_engraving(rng);
            log.add(
                format!("You see some writing on the floor: '{}'", msg),
                turn,
            );
            found_something = true;
        }

        if !found_something {
            log.add("You find nothing.", turn);
        }
        return true;
    }
    false
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
///
pub fn search_success_rate(
    base_chance: i32,
    luck: i32,
    wisdom: i32,
    search_skill: i32,
    consecutive_searches: u32,
) -> i32 {
    let mut chance = base_chance;

    //
    if wisdom >= 15 {
        chance += (wisdom - 14) * 2;
    }

    //
    chance += luck * 3;

    //
    chance += search_skill * 5;

    //
    if consecutive_searches > 1 {
        chance += (consecutive_searches as i32 - 1) * 5;
    }

    chance.clamp(5, 95)
}

///
pub fn trap_detection_difficulty(trap_name: &str) -> i32 {
    match trap_name {
        "pit" | "spiked pit" => 20,
        "bear trap" => 30,
        "land mine" => 40,
        "rolling boulder" => 35,
        "arrow trap" | "dart trap" => 45,
        "falling rock trap" => 50,
        "squeaky board" => 25,
        "teleportation trap" => 60,
        "level teleporter" => 65,
        "fire trap" => 55,
        "sleeping gas trap" => 50,
        "magic trap" => 70,
        "anti-magic field" => 75,
        "polymorph trap" => 70,
        "web" => 15,
        "statue trap" => 80,
        "magic portal" => 90,
        _ => 50,
    }
}

///
pub fn secret_passage_difficulty(wall_type: &str) -> i32 {
    match wall_type {
        "secret_door" => 40,
        "secret_corridor" => 50,
        "hidden_staircase" => 70,
        _ => 45,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResult {
    Nothing,
    TrapFound(String),
    SecretDoorFound,
    SecretCorridorFound,
    HiddenStaircaseFound,
    EngravingFound(String),
    MimicDetected,
    GoldFound(u32),
}

///
pub fn search_result_message(result: &SearchResult) -> String {
    match result {
        SearchResult::Nothing => "You find nothing.".to_string(),
        SearchResult::TrapFound(name) => format!("You find a {}!", name),
        SearchResult::SecretDoorFound => "You find a secret door!".to_string(),
        SearchResult::SecretCorridorFound => "You find a hidden corridor!".to_string(),
        SearchResult::HiddenStaircaseFound => "You find a hidden staircase!".to_string(),
        SearchResult::EngravingFound(text) => {
            format!("You see some writing on the floor: '{}'", text)
        }
        SearchResult::MimicDetected => "That's a mimic!".to_string(),
        SearchResult::GoldFound(amount) => format!("You find {} gold pieces!", amount),
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn search_radius(base: i32, has_enhanced_search: bool) -> i32 {
    if has_enhanced_search {
        (base + 1).min(3)
    } else {
        base.min(2)
    }
}

///
///
pub fn auto_search_chance(luck: i32, has_searching: bool) -> i32 {
    if !has_searching {
        return 0;
    }
    let base = 15 + luck * 2;
    base.clamp(5, 40)
}

///
pub fn detect_mimic_chance(distance: i32, luck: i32, rng: &mut NetHackRng) -> bool {
    if distance > 1 {
        return false;
    }
    let chance = 20 + luck * 3;
    rng.rn2(100) < chance.clamp(5, 60)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct SearchStatistics {
    pub total_searches: u32,
    pub traps_found: u32,
    pub secret_doors_found: u32,
    pub secret_corridors_found: u32,
    pub mimics_detected: u32,
    pub nothing_found: u32,
    pub auto_searches: u32,
}

impl SearchStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_search(&mut self, result: &SearchResult) {
        self.total_searches += 1;
        match result {
            SearchResult::Nothing => self.nothing_found += 1,
            SearchResult::TrapFound(_) => self.traps_found += 1,
            SearchResult::SecretDoorFound => self.secret_doors_found += 1,
            SearchResult::SecretCorridorFound => self.secret_corridors_found += 1,
            SearchResult::MimicDetected => self.mimics_detected += 1,
            _ => {}
        }
    }

    pub fn discovery_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        let found = self.traps_found + self.secret_doors_found + self.secret_corridors_found;
        (found as f64) / (self.total_searches as f64) * 100.0
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod search_extended_tests {
    use super::*;

    #[test]
    fn test_search_success_rate() {
        let rate = search_success_rate(30, 3, 18, 2, 1);
        assert!(rate > 30);
        assert!(rate <= 95);
    }

    #[test]
    fn test_consecutive_bonus() {
        let r1 = search_success_rate(30, 0, 10, 0, 1);
        let r5 = search_success_rate(30, 0, 10, 0, 5);
        assert!(r5 > r1);
    }

    #[test]
    fn test_trap_difficulty() {
        assert!(trap_detection_difficulty("web") < trap_detection_difficulty("magic trap"));
        assert!(trap_detection_difficulty("pit") < trap_detection_difficulty("statue trap"));
    }

    #[test]
    fn test_search_result_message() {
        let r = SearchResult::TrapFound("bear trap".to_string());
        assert!(search_result_message(&r).contains("bear trap"));
    }

    #[test]
    fn test_search_radius() {
        assert_eq!(search_radius(1, false), 1);
        assert_eq!(search_radius(1, true), 2);
        assert_eq!(search_radius(3, true), 3);
    }

    #[test]
    fn test_auto_search() {
        assert_eq!(auto_search_chance(0, false), 0);
        assert!(auto_search_chance(3, true) > 0);
    }

    #[test]
    fn test_search_stats() {
        let mut stats = SearchStatistics::new();
        stats.record_search(&SearchResult::TrapFound("pit".to_string()));
        stats.record_search(&SearchResult::Nothing);
        assert_eq!(stats.total_searches, 2);
        assert_eq!(stats.traps_found, 1);
        assert!(stats.discovery_rate() > 0.0);
    }
}
