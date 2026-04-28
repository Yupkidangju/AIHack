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

// =============================================================================
// [v2.9.1] do_wear.c 대량 이식  방어구 효과/제한/변신/침식/섀폴리
// 원본: nethack-3.6.7/src/do_wear.c (2,816줄)
// =============================================================================

/// [v2.9.1] 방어구 고유 효과 (원본: do_wear.c:on_do_wear Armor_on)
/// 장비 착용 시 발동하는 특수 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArmorEffect {
    /// 효과 없음
    None,
    /// 능력치 변경 (str, dex 등)
    StatChange { stat: &'static str, delta: i32 },
    /// 상태 효과 부여
    StatusGrant(&'static str),
    /// 시야 변경
    VisionChange(&'static str),
    /// 자동 저주
    AutoCurse,
    /// 성향 반전
    AlignmentReverse,
    /// AC 보너스 (착용 즉시)
    ImmediateACBonus(i32),
}

/// [v2.9.1] 착용 시 효과 결정 (원본: Armor_on)
pub fn armor_on_effect(name: &str) -> Vec<ArmorEffect> {
    match name {
        "gauntlets of power" => vec![ArmorEffect::StatChange { stat: "str", delta: 25 }],
        "gauntlets of dexterity" => vec![ArmorEffect::StatChange { stat: "dex", delta: 0 }],
        "gauntlets of fumbling" => vec![ArmorEffect::StatusGrant("fumbling"), ArmorEffect::AutoCurse],
        "helm of brilliance" => vec![
            ArmorEffect::StatChange { stat: "int", delta: 0 },
            ArmorEffect::StatChange { stat: "wis", delta: 0 },
        ],
        "helm of telepathy" => vec![ArmorEffect::StatusGrant("telepathy")],
        "helm of opposite alignment" => vec![ArmorEffect::AlignmentReverse, ArmorEffect::AutoCurse],
        "dunce cap" => vec![
            ArmorEffect::StatChange { stat: "int", delta: -1 },
            ArmorEffect::StatChange { stat: "wis", delta: -1 },
            ArmorEffect::AutoCurse,
        ],
        "cloak of magic resistance" => vec![ArmorEffect::StatusGrant("magic_resistance")],
        "cloak of invisibility" => vec![ArmorEffect::StatusGrant("invisible")],
        "cloak of protection" => vec![ArmorEffect::ImmediateACBonus(-3)],
        "cloak of displacement" => vec![ArmorEffect::StatusGrant("displacement")],
        "elven cloak" => vec![ArmorEffect::StatusGrant("stealth")],
        "speed boots" => vec![ArmorEffect::StatusGrant("speed")],
        "levitation boots" => vec![ArmorEffect::StatusGrant("levitation"), ArmorEffect::AutoCurse],
        "fumble boots" => vec![ArmorEffect::StatusGrant("fumbling"), ArmorEffect::AutoCurse],
        "jumping boots" => vec![ArmorEffect::StatusGrant("jumping")],
        "water walking boots" => vec![ArmorEffect::StatusGrant("water_walking")],
        "elven boots" => vec![ArmorEffect::StatusGrant("stealth")],
        "kicking boots" => vec![ArmorEffect::StatChange { stat: "kick_damage", delta: 5 }],
        "shield of reflection" => vec![ArmorEffect::StatusGrant("reflection")],
        _ => vec![ArmorEffect::None],
    }
}

/// [v2.9.1] 해제 시 효과 역전 (원본: Armor_off)
pub fn armor_off_effect(name: &str) -> Vec<ArmorEffect> {
    match name {
        "gauntlets of power" => vec![ArmorEffect::StatChange { stat: "str", delta: -25 }],
        "gauntlets of fumbling" => vec![ArmorEffect::StatusGrant("remove_fumbling")],
        "helm of brilliance" => vec![
            ArmorEffect::StatChange { stat: "int", delta: 0 },
            ArmorEffect::StatChange { stat: "wis", delta: 0 },
        ],
        "helm of telepathy" => vec![ArmorEffect::StatusGrant("remove_telepathy")],
        "dunce cap" => vec![
            ArmorEffect::StatChange { stat: "int", delta: 1 },
            ArmorEffect::StatChange { stat: "wis", delta: 1 },
        ],
        "cloak of magic resistance" => vec![ArmorEffect::StatusGrant("remove_magic_resistance")],
        "cloak of invisibility" => vec![ArmorEffect::StatusGrant("remove_invisible")],
        "cloak of displacement" => vec![ArmorEffect::StatusGrant("remove_displacement")],
        "elven cloak" | "elven boots" => vec![ArmorEffect::StatusGrant("remove_stealth")],
        "speed boots" => vec![ArmorEffect::StatusGrant("remove_speed")],
        "levitation boots" => vec![ArmorEffect::StatusGrant("remove_levitation")],
        "fumble boots" => vec![ArmorEffect::StatusGrant("remove_fumbling")],
        "jumping boots" => vec![ArmorEffect::StatusGrant("remove_jumping")],
        "water walking boots" => vec![ArmorEffect::StatusGrant("remove_water_walking")],
        "shield of reflection" => vec![ArmorEffect::StatusGrant("remove_reflection")],
        _ => vec![ArmorEffect::None],
    }
}

/// [v2.9.1] 변신 시 방어구 자동 해제 판정 (원본: do_wear.c:1200-1280 break_armor)
/// 변신 체형에 따라 어떤 장비가 떨어지는지 결정
pub fn break_armor_check(
    new_body_size: i32,   // 0=tiny, 1=small, 2=medium, 3=large, 4=huge
    has_hands: bool,
    has_head: bool,
    has_feet: bool,
    is_whirly: bool,      // 소용돌이형 (에어 엘리멘탈 등)
    is_noncorporeal: bool, // 비육체 (유령 등)
) -> Vec<&'static str> {
    let mut dropped = Vec::new();

    // 비육체  모든 장비 떨어짐
    if is_noncorporeal {
        return vec!["armor", "cloak", "helmet", "gloves", "boots", "shield", "shirt"];
    }

    // 소용돌이형  대부분 떨어짐
    if is_whirly {
        dropped.push("helmet");
        dropped.push("gloves");
        dropped.push("boots");
        dropped.push("shield");
    }

    // 크기 변화
    if new_body_size <= 1 {
        // 작은 체형: 갑옷, 클로크 떨어짐
        dropped.push("armor");
        dropped.push("cloak");
    }

    if new_body_size >= 4 {
        // 거대 체형: 갑옷 파괴
        dropped.push("armor");
        dropped.push("shirt");
    }

    if !has_hands {
        dropped.push("gloves");
        dropped.push("shield");
    }

    if !has_head {
        dropped.push("helmet");
    }

    if !has_feet {
        dropped.push("boots");
    }

    dropped
}

/// [v2.9.1] 드래곤 스케일  드래곤 스케일 메일 변환 (원본: do_wear.c:870-920 Dragon_scales_to_mail)
pub fn dragon_scales_to_mail(scales_name: &str) -> Option<&'static str> {
    match scales_name {
        "silver dragon scales" => Some("silver dragon scale mail"),
        "red dragon scales" => Some("red dragon scale mail"),
        "blue dragon scales" => Some("blue dragon scale mail"),
        "green dragon scales" => Some("green dragon scale mail"),
        "yellow dragon scales" => Some("yellow dragon scale mail"),
        "black dragon scales" => Some("black dragon scale mail"),
        "white dragon scales" => Some("white dragon scale mail"),
        "orange dragon scales" => Some("orange dragon scale mail"),
        "gray dragon scales" => Some("gray dragon scale mail"),
        _ => None,
    }
}

