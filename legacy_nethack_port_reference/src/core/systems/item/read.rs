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
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollType {
    Enchant,       // SCR_ENCHANT_WEAPON / SCR_ENCHANT_ARMOR
    Identify,      // SCR_IDENTIFY
    RemoveCurse,   // SCR_REMOVE_CURSE
    CreateMonster, // SCR_CREATE_MONSTER
    Teleportation, // SCR_TELEPORTATION
    Gold,          // SCR_GOLD_DETECTION
    Food,          // SCR_FOOD_DETECTION
    Confuse,       // SCR_CONFUSE_MONSTER
    Scare,         // SCR_SCARE_MONSTER
    AmnesiaCurse,
    Fire,          // SCR_FIRE
    Earth,
    Punishment,    // SCR_PUNISHMENT
    Stinking,      // SCR_STINKING_CLOUD
    Blank,         // SCR_BLANK_PAPER
    Genocide,      // SCR_GENOCIDE
    Light,         // SCR_LIGHT
    Taming,        // SCR_TAMING
    Charging,      // SCR_CHARGING
    MagicMapping,  // SCR_MAGIC_MAPPING
    Destroy,       // SCR_DESTROY_ARMOR
    Mail,          // SCR_MAIL
}

///
#[derive(Debug, Clone)]
pub struct ScrollResult {
    pub scroll_type: ScrollType,
    pub success: bool,
    pub message: String,
    pub consumed: bool,
    pub identified: bool,
    pub items_affected: i32,
    pub monsters_affected: i32,
}

impl ScrollResult {
    pub fn new(scroll_type: ScrollType, msg: &str) -> Self {
        Self {
            scroll_type,
            success: true,
            message: msg.to_string(),
            consumed: true,
            identified: true,
            items_affected: 0,
            monsters_affected: 0,
        }
    }

