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

use crate::core::entity::object::ItemClass;
use crate::ui::log::GameLog;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct PickupConfig {
    pub autopickup: bool,
    pub pickup_types: Vec<ItemClass>,
    pub pickup_thrown: bool,
    pub pickup_burden: BurdenLevel,
}

impl Default for PickupConfig {
    fn default() -> Self {
        Self {
            autopickup: true,
            pickup_types: vec![ItemClass::Coin, ItemClass::Gem],
            pickup_thrown: true,
            pickup_burden: BurdenLevel::Stressed,
        }
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BurdenLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

// =============================================================================
//
// =============================================================================

///
pub fn weight_capacity(strength: i32) -> i32 {
    //
    //
    let base = match strength {
        0..=3 => 100,
        4..=6 => 200,
        7..=9 => 350,
        10..=12 => 500,
        13..=15 => 700,
        16..=17 => 900,
        18 => 1000,
        _ => 1000 + (strength - 18) * 100,
    };
    base
}

///
pub fn burden_level(current_weight: i32, strength: i32) -> BurdenLevel {
    let cap = weight_capacity(strength);
    let ratio = if cap > 0 {
        current_weight * 100 / cap
    } else {
        100
    };

    if ratio <= 50 {
        BurdenLevel::Unencumbered
    } else if ratio <= 67 {
        BurdenLevel::Burdened
    } else if ratio <= 83 {
        BurdenLevel::Stressed
    } else if ratio <= 100 {
        BurdenLevel::Strained
    } else if ratio <= 117 {
        BurdenLevel::Overtaxed
    } else {
        BurdenLevel::Overloaded
    }
}

///
pub fn burden_speed_penalty(level: BurdenLevel) -> i32 {
    match level {
        BurdenLevel::Unencumbered => 0,
        BurdenLevel::Burdened => -1,
        BurdenLevel::Stressed => -3,
        BurdenLevel::Strained => -5,
        BurdenLevel::Overtaxed => -7,
        BurdenLevel::Overloaded => -9,
    }
}

///
pub fn burden_name(level: BurdenLevel) -> &'static str {
    match level {
        BurdenLevel::Unencumbered => "Unencumbered",
        BurdenLevel::Burdened => "Burdened",
        BurdenLevel::Stressed => "Stressed",
        BurdenLevel::Strained => "Strained",
        BurdenLevel::Overtaxed => "Overtaxed",
        BurdenLevel::Overloaded => "Overloaded",
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct PickupResult {
    pub picked_up: bool,
    pub item_name: String,
    pub item_count: u32,
    pub weight_added: i32,
    pub message: String,
    pub new_burden: BurdenLevel,
}

impl PickupResult {
    pub fn fail(msg: &str) -> Self {
        Self {
            picked_up: false,
            item_name: String::new(),
            item_count: 0,
            weight_added: 0,
            message: msg.to_string(),
            new_burden: BurdenLevel::Unencumbered,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn try_pickup_item(
    item_name: &str,
    item_weight: i32,
    item_count: u32,
    item_class: ItemClass,
    current_weight: i32,
    strength: i32,
    is_levitating: bool,
    log: &mut GameLog,
    turn: u64,
) -> PickupResult {
    //
    if is_levitating {
        let msg = "You cannot reach the floor.";
        log.add(msg, turn);
        return PickupResult::fail(msg);
    }

    let total_weight = item_weight * item_count as i32;
    let new_weight = current_weight + total_weight;
    let new_burden = burden_level(new_weight, strength);

    //
    if new_burden >= BurdenLevel::Overloaded {
        let msg = format!("{} is too heavy to pick up!", item_name);
        log.add(&msg, turn);
        return PickupResult::fail(&msg);
    }

    //
    let msg = if item_count > 1 {
        format!("You pick up {} {}.", item_count, item_name)
    } else {
        format!("You pick up {}.", item_name)
    };
    log.add(&msg, turn);

    //
    let old_burden = burden_level(current_weight, strength);
    if new_burden > old_burden {
        log.add(&format!("You are now {}.", burden_name(new_burden)), turn);
    }

    PickupResult {
        picked_up: true,
        item_name: item_name.to_string(),
        item_count,
        weight_added: total_weight,
        message: msg,
        new_burden,
    }
}

///
pub fn pickup_gold(amount: i32, is_levitating: bool, log: &mut GameLog, turn: u64) -> PickupResult {
    if is_levitating {
        let msg = "You cannot reach the floor.";
        log.add(msg, turn);
        return PickupResult::fail(msg);
    }
    if amount <= 0 {
        return PickupResult::fail("No gold here.");
    }

    let msg = format!(
        "You pick up {} gold piece{}.",
        amount,
        if amount != 1 { "s" } else { "" }
    );
    log.add_colored(&msg, [255, 215, 0], turn);

    PickupResult {
        picked_up: true,
        item_name: "gold".to_string(),
        item_count: amount as u32,
        weight_added: 1,
        message: msg,
        new_burden: BurdenLevel::Unencumbered,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn should_autopickup(item_class: ItemClass, item_name: &str, config: &PickupConfig) -> bool {
    if !config.autopickup {
        return false;
    }

    //
    if config.pickup_types.contains(&item_class) {
        return true;
    }

    //
    if config.pickup_thrown {
        //
        if item_name.contains("arrow")
            || item_name.contains("bolt")
            || item_name.contains("shuriken")
            || item_name.contains("dart")
        {
            return true;
        }
    }

    false
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct DropResult {
    pub dropped: bool,
    pub item_name: String,
    pub item_count: u32,
    pub weight_removed: i32,
    pub message: String,
}

///
pub fn try_drop_item(
    item_name: &str,
    item_weight: i32,
    item_count: u32,
    is_cursed: bool,
    is_equipped: bool,
    tile_type: TileType,
    log: &mut GameLog,
    turn: u64,
) -> DropResult {
    //
    if is_cursed && is_equipped {
        let msg = format!("Your {} is welded to you!", item_name);
        log.add_colored(&msg, [200, 50, 50], turn);
        return DropResult {
            dropped: false,
            item_name: item_name.to_string(),
            item_count: 0,
            weight_removed: 0,
            message: msg,
        };
    }

    //
    let destroyed = matches!(tile_type, TileType::LavaPool | TileType::Pool);
    let msg = if destroyed {
        if tile_type == TileType::LavaPool {
            format!("Your {} is destroyed by the lava!", item_name)
        } else {
            format!("Your {} sinks into the water.", item_name)
        }
    } else if item_count > 1 {
        format!("You drop {} {}.", item_count, item_name)
    } else {
        format!("You drop {}.", item_name)
    };

    if destroyed {
        log.add_colored(&msg, [255, 100, 0], turn);
    } else {
        log.add(&msg, turn);
    }

    let total_weight = item_weight * item_count as i32;
    DropResult {
        dropped: true,
        item_name: item_name.to_string(),
        item_count,
        weight_removed: total_weight,
        message: msg,
    }
}

///
pub fn drop_all_prompt() -> &'static str {
    "Drop all items? [y/n]"
}

// =============================================================================
//
// =============================================================================

///
pub fn floor_items_message(item_count: usize, has_gold: bool, gold_amount: i32) -> String {
    if item_count == 0 && !has_gold {
        "There is nothing here.".to_string()
    } else if item_count == 0 && has_gold {
        format!(
            "There {} {} gold piece{} here.",
            if gold_amount == 1 { "is" } else { "are" },
            gold_amount,
            if gold_amount != 1 { "s" } else { "" }
        )
    } else if item_count == 1 && !has_gold {
        "There is an item here.".to_string()
    } else {
        let mut msg = format!("There are {} things here", item_count);
        if has_gold {
            msg.push_str(&format!(", plus {} gold", gold_amount));
        }
        msg.push('.');
        msg
    }
}

use crate::core::dungeon::tile::TileType;

// =============================================================================
//
// =============================================================================

///
pub fn can_pickup_corpse(
    corpse_age: u64,
    current_turn: u64,
    is_tinned: bool,
) -> (bool, &'static str) {
    if is_tinned {
        return (true, "The corpse is preserved in a tin.");
    }

    let age = current_turn.saturating_sub(corpse_age);
    if age > 50 {
        (false, "This corpse is too old and has rotted away!")
    } else if age > 30 {
        (true, "The corpse smells terrible.")
    } else {
        (true, "")
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn can_pickup_boulder(is_giant: bool, player_str: i32) -> bool {
    //
    is_giant || player_str >= 25
}

///
pub fn check_floor_for_trap(
    has_trap: bool,
    trap_discovered: bool,
    log: &mut GameLog,
    turn: u64,
) -> bool {
    if has_trap && !trap_discovered {
        log.add("Wait! There's a trap here.", turn);
        return true;
    }
    false
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_capacity() {
        assert!(weight_capacity(18) > weight_capacity(10));
        assert!(weight_capacity(10) > weight_capacity(3));
    }

    #[test]
    fn test_burden_level() {
        assert_eq!(burden_level(100, 18), BurdenLevel::Unencumbered);
        assert_eq!(burden_level(5000, 10), BurdenLevel::Overloaded);
    }

    #[test]
    fn test_floor_message() {
        let msg = floor_items_message(0, false, 0);
        assert_eq!(msg, "There is nothing here.");
        let msg = floor_items_message(0, true, 50);
        assert!(msg.contains("50"));
    }

    #[test]
    fn test_autopickup() {
        let config = PickupConfig::default();
        assert!(should_autopickup(ItemClass::Coin, "gold piece", &config));
        assert!(!should_autopickup(ItemClass::Weapon, "sword", &config));
    }

    #[test]
    fn test_corpse_check() {
        let (ok, _msg) = can_pickup_corpse(0, 10, false);
        assert!(ok);
        let (ok, _msg) = can_pickup_corpse(0, 100, false);
        assert!(!ok);
    }

    #[test]
    fn test_boulder_pickup() {
        assert!(can_pickup_boulder(true, 10));
        assert!(can_pickup_boulder(false, 25));
        assert!(!can_pickup_boulder(false, 15));
    }
}