/// [v2.9.1] 드래곤 스케일 메일  드래곤 스케일 역변환
pub fn dragon_mail_to_scales(mail_name: &str) -> Option<&'static str> {
    match mail_name {
        "silver dragon scale mail" => Some("silver dragon scales"),
        "red dragon scale mail" => Some("red dragon scales"),
        "blue dragon scale mail" => Some("blue dragon scales"),
        "green dragon scale mail" => Some("green dragon scales"),
        "yellow dragon scale mail" => Some("yellow dragon scales"),
        "black dragon scale mail" => Some("black dragon scales"),
        "white dragon scale mail" => Some("white dragon scales"),
        "orange dragon scale mail" => Some("orange dragon scales"),
        "gray dragon scale mail" => Some("gray dragon scales"),
        _ => None,
    }
}

/// [v2.9.1] 드래곤 스케일 메일 저항 (원본: do_wear.c dragonscale_intrinsic)
pub fn dragon_scale_resistance(name: &str) -> Option<&'static str> {
    let prefix = if name.contains("dragon scale") {
        name.split(" dragon").next().unwrap_or("")
    } else {
        return None;
    };
    match prefix {
        "silver" => Some("reflection"),
        "red" => Some("fire_resistance"),
        "blue" => Some("shock_resistance"),
        "green" => Some("poison_resistance"),
        "yellow" => Some("acid_resistance"),
        "black" => Some("disintegration_resistance"),
        "white" => Some("cold_resistance"),
        "orange" => Some("sleep_resistance"),
        "gray" => Some("magic_resistance"),
        _ => None,
    }
}

