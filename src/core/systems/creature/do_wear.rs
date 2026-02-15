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
use crate::core::systems::worn::{WornSlot, WornSlots};

// =============================================================================
//
// =============================================================================

///
pub fn wear_delay(item_name: &str) -> i32 {
    match item_name {
        //
        "leather armor" | "studded leather armor" | "ring mail" => 3,
        "scale mail" | "chain mail" | "splint mail" => 5,
        "banded mail" | "plate mail" | "crystal plate mail" => 5,
        "dragon scale mail"
        | "silver dragon scale mail"
        | "red dragon scale mail"
        | "blue dragon scale mail"
        | "green dragon scale mail"
        | "yellow dragon scale mail"
        | "black dragon scale mail"
        | "white dragon scale mail"
        | "orange dragon scale mail"
        | "gray dragon scale mail" => 5,

        //
        "silver dragon scales"
        | "red dragon scales"
        | "blue dragon scales"
        | "green dragon scales"
        | "yellow dragon scales"
        | "black dragon scales"
        | "white dragon scales"
        | "orange dragon scales"
        | "gray dragon scales" => 5,

        // 留앺넗
        "cloak of magic resistance"
        | "cloak of invisibility"
        | "cloak of protection"
        | "cloak of displacement"
        | "oilskin cloak"
        | "elven cloak"
        | "leather cloak"
        | "alchemy smock"
        | "mummy wrapping" => 1,

        //
        "elven leather helm"
        | "orcish helm"
        | "dwarvish iron helm"
        | "helm of brilliance"
        | "helm of opposite alignment"
        | "helm of telepathy"
        | "cornuthaum"
        | "dunce cap"
        | "dented pot" => 1,

        //
        "leather gloves"
        | "gauntlets of fumbling"
        | "gauntlets of power"
        | "gauntlets of dexterity" => 1,

        //
        "low boots"
        | "high boots"
        | "iron shoes"
        | "elven boots"
        | "kicking boots"
        | "fumble boots"
        | "levitation boots"
        | "jumping boots"
        | "speed boots"
        | "water walking boots" => 1,

        //
        "small shield"
        | "large shield"
        | "orcish shield"
        | "elven shield"
        | "dwarvish roundshield"
        | "Uruk-hai shield"
        | "shield of reflection" => 1,

        //
        "Hawaiian shirt" | "T-shirt" => 5,

        //
        _ => 1,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn is_suit(name: &str) -> bool {
    matches!(
        name,
        "leather armor"
            | "studded leather armor"
            | "ring mail"
            | "scale mail"
            | "chain mail"
            | "splint mail"
            | "banded mail"
            | "plate mail"
            | "crystal plate mail"
            | "silver dragon scale mail"
            | "red dragon scale mail"
            | "blue dragon scale mail"
            | "green dragon scale mail"
            | "yellow dragon scale mail"
            | "black dragon scale mail"
            | "white dragon scale mail"
            | "orange dragon scale mail"
            | "gray dragon scale mail"
            | "silver dragon scales"
            | "red dragon scales"
            | "blue dragon scales"
            | "green dragon scales"
            | "yellow dragon scales"
            | "black dragon scales"
            | "white dragon scales"
            | "orange dragon scales"
            | "gray dragon scales"
    )
}

///
pub fn is_cloak(name: &str) -> bool {
    matches!(
        name,
        "cloak of magic resistance"
            | "cloak of invisibility"
            | "cloak of protection"
            | "cloak of displacement"
            | "oilskin cloak"
            | "elven cloak"
            | "leather cloak"
            | "alchemy smock"
            | "mummy wrapping"
    )
}

///
pub fn is_helmet(name: &str) -> bool {
    matches!(
        name,
        "elven leather helm"
            | "orcish helm"
            | "dwarvish iron helm"
            | "helm of brilliance"
            | "helm of opposite alignment"
            | "helm of telepathy"
            | "cornuthaum"
            | "dunce cap"
            | "dented pot"
    )
}

///
pub fn is_gloves(name: &str) -> bool {
    matches!(
        name,
        "leather gloves"
            | "gauntlets of fumbling"
            | "gauntlets of power"
            | "gauntlets of dexterity"
    )
}

///
pub fn is_boots(name: &str) -> bool {
    matches!(
        name,
        "low boots"
            | "high boots"
            | "iron shoes"
            | "elven boots"
            | "kicking boots"
            | "fumble boots"
            | "levitation boots"
            | "jumping boots"
            | "speed boots"
            | "water walking boots"
    )
}

///
pub fn is_shield(name: &str) -> bool {
    matches!(
        name,
        "small shield"
            | "large shield"
            | "orcish shield"
            | "elven shield"
            | "dwarvish roundshield"
            | "Uruk-hai shield"
            | "shield of reflection"
    )
}

///
pub fn is_shirt(name: &str) -> bool {
    matches!(name, "Hawaiian shirt" | "T-shirt")
}

///
pub fn item_to_slot(name: &str, class: ItemClass) -> u32 {
    match class {
        ItemClass::Armor => {
            if is_suit(name) {
                WornSlots::ARM
            } else if is_cloak(name) {
                WornSlots::ARMC
            } else if is_helmet(name) {
                WornSlots::ARMH
            } else if is_gloves(name) {
                WornSlots::ARMG
            } else if is_boots(name) {
                WornSlots::ARMF
            } else if is_shield(name) {
                WornSlots::ARMS
            } else if is_shirt(name) {
                WornSlots::ARMU
            } else {
                0
            }
        }
        ItemClass::Ring => WornSlots::RINGL | WornSlots::RINGR,
        ItemClass::Amulet => WornSlots::AMUL,
        ItemClass::Weapon => 0,
        ItemClass::Tool => WornSlots::TOOL,
        _ => 0,
    }
}

///
///
pub fn item_to_worn_slot(name: &str, class: ItemClass) -> Option<WornSlot> {
    match class {
        ItemClass::Armor => {
            if is_suit(name) {
                Some(WornSlot::Armor)
            } else if is_cloak(name) {
                Some(WornSlot::Cloak)
            } else if is_helmet(name) {
                Some(WornSlot::Helmet)
            } else if is_gloves(name) {
                Some(WornSlot::Gloves)
            } else if is_boots(name) {
                Some(WornSlot::Boots)
            } else if is_shield(name) {
                Some(WornSlot::Shield)
            } else if is_shirt(name) {
                Some(WornSlot::Shirt)
            } else {
                None
            }
        }
        ItemClass::Ring => Some(WornSlot::RingLeft),
        ItemClass::Amulet => Some(WornSlot::Amulet),
        ItemClass::Tool => Some(WornSlot::Tool),
        _ => None,
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub enum WearResult {
    Ok,
    AlreadyWearing,
    MustRemoveFirst(String),
    CannotWear(String),
    Cursed,
}

///
pub fn can_wear(slot: u32, currently_worn: &WornSlots, current_slot_cursed: bool) -> WearResult {
    //
    if currently_worn.contains(slot) {
        if current_slot_cursed {
            return WearResult::Cursed;
        }
        return WearResult::AlreadyWearing;
    }

    //
    if slot == WornSlots::ARM && currently_worn.contains(WornSlots::ARMC) {
        return WearResult::MustRemoveFirst("cloak".to_string());
    }

    //
    if slot == WornSlots::ARMU {
        if currently_worn.contains(WornSlots::ARM) {
            return WearResult::MustRemoveFirst("armor".to_string());
        }
        if currently_worn.contains(WornSlots::ARMC) {
            return WearResult::MustRemoveFirst("cloak".to_string());
        }
    }

    WearResult::Ok
}

///
pub fn can_remove(slot: u32, currently_worn: &WornSlots, item_cursed: bool) -> WearResult {
    //
    if item_cursed {
        return WearResult::Cursed;
    }

    //
    if !currently_worn.contains(slot) {
        return WearResult::CannotWear("You are not wearing that.".to_string());
    }

    //
    if slot == WornSlots::ARM && currently_worn.contains(WornSlots::ARMC) {
        return WearResult::MustRemoveFirst("cloak".to_string());
    }

    //
    if slot == WornSlots::ARMU && currently_worn.contains(WornSlots::ARM) {
        return WearResult::MustRemoveFirst("armor".to_string());
    }

    WearResult::Ok
}

///
pub fn can_wear_slot(
    slot: WornSlot,
    currently_worn: &WornSlots,
    current_slot_cursed: bool,
) -> WearResult {
    //
    if currently_worn.has(slot) {
        if current_slot_cursed {
            return WearResult::Cursed;
        }
        return WearResult::AlreadyWearing;
    }

    //
    if slot == WornSlot::Armor && currently_worn.has(WornSlot::Cloak) {
        return WearResult::MustRemoveFirst("cloak".to_string());
    }

    //
    if slot == WornSlot::Shirt {
        if currently_worn.has(WornSlot::Armor) {
            return WearResult::MustRemoveFirst("armor".to_string());
        }
        if currently_worn.has(WornSlot::Cloak) {
            return WearResult::MustRemoveFirst("cloak".to_string());
        }
    }

    WearResult::Ok
}

///
pub fn can_remove_slot(
    slot: WornSlot,
    currently_worn: &WornSlots,
    item_cursed: bool,
) -> WearResult {
    //
    if item_cursed {
        return WearResult::Cursed;
    }

    //
    if !currently_worn.has(slot) {
        return WearResult::CannotWear("You are not wearing that.".to_string());
    }

    //
    if slot == WornSlot::Armor && currently_worn.has(WornSlot::Cloak) {
        return WearResult::MustRemoveFirst("cloak".to_string());
    }

    //
    if slot == WornSlot::Shirt && currently_worn.has(WornSlot::Armor) {
        return WearResult::MustRemoveFirst("armor".to_string());
    }

    WearResult::Ok
}

// =============================================================================
//
// =============================================================================

///
pub fn autocurses_on_wear(name: &str) -> bool {
    matches!(
        name,
        "helm of opposite alignment" | "dunce cap" | "loadstone"
    )
}

// =============================================================================
//
// =============================================================================

///
pub fn arm_bonus(base_ac: i32, spe: i32, eroded: i32) -> i32 {
    //
    //
    let erosion_penalty = eroded;
    let bonus = base_ac + spe - erosion_penalty;
    bonus.max(0)
}

// =============================================================================
//
// =============================================================================

///
pub fn wear_message(item_name: &str, slot: u32) -> String {
    match slot {
        s if s == WornSlots::ARM => format!("You put on your {}.", item_name),
        s if s == WornSlots::ARMC => format!("You put on the {}.", item_name),
        s if s == WornSlots::ARMH => format!("You put on the {}.", item_name),
        s if s == WornSlots::ARMS => format!("You put {} on your left arm.", item_name),
        s if s == WornSlots::ARMG => format!("You put on a pair of {}.", item_name),
        s if s == WornSlots::ARMF => format!("You put on a pair of {}.", item_name),
        s if s == WornSlots::ARMU => format!("You put on the {}.", item_name),
        s if s == WornSlots::AMUL => format!("You put the {} around your neck.", item_name),
        s if s & (WornSlots::RINGL | WornSlots::RINGR) != 0 => {
            format!("You put the {} on your finger.", item_name)
        }
        _ => format!("You put on the {}.", item_name),
    }
}

///
pub fn remove_message(item_name: &str, slot: u32) -> String {
    match slot {
        s if s == WornSlots::ARM => format!("You take off your {}.", item_name),
        s if s == WornSlots::ARMC => format!("You take off the {}.", item_name),
        s if s == WornSlots::ARMH => format!("You take off the {}.", item_name),
        s if s == WornSlots::ARMS => format!("You put away the {}.", item_name),
        s if s == WornSlots::ARMG => format!("You take off your {}.", item_name),
        s if s == WornSlots::ARMF => format!("You take off your {}.", item_name),
        s if s == WornSlots::ARMU => format!("You take off the {}.", item_name),
        s if s == WornSlots::AMUL => format!("You remove the {} from your neck.", item_name),
        s if s & (WornSlots::RINGL | WornSlots::RINGR) != 0 => {
            format!("You remove the {} from your finger.", item_name)
        }
        _ => format!("You take off the {}.", item_name),
    }
}

///
pub fn wear_slot_message(item_name: &str, slot: WornSlot) -> String {
    match slot {
        WornSlot::Armor => format!("You put on your {}.", item_name),
        WornSlot::Cloak => format!("You put on the {}.", item_name),
        WornSlot::Helmet => format!("You put on the {}.", item_name),
        WornSlot::Shield => format!("You put {} on your left arm.", item_name),
        WornSlot::Gloves => format!("You put on a pair of {}.", item_name),
        WornSlot::Boots => format!("You put on a pair of {}.", item_name),
        WornSlot::Shirt => format!("You put on the {}.", item_name),
        WornSlot::Amulet => format!("You put the {} around your neck.", item_name),
        WornSlot::RingLeft | WornSlot::RingRight => {
            format!("You put the {} on your finger.", item_name)
        }
        _ => format!("You put on the {}.", item_name),
    }
}

///
pub fn remove_slot_message(item_name: &str, slot: WornSlot) -> String {
    match slot {
        WornSlot::Armor => format!("You take off your {}.", item_name),
        WornSlot::Cloak => format!("You take off the {}.", item_name),
        WornSlot::Helmet => format!("You take off the {}.", item_name),
        WornSlot::Shield => format!("You put away the {}.", item_name),
        WornSlot::Gloves => format!("You take off your {}.", item_name),
        WornSlot::Boots => format!("You take off your {}.", item_name),
        WornSlot::Shirt => format!("You take off the {}.", item_name),
        WornSlot::Amulet => format!("You remove the {} from your neck.", item_name),
        WornSlot::RingLeft | WornSlot::RingRight => {
            format!("You remove the {} from your finger.", item_name)
        }
        _ => format!("You take off the {}.", item_name),
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn is_two_handed(name: &str) -> bool {
    matches!(
        name,
        "two-handed sword"
            | "battle-axe"
            | "quarterstaff"
            | "lance"
            | "halberd"
            | "bardiche"
            | "voulge"
            | "dwarvish mattock"
            | "bill-guisarme"
            | "lucern hammer"
            | "bec de corbin"
            | "fauchard"
            | "guisarme"
            | "ranseur"
            | "spetum"
            | "partisan"
            | "glaive"
            | "longbow"
            | "elven bow"
            | "orcish bow"
            | "crossbow"
    )
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_suit() {
        assert!(is_suit("chain mail"));
        assert!(is_suit("silver dragon scale mail"));
        assert!(!is_suit("cloak of magic resistance"));
    }

    #[test]
    fn test_is_cloak() {
        assert!(is_cloak("elven cloak"));
        assert!(!is_cloak("chain mail"));
    }

    #[test]
    fn test_item_to_slot() {
        assert_eq!(item_to_slot("chain mail", ItemClass::Armor), WornSlots::ARM);
        assert_eq!(
            item_to_slot("elven cloak", ItemClass::Armor),
            WornSlots::ARMC
        );
        assert_eq!(
            item_to_slot("elven boots", ItemClass::Armor),
            WornSlots::ARMF
        );
    }

    #[test]
    fn test_can_wear() {
        let worn = WornSlots(0);
        assert!(matches!(
            can_wear(WornSlots::ARM, &worn, false),
            WearResult::Ok
        ));

        let mut worn = WornSlots(0);
        worn.set(WornSlots::ARMC);
        assert!(matches!(
            can_wear(WornSlots::ARM, &worn, false),
            WearResult::MustRemoveFirst(_)
        ));
    }

    #[test]
    fn test_wear_delay() {
        assert_eq!(wear_delay("chain mail"), 5);
        assert_eq!(wear_delay("elven cloak"), 1);
        assert_eq!(wear_delay("leather gloves"), 1);
    }

    #[test]
    fn test_arm_bonus() {
        assert_eq!(arm_bonus(5, 2, 0), 7); // AC5 + spe2
        assert_eq!(arm_bonus(5, 2, 1), 6);
    }

    //
    #[test]
    fn test_item_to_worn_slot() {
        assert_eq!(
            item_to_worn_slot("chain mail", ItemClass::Armor),
            Some(WornSlot::Armor)
        );
        assert_eq!(
            item_to_worn_slot("elven cloak", ItemClass::Armor),
            Some(WornSlot::Cloak)
        );
        assert_eq!(
            item_to_worn_slot("elven boots", ItemClass::Armor),
            Some(WornSlot::Boots)
        );
        assert_eq!(
            item_to_worn_slot("ring of fire resistance", ItemClass::Ring),
            Some(WornSlot::RingLeft)
        );
    }

    #[test]
    fn test_can_wear_slot_enum() {
        let worn = WornSlots::empty();
        assert!(matches!(
            can_wear_slot(WornSlot::Armor, &worn, false),
            WearResult::Ok
        ));

        let mut worn = WornSlots::empty();
        worn.wear(WornSlot::Cloak);
        assert!(matches!(
            can_wear_slot(WornSlot::Armor, &worn, false),
            WearResult::MustRemoveFirst(_)
        ));
    }

    #[test]
    fn test_wear_slot_message_enum() {
        let msg = wear_slot_message("chain mail", WornSlot::Armor);
        assert!(msg.contains("put on your"));

        let msg = remove_slot_message("amulet of life saving", WornSlot::Amulet);
        assert!(msg.contains("from your neck"));
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErosionType {
    None,
    Rust,
    Corrode,
    Burn,
    Rot,
}

///
pub fn erosion_message(erosion_level: i32, erosion_type: ErosionType) -> &'static str {
    match (erosion_level, erosion_type) {
        (_, ErosionType::None) => "",
        (1, ErosionType::Rust) => "rusty",
        (2, ErosionType::Rust) => "very rusty",
        (3, ErosionType::Rust) => "thoroughly rusty",
        (1, ErosionType::Corrode) => "corroded",
        (2, ErosionType::Corrode) => "very corroded",
        (3, ErosionType::Corrode) => "thoroughly corroded",
        (1, ErosionType::Burn) => "burnt",
        (2, ErosionType::Burn) => "very burnt",
        (3, ErosionType::Burn) => "thoroughly burnt",
        (1, ErosionType::Rot) => "rotted",
        (2, ErosionType::Rot) => "very rotted",
        (3, ErosionType::Rot) => "thoroughly rotted",
        _ => "damaged",
    }
}

///
pub fn uncurse_cost(item_base_value: u32) -> u32 {
    (item_base_value / 2).max(50)
}

///
pub fn wear_fail_message(reason: &WearResult) -> &'static str {
    match reason {
        WearResult::AlreadyWearing => "You are already wearing that!",
        WearResult::Cursed => "It is cursed! You can't take it off!",
        WearResult::MustRemoveFirst(_) => "You need to remove something else first.",
        WearResult::CannotWear(_) => "You can't wear that.",
        WearResult::Ok => "",
    }
}

///
pub fn armor_encumbrance_level(total_armor_weight: i32) -> &'static str {
    if total_armor_weight <= 50 {
        "Unencumbered"
    } else if total_armor_weight <= 100 {
        "Burdened"
    } else if total_armor_weight <= 150 {
        "Stressed"
    } else if total_armor_weight <= 200 {
        "Strained"
    } else {
        "Overtaxed"
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct DoWearStatistics {
    pub items_worn: u32,
    pub items_removed: u32,
    pub curse_blocks: u32,
    pub erosion_events: u32,
    pub uncurse_attempts: u32,
}

impl DoWearStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_wear(&mut self) {
        self.items_worn += 1;
    }
    pub fn record_remove(&mut self) {
        self.items_removed += 1;
    }
    pub fn record_curse_block(&mut self) {
        self.curse_blocks += 1;
    }
}

#[cfg(test)]
mod do_wear_extended_tests {
    use super::*;

    #[test]
    fn test_erosion_msg() {
        assert_eq!(erosion_message(1, ErosionType::Rust), "rusty");
        assert_eq!(
            erosion_message(3, ErosionType::Corrode),
            "thoroughly corroded"
        );
    }

    #[test]
    fn test_uncurse_cost() {
        assert!(uncurse_cost(200) >= 50);
    }

    #[test]
    fn test_encumbrance() {
        assert_eq!(armor_encumbrance_level(30), "Unencumbered");
        assert_eq!(armor_encumbrance_level(250), "Overtaxed");
    }

    #[test]
    fn test_do_wear_stats() {
        let mut s = DoWearStatistics::new();
        s.record_wear();
        s.record_curse_block();
        assert_eq!(s.items_worn, 1);
        assert_eq!(s.curse_blocks, 1);
    }
}