    pub fn fail(scroll_type: ScrollType, msg: &str) -> Self {
        Self {
            scroll_type,
            success: false,
            message: msg.to_string(),
            consumed: true,
            identified: false,
            items_affected: 0,
            monsters_affected: 0,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn scroll_name_to_type(name: &str) -> Option<ScrollType> {
    let lower = name.to_lowercase();
    if lower.contains("enchant") {
        Some(ScrollType::Enchant)
    } else if lower.contains("identify") {
        Some(ScrollType::Identify)
    } else if lower.contains("remove curse") {
        Some(ScrollType::RemoveCurse)
    } else if lower.contains("create monster") {
        Some(ScrollType::CreateMonster)
    } else if lower.contains("teleportation") {
        Some(ScrollType::Teleportation)
    } else if lower.contains("gold detection") {
        Some(ScrollType::Gold)
    } else if lower.contains("food detection") {
        Some(ScrollType::Food)
    } else if lower.contains("confuse monster") {
        Some(ScrollType::Confuse)
    } else if lower.contains("scare monster") {
        Some(ScrollType::Scare)
    } else if lower.contains("amnesia") {
        Some(ScrollType::AmnesiaCurse)
    } else if lower.contains("fire") {
        Some(ScrollType::Fire)
    } else if lower.contains("earth") {
        Some(ScrollType::Earth)
    } else if lower.contains("punishment") {
        Some(ScrollType::Punishment)
    } else if lower.contains("stinking cloud") {
        Some(ScrollType::Stinking)
    } else if lower.contains("blank") {
        Some(ScrollType::Blank)
    } else if lower.contains("genocide") {
        Some(ScrollType::Genocide)
    } else if lower.contains("light") {
        Some(ScrollType::Light)
    } else if lower.contains("taming") {
        Some(ScrollType::Taming)
    } else if lower.contains("charging") {
        Some(ScrollType::Charging)
    } else if lower.contains("magic mapping") || lower.contains("map") {
        Some(ScrollType::MagicMapping)
    } else if lower.contains("destroy armor") {
        Some(ScrollType::Destroy)
    } else if lower.contains("mail") {
        Some(ScrollType::Mail)
    } else {
        None
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn read_scroll(
    scroll_name: &str,
    blessed: bool,
    cursed: bool,
    confused: bool,
    blind: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    //
    if blind {
        let msg = "Being blind, you cannot read the scroll.";
        log.add(msg, turn);
        return ScrollResult {
            scroll_type: ScrollType::Blank,
            success: false,
            message: msg.to_string(),
            consumed: false,
            identified: false,
            items_affected: 0,
            monsters_affected: 0,
        };
    }

    let scroll_type = match scroll_name_to_type(scroll_name) {
        Some(t) => t,
        None => {
            let msg = "You read the scroll, but nothing happens.";
            log.add(msg, turn);
            return ScrollResult::fail(ScrollType::Blank, msg);
        }
    };

    //
    match scroll_type {
        ScrollType::Identify => scroll_identify(blessed, cursed, rng, log, turn),
        ScrollType::RemoveCurse => scroll_remove_curse(blessed, cursed, rng, log, turn),
        ScrollType::Enchant => scroll_enchant(blessed, cursed, confused, rng, log, turn),
        ScrollType::Teleportation => scroll_teleport(blessed, cursed, confused, rng, log, turn),
        ScrollType::CreateMonster => {
            scroll_create_monster(blessed, cursed, confused, rng, log, turn)
        }
        ScrollType::Gold => scroll_gold_detect(blessed, cursed, confused, rng, log, turn),
        ScrollType::Food => scroll_food_detect(blessed, cursed, rng, log, turn),
        ScrollType::Confuse => scroll_confuse(blessed, cursed, rng, log, turn),
        ScrollType::Scare => scroll_scare(blessed, cursed, rng, log, turn),
        ScrollType::AmnesiaCurse => scroll_amnesia(blessed, cursed, rng, log, turn),
        ScrollType::Fire => scroll_fire(blessed, cursed, rng, log, turn),
        ScrollType::Earth => scroll_earth(blessed, cursed, rng, log, turn),
        ScrollType::Punishment => scroll_punishment(blessed, cursed, rng, log, turn),
        ScrollType::Stinking => scroll_stinking(blessed, cursed, rng, log, turn),
        ScrollType::Blank => {
            let msg = "This scroll seems to be blank.";
            log.add(msg, turn);
            ScrollResult::new(ScrollType::Blank, msg)
        }
        ScrollType::Genocide => scroll_genocide(blessed, cursed, rng, log, turn),
        ScrollType::Light => scroll_light(blessed, cursed, rng, log, turn),
        ScrollType::Taming => scroll_taming(blessed, cursed, rng, log, turn),
        ScrollType::Charging => scroll_charging(blessed, cursed, rng, log, turn),
        ScrollType::MagicMapping => scroll_magic_mapping(blessed, cursed, confused, rng, log, turn),
        ScrollType::Destroy => scroll_destroy_armor(blessed, cursed, rng, log, turn),
        ScrollType::Mail => {
            let msg = "\"strstrstrstrstr...\" It appears to be junk mail.";
            log.add(msg, turn);
            ScrollResult::new(ScrollType::Mail, msg)
        }
    }
}

// =============================================================================
//
// =============================================================================

///
fn scroll_identify(
    blessed: bool,
    _cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    //
    let count = if blessed { 99 } else { 1 };
    let msg = if blessed {
        "You identify all of your possessions."
    } else {
        "You may identify an item."
    };
    log.add(msg, turn);
    let mut result = ScrollResult::new(ScrollType::Identify, msg);
    result.items_affected = count;
    result
}

///
fn scroll_remove_curse(
    blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "You feel a malevolent aura surround your possessions.";
        log.add_colored(msg, [200, 50, 50], turn);
        ScrollResult::new(ScrollType::RemoveCurse, msg)
    } else if blessed {
        //
        let msg = "You feel as if someone is helping you.";
        log.add_colored(msg, [100, 200, 255], turn);
        let mut result = ScrollResult::new(ScrollType::RemoveCurse, msg);
        result.items_affected = 99;
        result
    } else {
        //
        let msg = "You feel less burdened.";
        log.add(msg, turn);
        let mut result = ScrollResult::new(ScrollType::RemoveCurse, msg);
        result.items_affected = 7;
        result
    }
}

///
fn scroll_enchant(
    blessed: bool,
    cursed: bool,
    confused: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let amount = -(rng.rn2(3) + 1);
        let msg = format!("Your weapon shudders for a moment. (-{})", -amount);
        log.add_colored(&msg, [255, 100, 100], turn);
        let mut result = ScrollResult::new(ScrollType::Enchant, &msg);
        result.items_affected = amount as i32;
        result
    } else if confused {
        //
        let msg = "Your weapon feels warm for a moment.";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Enchant, msg)
    } else {
        //
        let amount = if blessed { rng.rn2(3) + 2 } else { 1 };
        let msg = format!(
            "Your weapon glows {}for a moment. (+{})",
            if blessed { "brilliantly " } else { "" },
            amount
        );
        log.add_colored(&msg, [100, 255, 100], turn);
        let mut result = ScrollResult::new(ScrollType::Enchant, &msg);
        result.items_affected = amount as i32;
        result
    }
}

///
fn scroll_teleport(
    blessed: bool,
    cursed: bool,
    confused: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if confused || cursed {
        //
        let msg = "You feel very disoriented for a moment.";
        log.add_colored(msg, [255, 200, 0], turn);
        ScrollResult::new(ScrollType::Teleportation, msg)
    } else {
        //
        let msg = if blessed {
            "You feel in control of your movements."
        } else {
            "You feel disoriented for a moment."
        };
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Teleportation, msg)
    }
}

///
fn scroll_create_monster(
    blessed: bool,
    cursed: bool,
    confused: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    let count = if confused {
        //
        rng.rn2(4) + 2
    } else if blessed {
        rng.rn2(4) + 1
    } else {
        rng.rn2(5) + 1
    };

    let msg = if confused {
        format!(
            "You have attracted unwanted attention! ({} insects appear)",
            count
        )
    } else {
        format!(
            "{} monster{} {} appears around you!",
            count,
            if count != 1 { "s" } else { "" },
            if count != 1 { "" } else { "" }
        )
    };
    log.add(&msg, turn);
    let mut result = ScrollResult::new(ScrollType::CreateMonster, &msg);
    result.monsters_affected = count as i32;
    result
}

///
fn scroll_gold_detect(
    blessed: bool,
    cursed: bool,
    confused: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if confused {
        //
        let msg = "You feel very greedy, then it passes.";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Gold, msg)
    } else if cursed {
        let msg = "You feel unable to detect gold.";
        log.add(msg, turn);
        ScrollResult::fail(ScrollType::Gold, msg)
    } else {
        let msg = if blessed {
            "You sense the location of all treasure."
        } else {
            "You sense the location of gold."
        };
        log.add_colored(msg, [255, 215, 0], turn);
        ScrollResult::new(ScrollType::Gold, msg)
    }
}