/// [v2.9.1] 방어구 재질별 침식 면역 (원본: do_wear.c:690-740)
pub fn armor_resists_erosion(name: &str, erosion_type: ErosionType) -> bool {
    match erosion_type {
        ErosionType::Rust => {
            // 나무/돌/뼈/유리/천/가죽은 녹슬지 않음
            name.contains("leather") || name.contains("elven")
                || name.contains("studded") || name.contains("dragon")
                || name.contains("crystal") || name.contains("cloth")
        }
        ErosionType::Corrode => {
            // 석재/나무/가죽은 부식되지 않음
            name.contains("leather") || name.contains("crystal")
                || name.contains("dragon")
        }
        ErosionType::Burn => {
            // 금속은 불에 타지 않음
            name.contains("iron") || name.contains("steel")
                || name.contains("mithril") || name.contains("dwarvish")
                || name.contains("chain") || name.contains("plate")
                || name.contains("ring mail") || name.contains("scale mail")
                || name.contains("splint") || name.contains("banded")
        }
        ErosionType::Rot => {
            // 금속/돌/유리는 썩지 않음
            name.contains("iron") || name.contains("steel")
                || name.contains("crystal") || name.contains("chain")
                || name.contains("plate") || name.contains("mithril")
        }
        ErosionType::None => true,
    }
}

/// [v2.9.1] 방어구 수리 비용 (원본: do_wear.c:fix_armor_cost)
pub fn repair_armor_cost(base_value: u32, erosion_level: i32) -> u32 {
    let multiplier = match erosion_level {
        1 => 1,
        2 => 3,
        3 => 6,
        _ => 0,
    };
    (base_value / 4 * multiplier as u32).max(10)
}

/// [v2.9.1] 은 방어구 vs 특정 종족 효과 (원본: do_wear.c silver_damage)
pub fn silver_armor_effect(armor_name: &str, target_is_undead: bool, target_is_demon: bool) -> i32 {
    if !armor_name.contains("silver") {
        return 0;
    }
    if target_is_undead || target_is_demon {
        // 은 방어구를 맞으면 추가 데미지 (킥 등)
        20
    } else {
        0
    }
}

