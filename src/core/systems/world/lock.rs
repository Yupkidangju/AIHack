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
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockTool {
    Key,
    LockPick,
    CreditCard,
    WandKnock,
    SpellKnock, // Knock 留덈쾿
    Force,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockTarget {
    Door,
    Chest,
    LargeBox,
}

///
#[derive(Debug, Clone)]
pub struct LockResult {
    pub success: bool,
    pub message: String,
    pub door_state: Option<DoorState>,
    pub tool_broke: bool,
    pub turns_used: i32,
    pub trapped: bool,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    Broken,
}

impl LockResult {
    pub fn fail(msg: &str) -> Self {
        Self {
            success: false,
            message: msg.to_string(),
            door_state: None,
            tool_broke: false,
            turns_used: 1,
            trapped: false,
        }
    }
    pub fn ok(msg: &str, state: DoorState, turns: i32) -> Self {
        Self {
            success: true,
            message: msg.to_string(),
            door_state: Some(state),
            tool_broke: false,
            turns_used: turns,
            trapped: false,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn pick_lock_door(
    tool: LockTool,
    player_dex: i32,
    door_trapped: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> LockResult {
    //
    if tool == LockTool::WandKnock || tool == LockTool::SpellKnock {
        if door_trapped {
            log.add("You hear a click.", turn);
            return LockResult {
                success: true,
                message: "Trap disarmed.".into(),
                door_state: Some(DoorState::Closed),
                tool_broke: false,
                turns_used: 1,
                trapped: false,
            };
        }
        log.add("The door unlocks!", turn);
        return LockResult::ok("Door unlocked.", DoorState::Closed, 1);
    }

    //
    if door_trapped && rng.rn2(15) == 0 {
        log.add_colored("CLICK! You trigger a trap!", [255, 0, 0], turn);
        return LockResult {
            success: false,
            message: "Trap triggered!".into(),
            door_state: Some(DoorState::Locked),
            tool_broke: false,
            turns_used: 1,
            trapped: true,
        };
    }

    //
    let base_chance = match tool {
        LockTool::Key => 80,
        LockTool::LockPick => 60,
        LockTool::CreditCard => 40,
        _ => 20,
    };
    let dex_bonus = (player_dex - 10) * 3;
    let chance = (base_chance + dex_bonus).clamp(5, 95);

    if rng.rn2(100) < chance {
        log.add("You succeed in unlocking the door.", turn);
        LockResult::ok("Door unlocked.", DoorState::Closed, 1)
    } else {
        //
        let broke = tool == LockTool::CreditCard && rng.rn2(5) == 0;
        if broke {
            log.add("Your credit card breaks in two!", turn);
        } else {
            log.add("You fail to unlock the door.", turn);
        }
        LockResult {
            success: false,
            message: "Failed.".into(),
            door_state: Some(DoorState::Locked),
            tool_broke: broke,
            turns_used: 1,
            trapped: false,
        }
    }
}

///
pub fn lock_door(
    tool: LockTool,
    player_dex: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> LockResult {
    if tool != LockTool::Key && tool != LockTool::LockPick {
        let msg = "You can't lock a door with that.";
        log.add(msg, turn);
        return LockResult::fail(msg);
    }

    let chance = 70 + (player_dex - 10) * 3;
    if rng.rn2(100) < chance.clamp(10, 95) {
        log.add("You lock the door.", turn);
        LockResult::ok("Door locked.", DoorState::Locked, 1)
    } else {
        log.add("You fail to lock the door.", turn);
        LockResult::fail("Failed to lock.")
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn pick_lock_container(
    tool: LockTool,
    target: LockTarget,
    container_trapped: bool,
    player_dex: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> LockResult {
    let target_name = match target {
        LockTarget::Chest => "chest",
        LockTarget::LargeBox => "large box",
        LockTarget::Door => "door",
    };

    //
    if tool == LockTool::WandKnock || tool == LockTool::SpellKnock {
        if container_trapped {
            log.add(&format!("You hear a click from the {}.", target_name), turn);
            return LockResult {
                success: true,
                message: "Trap disarmed.".into(),
                door_state: None,
                tool_broke: false,
                turns_used: 1,
                trapped: false,
            };
        }
        log.add(&format!("The {} unlocks!", target_name), turn);
        return LockResult {
            success: true,
            message: format!("{} unlocked.", target_name),
            door_state: None,
            tool_broke: false,
            turns_used: 1,
            trapped: false,
        };
    }

    //
    if container_trapped && rng.rn2(10) == 0 {
        log.add_colored(
            &format!("CLICK! A needle shoots out from the {}!", target_name),
            [255, 0, 0],
            turn,
        );
        return LockResult {
            success: false,
            message: "Poison needle!".into(),
            door_state: None,
            tool_broke: false,
            turns_used: 1,
            trapped: true,
        };
    }

    //
    let base = match tool {
        LockTool::Key => 85,
        LockTool::LockPick => 65,
        LockTool::CreditCard => 45,
        _ => 25,
    };
    let chance = (base + (player_dex - 10) * 3).clamp(5, 95);

    if rng.rn2(100) < chance {
        log.add(
            &format!("You succeed in unlocking the {}.", target_name),
            turn,
        );
        LockResult {
            success: true,
            message: format!("{} unlocked.", target_name),
            door_state: None,
            tool_broke: false,
            turns_used: 1,
            trapped: false,
        }
    } else {
        let broke = matches!(tool, LockTool::CreditCard) && rng.rn2(5) == 0;
        if broke {
            log.add("Your credit card breaks in two!", turn);
        } else {
            log.add(&format!("You fail to unlock the {}.", target_name), turn);
        }
        LockResult {
            success: false,
            message: "Failed.".into(),
            door_state: None,
            tool_broke: broke,
            turns_used: 1,
            trapped: false,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn kick_door(
    player_str: i32,
    player_dex: i32,
    door_trapped: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> LockResult {
    //
    if door_trapped && rng.rn2(5) == 0 {
        log.add_colored("The door was trapped!", [255, 100, 0], turn);
        return LockResult {
            success: false,
            message: "Trap triggered.".into(),
            door_state: Some(DoorState::Locked),
            tool_broke: false,
            turns_used: 1,
            trapped: true,
        };
    }

    //
    let chance = player_str * 3 + player_dex + rng.rn2(20);
    if chance > 50 {
        if rng.rn2(3) == 0 {
            //
            log.add("The door crashes open!", turn);
            LockResult::ok("Door broken open.", DoorState::Broken, 1)
        } else {
            log.add("The door swings open.", turn);
            LockResult::ok("Door opened.", DoorState::Open, 1)
        }
    } else {
        log.add("The door resists!", turn);
        LockResult::fail("Door resists.")
    }
}

///
pub fn force_container(
    player_str: i32,
    weapon_type: &str,
    container_trapped: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> LockResult {
    //
    if container_trapped && rng.rn2(8) == 0 {
        log.add_colored("KABOOM! The box explodes!", [255, 100, 0], turn);
        return LockResult {
            success: false,
            message: "Explosion!".into(),
            door_state: None,
            tool_broke: false,
            turns_used: 1,
            trapped: true,
        };
    }

    //
    let weapon_bonus = if weapon_type.contains("axe") || weapon_type.contains("mattock") {
        20
    } else if weapon_type.contains("sword") || weapon_type.contains("mace") {
        10
    } else {
        0
    };

    let chance = player_str * 2 + weapon_bonus + rng.rn2(20);
    if chance > 40 {
        log.add("You succeed in forcing the lock open.", turn);
        LockResult {
            success: true,
            message: "Forced open.".into(),
            door_state: None,
            tool_broke: false,
            turns_used: 2,
            trapped: false,
        }
    } else {
        log.add("You fail to force the lock.", turn);
        LockResult::fail("Failed to force.")
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn is_locked_door(tile: TileType, locked: bool) -> bool {
    matches!(tile, TileType::Door) && locked
}

///
pub fn is_closed_door(tile: TileType) -> bool {
    matches!(tile, TileType::Door)
}

///
pub fn open_door(tile: TileType, locked: bool, log: &mut GameLog, turn: u64) -> Option<TileType> {
    if tile == TileType::Door && !locked {
        log.add("The door opens.", turn);
        Some(TileType::OpenDoor)
    } else if tile == TileType::Door && locked {
        log.add("This door is locked.", turn);
        None
    } else {
        None
    }
}

///
pub fn close_door(tile: TileType, log: &mut GameLog, turn: u64) -> Option<TileType> {
    if tile == TileType::OpenDoor {
        log.add("The door closes.", turn);
        Some(TileType::Door)
    } else {
        log.add("There is no open door here.", turn);
        None
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wand_knock_always_unlocks() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = pick_lock_door(LockTool::WandKnock, 10, false, &mut rng, &mut log, 1);
        assert!(result.success);
    }

    #[test]
    fn test_lock_door_needs_key() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = lock_door(LockTool::CreditCard, 10, &mut rng, &mut log, 1);
        assert!(!result.success);
    }

    #[test]
    fn test_is_locked_door() {
        assert!(is_locked_door(TileType::Door, true));
        assert!(!is_locked_door(TileType::Door, false));
        assert!(!is_locked_door(TileType::Room, true));
    }

    #[test]
    fn test_open_close_door() {
        let mut log = GameLog::new(100);
        assert_eq!(
            open_door(TileType::Door, false, &mut log, 1),
            Some(TileType::OpenDoor)
        );
        assert_eq!(
            close_door(TileType::OpenDoor, &mut log, 1),
            Some(TileType::Door)
        );
    }
}