///
fn scroll_food_detect(
    blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        let msg = "You vomit!";
        log.add_colored(msg, [150, 255, 50], turn);
        ScrollResult::new(ScrollType::Food, msg)
    } else {
        let msg = if blessed {
            "You sense all edible objects on this level."
        } else {
            "You smell food nearby."
        };
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Food, msg)
    }
}

///
fn scroll_confuse(
    _blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "You feel confused!";
        log.add_colored(msg, [255, 100, 255], turn);
        ScrollResult::new(ScrollType::Confuse, msg)
    } else {
        //
        let msg = "Your hands begin to glow purple.";
        log.add_colored(msg, [200, 100, 255], turn);
        let mut result = ScrollResult::new(ScrollType::Confuse, msg);
        result.consumed = true;
        result
    }
}

///
fn scroll_scare(
    blessed: bool,
    _cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    //
    let msg = if blessed {
        "You hear maniacal laughter in the distance."
    } else {
        "You hear sad wailing close at hand."
    };
    log.add(msg, turn);
    ScrollResult::new(ScrollType::Scare, msg)
}

///
fn scroll_amnesia(
    _blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "Who was it that said 'strstrstrstrstr!'?";
        log.add_colored(msg, [200, 200, 200], turn);
        ScrollResult::new(ScrollType::AmnesiaCurse, msg)
    } else {
        //
        let msg = "Thinking of Maud you forget everything else.";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::AmnesiaCurse, msg)
    }
}

///
fn scroll_fire(
    blessed: bool,
    cursed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    //
    let damage = rng.rn2(15) + 5;
    let msg = if cursed {
        format!(
            "The scroll erupts in flame! You take {} points of damage!",
            damage
        )
    } else if blessed {
        "The scroll erupts in flame, but you are unharmed.".to_string()
    } else {
        format!(
            "The scroll erupts in flame! ({} damage to nearby monsters)",
            damage
        )
    };
    log.add_colored(&msg, [255, 100, 0], turn);
    ScrollResult::new(ScrollType::Fire, &msg)
}

///
fn scroll_earth(
    _blessed: bool,
    _cursed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    let boulders = rng.rn2(5) + 1;
    let msg = format!(
        "The dungeon rumbles... {} boulder{} fall from the ceiling!",
        boulders,
        if boulders != 1 { "s" } else { "" }
    );
    log.add_colored(&msg, [200, 150, 100], turn);
    let mut result = ScrollResult::new(ScrollType::Earth, &msg);
    result.items_affected = boulders as i32;
    result
}