/// [v2.9.1] 착용 순서 검증 (원본: do_wear.c donning_order)
/// 올바른 착용 순서: 셔츠  갑옷  클로크
pub fn correct_donning_order(slot: WornSlot, currently_worn: &WornSlots) -> Result<(), &'static str> {
    match slot {
        WornSlot::Shirt => {
            if currently_worn.has(WornSlot::Armor) {
                return Err("You must remove your armor first.");
            }
            if currently_worn.has(WornSlot::Cloak) {
                return Err("You must remove your cloak first.");
            }
            Ok(())
        }
        WornSlot::Armor => {
            if currently_worn.has(WornSlot::Cloak) {
                return Err("You must remove your cloak first.");
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// [v2.9.1] 해제 순서 검증 (원본: do_wear.c doffing_order)
pub fn correct_doffing_order(slot: WornSlot, currently_worn: &WornSlots) -> Result<(), &'static str> {
    match slot {
        WornSlot::Cloak => Ok(()), // 클로크는 항상 해제 가능
        WornSlot::Armor => {
            if currently_worn.has(WornSlot::Cloak) {
                return Err("You must remove your cloak first.");
            }
            Ok(())
        }
        WornSlot::Shirt => {
            if currently_worn.has(WornSlot::Cloak) {
                return Err("You must remove your cloak first.");
            }
            if currently_worn.has(WornSlot::Armor) {
                return Err("You must remove your armor first.");
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// [v2.9.1] 반지 슬롯 선택 (원본: do_wear.c select_ring_slot)
pub fn select_ring_slot(
    left_occupied: bool,
    right_occupied: bool,
    left_cursed: bool,
    prefer_left: bool,
) -> Result<WornSlot, &'static str> {
    match (left_occupied, right_occupied) {
        (false, false) => {
            if prefer_left { Ok(WornSlot::RingLeft) } else { Ok(WornSlot::RingRight) }
        }
        (true, false) => Ok(WornSlot::RingRight),
        (false, true) => Ok(WornSlot::RingLeft),
        (true, true) => {
            if left_cursed {
                Err("Your ring on the left hand is cursed!")
            } else {
                Err("You are already wearing two rings.")
            }
        }
    }
}

/// [v2.9.1] AC 총합 계산 (원본: do_wear.c calc_total_ac)
pub fn calculate_total_ac(
    base_ac: i32,
    armor_ac: i32,
    shield_ac: i32,
    helmet_ac: i32,
    gloves_ac: i32,
    boots_ac: i32,
    cloak_ac: i32,
    ring_protection: i32,
    dex_bonus: i32,
) -> i32 {
    let total = base_ac - armor_ac - shield_ac - helmet_ac - gloves_ac - boots_ac - cloak_ac - ring_protection - dex_bonus;
    total
}

// =============================================================================
// [v2.9.1] do_wear.c 테스트
// =============================================================================
#[cfg(test)]
mod do_wear_v291_tests {
    use super::*;

    #[test]
    fn test_armor_on_speed_boots() {
        let effects = armor_on_effect("speed boots");
        assert!(effects.iter().any(|e| matches!(e, ArmorEffect::StatusGrant("speed"))));
    }

    #[test]
    fn test_armor_on_dunce_cap() {
        let effects = armor_on_effect("dunce cap");
        assert!(effects.iter().any(|e| matches!(e, ArmorEffect::AutoCurse)));
    }

    #[test]
    fn test_armor_off_reverse() {
        let effects = armor_off_effect("gauntlets of power");
        assert!(effects.iter().any(|e| matches!(e, ArmorEffect::StatChange { stat: "str", delta: -25 })));
    }

    #[test]
    fn test_break_armor_tiny() {
        let dropped = break_armor_check(0, true, true, true, false, false);
        assert!(dropped.contains(&"armor"));
        assert!(dropped.contains(&"cloak"));
    }

    #[test]
    fn test_break_armor_noncorporeal() {
        let dropped = break_armor_check(2, true, true, true, false, true);
        assert_eq!(dropped.len(), 7); // 모든 장비
    }

    #[test]
    fn test_break_armor_no_hands() {
        let dropped = break_armor_check(2, false, true, true, false, false);
        assert!(dropped.contains(&"gloves"));
        assert!(dropped.contains(&"shield"));
    }

    #[test]
    fn test_dragon_scales_to_mail() {
        assert_eq!(dragon_scales_to_mail("silver dragon scales"), Some("silver dragon scale mail"));
        assert_eq!(dragon_scales_to_mail("leather armor"), None);
    }

    #[test]
    fn test_dragon_mail_to_scales() {
        assert_eq!(dragon_mail_to_scales("red dragon scale mail"), Some("red dragon scales"));
    }

    #[test]
    fn test_dragon_resistance() {
        assert_eq!(dragon_scale_resistance("silver dragon scale mail"), Some("reflection"));
        assert_eq!(dragon_scale_resistance("red dragon scales"), Some("fire_resistance"));
        assert_eq!(dragon_scale_resistance("leather armor"), None);
    }

    #[test]
    fn test_erosion_immunity_rust() {
        assert!(armor_resists_erosion("leather armor", ErosionType::Rust));
        assert!(!armor_resists_erosion("iron shoes", ErosionType::Rust));
    }

    #[test]
    fn test_erosion_immunity_burn() {
        assert!(armor_resists_erosion("chain mail", ErosionType::Burn));
        assert!(!armor_resists_erosion("leather armor", ErosionType::Burn));
    }

    #[test]
    fn test_repair_cost() {
        assert!(repair_armor_cost(200, 1) < repair_armor_cost(200, 3));
    }

    #[test]
    fn test_silver_effect() {
        assert_eq!(silver_armor_effect("silver dragon scale mail", true, false), 20);
        assert_eq!(silver_armor_effect("chain mail", true, false), 0);
    }

    #[test]
    fn test_donning_order() {
        let mut worn = WornSlots::empty();
        assert!(correct_donning_order(WornSlot::Shirt, &worn).is_ok());
        worn.wear(WornSlot::Armor);
        assert!(correct_donning_order(WornSlot::Shirt, &worn).is_err());
    }

    #[test]
    fn test_doffing_order() {
        let mut worn = WornSlots::empty();
        worn.wear(WornSlot::Cloak);
        worn.wear(WornSlot::Armor);
        assert!(correct_doffing_order(WornSlot::Armor, &worn).is_err());
    }

    #[test]
    fn test_ring_slot_both_empty() {
        let slot = select_ring_slot(false, false, false, true);
        assert_eq!(slot, Ok(WornSlot::RingLeft));
    }

    #[test]
    fn test_ring_slot_left_full() {
        let slot = select_ring_slot(true, false, false, true);
        assert_eq!(slot, Ok(WornSlot::RingRight));
    }

    #[test]
    fn test_ring_slot_both_full() {
        let slot = select_ring_slot(true, true, false, true);
        assert!(slot.is_err());
    }

    #[test]
    fn test_calc_total_ac() {
        let ac = calculate_total_ac(10, 5, 2, 1, 1, 1, 1, 0, 0);
        assert_eq!(ac, -1); // 10 - 11 = -1
    }
}
