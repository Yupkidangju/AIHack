// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
//
// [v2.3.0
//
//
//
//
//
//
//
//
//
//
//
// =============================================================================

use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct DropResult {
    ///
    pub success: bool,
    /// 硫붿떆吏
    pub message: String,
    ///
    pub count: i32,
    ///
    pub fell_in_liquid: bool,
    ///
    pub gold_amount: i64,
}

///
///
pub fn do_drop(
    item_name: &str,
    item_count: i32,
    is_gold: bool,
    tile_is_pool: bool,
    tile_is_lava: bool,
    tile_is_altar: bool,
    log: &mut GameLog,
    turn: u64,
) -> DropResult {
    let mut result = DropResult {
        success: true,
        message: String::new(),
        count: item_count,
        fell_in_liquid: false,
        gold_amount: 0,
    };

    //
    if tile_is_pool {
        result.fell_in_liquid = true;
        result.message = format!("{} sinks!", item_name);
        log.add(&result.message, turn);
        return result;
    }

    //
    if tile_is_lava {
        result.fell_in_liquid = true;
        result.message = format!("{} is consumed by lava!", item_name);
        log.add_colored(&result.message, [255, 100, 50], turn);
        return result;
    }

    //
    if tile_is_altar {
        result.message = format!("You drop {} on the altar.", item_name);
        log.add_colored(&result.message, [200, 200, 255], turn);
        return result;
    }

    //
    if is_gold {
        result.message = format!("You drop some gold pieces.");
    } else {
        result.message = format!("You drop {}.", item_name);
    }
    log.add(&result.message, turn);
    result
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StairDirection {
    Down,
    Up,
}

///
#[derive(Debug, Clone)]
pub struct StairResult {
    pub success: bool,
    pub direction: StairDirection,
    pub message: String,
    ///
    pub fell: bool,
    ///
    pub escape_attempt: bool,
}

///
pub fn do_stair(
    direction: StairDirection,
    has_stair: bool,
    has_amulet: bool,
    is_top_level: bool,
    is_bottom_level: bool,
    is_levitating: bool,
    log: &mut GameLog,
    turn: u64,
) -> StairResult {
    let mut result = StairResult {
        success: false,
        direction,
        message: String::new(),
        fell: false,
        escape_attempt: false,
    };

    //
    if is_levitating && direction == StairDirection::Down {
        result.message = "You're floating up here. You can't go down.".to_string();
        log.add(&result.message, turn);
        return result;
    }

    //
    if !has_stair {
        result.message = match direction {
            StairDirection::Down => "You can't go down here.".to_string(),
            StairDirection::Up => "You can't go up here.".to_string(),
        };
        log.add(&result.message, turn);
        return result;
    }

    //
    if direction == StairDirection::Up && is_top_level {
        if has_amulet {
            result.escape_attempt = true;
            result.message = "You feel the magical portal opening...".to_string();
            result.success = true;
        } else {
            result.message = "You need the Amulet of Yendor to escape!".to_string();
        }
        log.add(&result.message, turn);
        return result;
    }

    //
    if direction == StairDirection::Down && is_bottom_level {
        result.message = "You can't go any deeper.".to_string();
        log.add(&result.message, turn);
        return result;
    }

    //
    result.success = true;
    result.message = match direction {
        StairDirection::Down => "You descend the staircase.".to_string(),
        StairDirection::Up => "You ascend the staircase.".to_string(),
    };
    log.add(&result.message, turn);
    result
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct SearchResult {
    ///
    pub found_secrets: Vec<(usize, usize)>,
    ///
    pub found_traps: Vec<(usize, usize)>,
    /// 硫붿떆吏
    pub message: String,
}

///
///
pub fn do_search(
    px: usize,
    py: usize,
    search_skill: i32,
    hidden_doors: &[(usize, usize)],
    hidden_traps: &[(usize, usize)],
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> SearchResult {
    let mut result = SearchResult {
        found_secrets: Vec::new(),
        found_traps: Vec::new(),
        message: String::new(),
    };

    let deltas: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for (dx, dy) in &deltas {
        let nx = px as i32 + dx;
        let ny = py as i32 + dy;
        if nx < 0 || ny < 0 {
            continue;
        }
        let (ux, uy) = (nx as usize, ny as usize);

        //
        for &(hx, hy) in hidden_doors {
            if hx == ux && hy == uy {
                let threshold = 1 + rng.rn2(7);
                if search_skill >= threshold as i32 {
                    result.found_secrets.push((ux, uy));
                }
            }
        }

        //
        for &(tx, ty) in hidden_traps {
            if tx == ux && ty == uy {
                let threshold = 1 + rng.rn2(7);
                if search_skill >= threshold as i32 {
                    result.found_traps.push((ux, uy));
                }
            }
        }
    }

    //
    if !result.found_secrets.is_empty() || !result.found_traps.is_empty() {
        let mut msgs = Vec::new();
        if !result.found_secrets.is_empty() {
            msgs.push(format!(
                "You find {} secret passage(s)!",
                result.found_secrets.len()
            ));
        }
        if !result.found_traps.is_empty() {
            msgs.push(format!("You find {} trap(s)!", result.found_traps.len()));
        }
        result.message = msgs.join(" ");
        log.add_colored(&result.message, [255, 255, 150], turn);
    } else {
        result.message = "You find nothing.".to_string();
    }

    result
}

// =============================================================================
//
// =============================================================================

///
pub fn do_sit(
    on_throne: bool,
    on_toilet: bool,
    on_altar: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> &'static str {
    if on_throne {
        //
        let effect = rng.rn2(10);
        let msg = match effect {
            0 => "You feel as if you have been knighted!",
            1 => "You feel very comfortable here.",
            2 => "A mysterious force surrounds you.",
            3 => "You feel a surge of power!",
            4 => "The throne crumbles beneath you.",
            _ => "You sit on the opulent throne.",
        };
        log.add_colored(msg, [255, 215, 0], turn);
        msg
    } else if on_toilet {
        let msg = "You sit on the toilet. How refreshing.";
        log.add(msg, turn);
        msg
    } else if on_altar {
        let msg = "You sit on the altar. Sacrilege!";
        log.add_colored(msg, [255, 100, 100], turn);
        msg
    } else {
        let msg = "You sit on the floor.";
        log.add(msg, turn);
        msg
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct KickResult {
    pub success: bool,
    pub message: String,
    pub damage_to_target: i32,
    pub damage_to_self: i32,
    pub broke_door: bool,
    pub broke_chest: bool,
}

///
pub fn kick_door(
    door_locked: bool,
    door_material_hardness: i32,
    player_strength: i32,
    wearing_boots: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> KickResult {
    let mut result = KickResult {
        success: false,
        message: String::new(),
        damage_to_target: 0,
        damage_to_self: 0,
        broke_door: false,
        broke_chest: false,
    };

    if !door_locked {
        result.message = "The door swings open.".to_string();
        result.success = true;
        log.add(&result.message, turn);
        return result;
    }

    //
    let kick_power = player_strength + if wearing_boots { 3 } else { 0 };
    let needed = door_material_hardness + rng.rn2(10) as i32;

    if kick_power >= needed {
        result.broke_door = true;
        result.success = true;
        result.message = "As you kick the door, it crashes open!".to_string();
        log.add_colored(&result.message, [255, 200, 100], turn);
    } else {
        result.damage_to_self = rng.rn2(2) as i32 + 1;
        result.message = "WHAMMM!!! You hurt your leg.".to_string();
        log.add_colored(&result.message, [255, 100, 100], turn);
    }

    result
}

///
pub fn kick_monster(
    player_strength: i32,
    player_level: i32,
    wearing_boots: bool,
    rng: &mut NetHackRng,
) -> i32 {
    //
    let base_dmg = 1 + rng.rn2(4) as i32;
    let str_bonus = (player_strength - 10).max(0) / 3;
    let level_bonus = player_level / 5;
    let boot_bonus = if wearing_boots { 2 } else { 0 };
    base_dmg + str_bonus + level_bonus + boot_bonus
}

// =============================================================================
//
// =============================================================================

///
///
pub fn turn_undead_range(player_level: i32, blessed: bool) -> i32 {
    let base = player_level / 2 + 3;
    if blessed {
        base + 3
    } else {
        base
    }
}

///
pub fn turn_undead_damage(player_level: i32, rng: &mut NetHackRng) -> i32 {
    let base = rng.rn2(player_level.max(1) as i32) as i32 + 1;
    base + player_level / 3
}

// =============================================================================
//
// =============================================================================

///
pub fn altar_identify_buc(
    item_name: &str,
    blessed: bool,
    cursed: bool,
    log: &mut GameLog,
    turn: u64,
) -> &'static str {
    let msg = if blessed {
        "There is an amber glow on the altar."
    } else if cursed {
        "There is a black flash on the altar."
    } else {
        "There is no reaction."
    };
    log.add_colored(
        msg,
        if blessed {
            [255, 200, 50]
        } else if cursed {
            [50, 50, 50]
        } else {
            [200, 200, 200]
        },
        turn,
    );
    msg
}

///
pub fn drop_in_fountain(
    item_name: &str,
    is_ring: bool,
    log: &mut GameLog,
    turn: u64,
) -> &'static str {
    if is_ring {
        let msg = "You drop the ring into the fountain. It sinks...";
        log.add_colored(msg, [100, 100, 255], turn);
        msg
    } else {
        let msg = "The object splashes into the fountain.";
        log.add(msg, turn);
        msg
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_basic() {
        let mut log = GameLog::new(100);
        let r = do_drop("a sword", 1, false, false, false, false, &mut log, 1);
        assert!(r.success);
        assert!(!r.fell_in_liquid);
    }

    #[test]
    fn test_drop_in_water() {
        let mut log = GameLog::new(100);
        let r = do_drop("a scroll", 1, false, true, false, false, &mut log, 1);
        assert!(r.fell_in_liquid);
    }

    #[test]
    fn test_stair_normal() {
        let mut log = GameLog::new(100);
        let r = do_stair(
            StairDirection::Down,
            true,
            false,
            false,
            false,
            false,
            &mut log,
            1,
        );
        assert!(r.success);
    }

    #[test]
    fn test_stair_no_stair() {
        let mut log = GameLog::new(100);
        let r = do_stair(
            StairDirection::Down,
            false,
            false,
            false,
            false,
            false,
            &mut log,
            1,
        );
        assert!(!r.success);
    }
}