///
fn scroll_punishment(
    _blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "You are being punished even more for your misbehavior!";
        log.add_colored(msg, [255, 0, 0], turn);
        ScrollResult::new(ScrollType::Punishment, msg)
    } else {
        //
        let msg = "You are being punished for your misbehavior!";
        log.add_colored(msg, [255, 100, 0], turn);
        ScrollResult::new(ScrollType::Punishment, msg)
    }
}

///
fn scroll_stinking(
    _blessed: bool,
    cursed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    let radius = if cursed { 1 } else { rng.rn2(3) + 2 };
    let msg = format!("A stinking cloud forms around you! (radius {})", radius);
    log.add_colored(&msg, [150, 200, 50], turn);
    ScrollResult::new(ScrollType::Stinking, &msg)
}

///
fn scroll_genocide(
    blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "You feel a genocidal mandate against your species!";
        log.add_colored(msg, [255, 0, 0], turn);
        ScrollResult::new(ScrollType::Genocide, msg)
    } else if blessed {
        //
        let msg = "You may choose a class of monsters to genocide.";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Genocide, msg)
    } else {
        //
        let msg = "You may choose a monster type to genocide.";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Genocide, msg)
    }
}

///
fn scroll_light(
    blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        let msg = "Darkness surrounds you.";
        log.add_colored(msg, [100, 100, 100], turn);
        ScrollResult::new(ScrollType::Light, msg)
    } else {
        let msg = if blessed {
            "A brilliant light flood the entire area!"
        } else {
            "A flash of light fills the room!"
        };
        log.add_colored(msg, [255, 255, 200], turn);
        ScrollResult::new(ScrollType::Light, msg)
    }
}

///
fn scroll_taming(
    blessed: bool,
    cursed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    let range = if blessed { 10 } else { 5 };
    if cursed {
        //
        let msg = "The scroll seems to anger nearby creatures!";
        log.add_colored(msg, [255, 100, 100], turn);
        let mut result = ScrollResult::new(ScrollType::Taming, msg);
        result.monsters_affected = rng.rn2(5) as i32;
        result
    } else {
        let msg = if blessed {
            "You feel as if all creatures nearby are your friends."
        } else {
            "You feel at one with nature."
        };
        log.add(msg, turn);
        let mut result = ScrollResult::new(ScrollType::Taming, msg);
        result.monsters_affected = range;
        result
    }
}

///
fn scroll_charging(
    blessed: bool,
    cursed: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        let msg = "You feel a loss of magical energy.";
        log.add_colored(msg, [200, 100, 200], turn);
        ScrollResult::new(ScrollType::Charging, msg)
    } else {
        let msg = if blessed {
            "You may select an item to fully recharge."
        } else {
            "You may select an item to recharge."
        };
        log.add(msg, turn);
        ScrollResult::new(ScrollType::Charging, msg)
    }
}

///
fn scroll_magic_mapping(
    blessed: bool,
    _cursed: bool,
    confused: bool,
    _rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if confused {
        //
        let msg = "Your mind goes blank for a moment...";
        log.add(msg, turn);
        ScrollResult::new(ScrollType::MagicMapping, msg)
    } else {
        let msg = if blessed {
            "An image of your complete surroundings forms in your mind!"
        } else {
            "A map coalesces in your mind!"
        };
        log.add_colored(msg, [100, 200, 255], turn);
        ScrollResult::new(ScrollType::MagicMapping, msg)
    }
}

///
fn scroll_destroy_armor(
    _blessed: bool,
    cursed: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> ScrollResult {
    if cursed {
        //
        let msg = "You feel your armor grow stronger!";
        log.add_colored(msg, [100, 255, 100], turn);
        ScrollResult::new(ScrollType::Destroy, msg)
    } else {
        //
        let slots = ["helmet", "body armor", "cloak", "boots", "gloves", "shield"];
        let idx = rng.rn2(slots.len() as i32) as usize;
        let msg = format!("Your {} crumbles to dust!", slots[idx]);
        log.add_colored(&msg, [255, 100, 100], turn);
        ScrollResult::new(ScrollType::Destroy, &msg)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn can_read(
    item_class: ItemClass,
    blind: bool,
    confused: bool,
    hallucinating: bool,
) -> Result<(), &'static str> {
    if blind {
        return Err("You can't see to read.");
    }
    if item_class != ItemClass::Scroll && item_class != ItemClass::Spellbook {
        return Err("That is not a readable item.");
    }
    //
    Ok(())
}

///
pub fn reading_difficulty(confusion: bool, hallucination: bool) -> i32 {
    let mut difficulty = 0;
    if confusion {
        difficulty += 3;
    }
    if hallucination {
        difficulty += 5;
    }
    difficulty
}

// =============================================================================
//
// =============================================================================

///
pub fn random_scroll_appearance(rng: &mut NetHackRng) -> &'static str {
    let appearances = [
        "ZELGO MER",
        "JUYED AWK YACC",
        "NR 9",
        "XIXAXA XOXAXA XUXAXA",
        "PRATYAVAYAH",
        "DAIYEN FANSEN",
        "LEP GEX VEN ZEA",
        "PRIRUTSENIE",
        "ELBIB YLANSEN",
        "VERR YED HANSEN",
        "VENZAR BORGAVVE",
        "THARR",
        "YUM YUM",
        "KERNOD WEL",
        "ELAM EANSEN",
        "DUAM XNAHT",
        "ANDOVA BEGARIN",
        "KIRJE",
        "VE FORBRANSEN",
        "HACKEM MUCHE",
        "VELOX NEB",
        "FOOBIE BLETCH",
        "TEMOV",
        "GARVEN DEH",
        "READ ME",
    ];
    let idx = rng.rn2(appearances.len() as i32) as usize;
    appearances[idx]
}

// =============================================================================
//
// =============================================================================

///
pub fn study_spellbook(
    spell_name: &str,
    spell_level: i32,
    player_intelligence: i32,
    confused: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> (bool, i32) {
    //
    let delay = spell_level * 10;

    //
    if confused {
        let msg = "Being confused, you cannot concentrate on the spellbook.";
        log.add(msg, turn);
        return (false, 0);
    }

    //
    let success_chance = player_intelligence + 10 - spell_level * 3;
    if rng.rn2(success_chance.max(1)) == 0 {
        let msg = format!("You fail to understand the spellbook of {}.", spell_name);
        log.add(&msg, turn);
        return (false, delay);
    }

    let msg = format!("You learn the spell of {}.", spell_name);
    log.add_colored(&msg, [100, 200, 255], turn);
    (true, delay)
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_name_mapping() {
        assert_eq!(
            scroll_name_to_type("scroll of identify"),
            Some(ScrollType::Identify)
        );
        assert_eq!(
            scroll_name_to_type("scroll of teleportation"),
            Some(ScrollType::Teleportation)
        );
        assert_eq!(
            scroll_name_to_type("scroll of magic mapping"),
            Some(ScrollType::MagicMapping)
        );
        assert_eq!(scroll_name_to_type("unknown scroll"), None);
    }

    #[test]
    fn test_can_read() {
        assert!(can_read(ItemClass::Scroll, false, false, false).is_ok());
        assert!(can_read(ItemClass::Scroll, true, false, false).is_err());
        assert!(can_read(ItemClass::Weapon, false, false, false).is_err());
    }

    #[test]
    fn test_scroll_identify() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = scroll_identify(true, false, &mut rng, &mut log, 1);
        assert_eq!(result.items_affected, 99);
        assert!(result.success);
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
pub fn scroll_rarity(scroll_type: ScrollType) -> i32 {
    match scroll_type {
        ScrollType::Genocide => 1,
        ScrollType::Charging => 2,
        ScrollType::Enchant => 3,
        ScrollType::MagicMapping => 3,
        ScrollType::Taming => 2,
        ScrollType::Identify => 5,
        ScrollType::RemoveCurse => 4,
        ScrollType::Teleportation => 4,
        ScrollType::CreateMonster => 4,
        ScrollType::Fire => 3,
        ScrollType::Earth => 3,
        ScrollType::Light => 5,
        ScrollType::Confuse => 4,
        ScrollType::Scare => 3,
        ScrollType::AmnesiaCurse => 2,
        ScrollType::Gold => 3,
        ScrollType::Food => 4,
        ScrollType::Blank => 5,
        ScrollType::Punishment => 2,
        ScrollType::Stinking => 3,
        ScrollType::Destroy => 2,
        ScrollType::Mail => 1,
    }
}

///
pub fn scroll_base_cost(scroll_type: ScrollType) -> i32 {
    match scroll_type {
        ScrollType::Genocide => 300,
        ScrollType::Charging => 250,
        ScrollType::Enchant => 200,
        ScrollType::MagicMapping => 100,
        ScrollType::Taming => 200,
        ScrollType::Identify => 20,
        ScrollType::RemoveCurse => 80,
        ScrollType::Teleportation => 100,
        ScrollType::CreateMonster => 200,
        ScrollType::Fire => 100,
        ScrollType::Earth => 200,
        ScrollType::Light => 50,
        ScrollType::Confuse => 100,
        ScrollType::Scare => 100,
        ScrollType::AmnesiaCurse => 200,
        ScrollType::Gold => 100,
        ScrollType::Food => 100,
        ScrollType::Blank => 60,
        ScrollType::Punishment => 150,
        ScrollType::Stinking => 100,
        ScrollType::Destroy => 100,
        ScrollType::Mail => 0,
    }
}

///
pub fn scroll_buc_multiplier(blessed: bool, cursed: bool) -> f32 {
    if blessed {
        1.5
    } else if cursed {
        0.5
    } else {
        1.0
    }
}

///
pub fn spellbook_explosion_risk(
    spell_level: i32,
    player_intelligence: i32,
    player_level: i32,
) -> i32 {
    //
    let difficulty = spell_level * 4 - player_intelligence - player_level;
    if difficulty <= 0 {
        0
    } else {
        (difficulty * 5).min(80)
    }
}

///
pub fn spellbook_study_turns(spell_level: i32) -> i32 {
    spell_level * 10 + 5
}

///
pub fn spellbook_failure_effect(spell_level: i32) -> &'static str {
    if spell_level >= 7 {
        "The spellbook explodes in your hands!"
    } else if spell_level >= 5 {
        "You feel a surge of arcane energy burn through you!"
    } else if spell_level >= 3 {
        "The spellbook crumbles to dust."
    } else {
        "The words blur before your eyes."
    }
}

///
pub fn scroll_identify_hint(scroll_type: ScrollType) -> &'static str {
    match scroll_type {
        ScrollType::Fire => "The scroll feels warm to the touch.",
        ScrollType::Light => "The scroll seems to glow faintly.",
        ScrollType::Scare => "You sense an aura of dread.",
        ScrollType::Enchant => "The scroll tingles in your hands.",
        ScrollType::Teleportation => "The scroll feels slippery.",
        ScrollType::Genocide => "The scroll feels heavy with power.",
        _ => "You notice nothing special about this scroll.",
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct ReadStatistics {
    pub scrolls_read: u32,
    pub scrolls_wasted: u32,
    pub spellbooks_studied: u32,
    pub spellbooks_failed: u32,
    pub items_identified: u32,
    pub curses_removed: u32,
}

impl ReadStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_read(&mut self) {
        self.scrolls_read += 1;
    }
    pub fn record_waste(&mut self) {
        self.scrolls_wasted += 1;
    }
    pub fn record_study(&mut self, success: bool) {
        self.spellbooks_studied += 1;
        if !success {
            self.spellbooks_failed += 1;
        }
    }
}

#[cfg(test)]
mod read_extended_tests {
    use super::*;

    #[test]
    fn test_rarity() {
        assert!(scroll_rarity(ScrollType::Identify) > scroll_rarity(ScrollType::Genocide));
    }

    #[test]
    fn test_base_cost() {
        assert!(scroll_base_cost(ScrollType::Genocide) > scroll_base_cost(ScrollType::Identify));
    }

    #[test]
    fn test_buc_multiplier() {
        assert!(scroll_buc_multiplier(true, false) > 1.0);
        assert!(scroll_buc_multiplier(false, true) < 1.0);
    }

    #[test]
    fn test_explosion_risk() {
        //
        assert!(spellbook_explosion_risk(7, 10, 5) > 0);
        //
        assert_eq!(spellbook_explosion_risk(1, 18, 15), 0);
    }

    #[test]
    fn test_study_turns() {
        assert!(spellbook_study_turns(7) > spellbook_study_turns(1));
    }

    #[test]
    fn test_identify_hint() {
        let h = scroll_identify_hint(ScrollType::Fire);
        assert!(h.contains("warm"));
    }

    #[test]
    fn test_read_stats() {
        let mut s = ReadStatistics::new();
        s.record_read();
        s.record_study(false);
        assert_eq!(s.scrolls_read, 1);
        assert_eq!(s.spellbooks_failed, 1);
    }
}
