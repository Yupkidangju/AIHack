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
// [v2.3.0
//
//
//
//
//
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodType {
    ///
    Ration, // K-ration, C-ration, food ration
    ///
    Corpse,
    ///
    Tin,
    ///
    Egg,
    ///
    Fruit,
    ///
    Wafer,
    ///
    Carrot,
    ///
    Garlic,
    ///
    RoyalJelly,
    ///
    Wolfsbane,
    ///
    Eucalyptus,
    ///
    Glob,
    ///
    Other,
}

///
#[derive(Debug, Clone)]
pub struct EatingState {
    ///
    pub food_name: String,
    ///
    pub remaining_turns: i32,
    ///
    pub food_type: FoodType,
    ///
    pub blessed: bool,
    pub cursed: bool,
    ///
    pub in_progress: bool,
    ///
    pub corpse_symbol: Option<char>,
    ///
    pub corpse_monster: Option<String>,
}

impl EatingState {
    pub fn new() -> Self {
        Self {
            food_name: String::new(),
            remaining_turns: 0,
            food_type: FoodType::Other,
            blessed: false,
            cursed: false,
            in_progress: false,
            corpse_symbol: None,
            corpse_monster: None,
        }
    }

    ///
    pub fn start(
        &mut self,
        name: &str,
        food_type: FoodType,
        blessed: bool,
        cursed: bool,
        turns: i32,
    ) {
        self.food_name = name.to_string();
        self.food_type = food_type;
        self.blessed = blessed;
        self.cursed = cursed;
        self.remaining_turns = turns;
        self.in_progress = true;
    }

    ///
    pub fn reset(&mut self) {
        self.in_progress = false;
        self.remaining_turns = 0;
        self.food_name.clear();
        self.corpse_symbol = None;
        self.corpse_monster = None;
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn food_nutrition(food_type: FoodType, blessed: bool, cursed: bool) -> i32 {
    let base = match food_type {
        FoodType::Ration => 800,
        FoodType::Corpse => 0,
        FoodType::Tin => 200,
        FoodType::Egg => 80,
        FoodType::Fruit => 250,
        FoodType::Wafer => 400,
        FoodType::Carrot => 50,
        FoodType::Garlic => 40,
        FoodType::RoyalJelly => 200,
        FoodType::Wolfsbane => 40,
        FoodType::Eucalyptus => 30,
        FoodType::Glob => 300,
        FoodType::Other => 100,
    };
    //
    if blessed {
        base + base / 2
    } else if cursed {
        base - base * 3 / 10
    } else {
        base
    }
}

///
///
pub fn corpse_nutrition(monster_weight: i32, monster_symbol: char) -> i32 {
    //
    let multiplier = match monster_symbol {
        'D' => 3,
        'H' | 'T' => 2,
        _ => 1,
    };
    (monster_weight * multiplier).max(10)
}

// =============================================================================
//
// =============================================================================

///
pub fn eating_turns(food_type: FoodType, nutrition: i32) -> i32 {
    match food_type {
        FoodType::Ration => 5,
        FoodType::Corpse => {
            //
            (nutrition / 100).clamp(1, 20)
        }
        FoodType::Tin => 3,
        FoodType::Egg => 1,
        FoodType::Fruit => 1,
        FoodType::Wafer => 2,
        FoodType::Carrot => 1,
        FoodType::Garlic | FoodType::Wolfsbane | FoodType::Eucalyptus => 1,
        FoodType::RoyalJelly => 1,
        FoodType::Glob => 2,
        FoodType::Other => 2,
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerLevel {
    Satiated = 0,
    Normal = 1,
    Hungry = 2,
    Weak = 3,
    Fainting = 4,
    Starved = 5,
}

///
pub fn nutrition_to_hunger(nutrition: i32) -> HungerLevel {
    if nutrition >= 1000 {
        HungerLevel::Satiated
    } else if nutrition >= 150 {
        HungerLevel::Normal
    } else if nutrition >= 50 {
        HungerLevel::Hungry
    } else if nutrition > 0 {
        HungerLevel::Weak
    } else if nutrition > -200 {
        HungerLevel::Fainting
    } else {
        HungerLevel::Starved
    }
}

///
pub fn hunger_name(level: HungerLevel) -> &'static str {
    match level {
        HungerLevel::Satiated => "Satiated",
        HungerLevel::Normal => "",
        HungerLevel::Hungry => "Hungry",
        HungerLevel::Weak => "Weak",
        HungerLevel::Fainting => "Fainting",
        HungerLevel::Starved => "Starved",
    }
}

///
///
pub fn nutrition_drain_per_turn(
    is_regenerating: bool,
    is_hungry_ring: bool,
    _encumbrance: i32,
) -> i32 {
    let mut drain = 1;
    if is_regenerating {
        drain += 1;
    }
    if is_hungry_ring {
        drain += 1;
    }
    drain
}

// =============================================================================
//
// =============================================================================

///
///
pub fn is_choking(nutrition: i32) -> bool {
    nutrition >= 2000
}

///
pub fn choking_message(nutrition: i32) -> &'static str {
    if nutrition >= 3000 {
        "You choke over your food."
    } else if nutrition >= 2500 {
        "You are having a hard time getting all of it down."
    } else if nutrition >= 2000 {
        "You are about to choke."
    } else {
        ""
    }
}

// =============================================================================
//
// =============================================================================

///
///
#[derive(Debug, Clone)]
pub struct CorpseEffect {
    ///
    pub resistance_gained: Option<&'static str>,
    ///
    pub hp_change: i32,
    ///
    pub status_effect: Option<(&'static str, u32)>,
    /// 硫붿떆吏
    pub message: String,
    ///
    pub food_poisoning: bool,
    ///
    pub stoning: bool,
    ///
    pub sliming: bool,
    ///
    pub poisoned: bool,
    ///
    pub teleportitis: bool,
    ///
    pub permanent_blind: bool,
    ///
    pub hallucination: bool,
    ///
    pub nutrition_bonus: i32,
}

impl CorpseEffect {
    fn none(msg: &str) -> Self {
        Self {
            resistance_gained: None,
            hp_change: 0,
            status_effect: None,
            message: msg.to_string(),
            food_poisoning: false,
            stoning: false,
            sliming: false,
            poisoned: false,
            teleportitis: false,
            permanent_blind: false,
            hallucination: false,
            nutrition_bonus: 0,
        }
    }
}

///
///
pub fn corpse_effect(
    monster_name: &str,
    monster_symbol: char,
    blessed: bool,
    rng: &mut NetHackRng,
) -> CorpseEffect {
    let mut effect = CorpseEffect::none("This corpse tastes normal.");

    match monster_symbol {
        //
        'D' => {
            let resist = match monster_name {
                n if n.contains("red") => Some("fire"),
                n if n.contains("white") => Some("cold"),
                n if n.contains("blue") => Some("elec"),
                n if n.contains("orange") => Some("sleep"),
                n if n.contains("black") => Some("disintegrate"),
                n if n.contains("green") => Some("poison"),
                n if n.contains("yellow") => Some("acid"),
                _ => None,
            };
            if let Some(r) = resist {
                effect.resistance_gained = Some(r);
                effect.message = format!("You feel a momentary chill. (gained {} resistance)", r);
            }
        }

        //
        'S' => {
            if !blessed && rng.rn2(5) == 0 {
                effect.poisoned = true;
                effect.hp_change = -rng.rn2(15) as i32 - 5;
                effect.message = "Ecch - that must have been poisonous!".to_string();
            } else {
                //
                if rng.rn2(3) == 0 {
                    effect.resistance_gained = Some("poison");
                    effect.message = "You feel hardier.".to_string();
                }
            }
        }

        //
        'c' => {
            if monster_name.contains("cockatrice") || monster_name.contains("chickatrice") {
                if !blessed {
                    effect.stoning = true;
                    effect.message = "You don't feel very well...".to_string();
                }
            }
        }

        //
        'P' => {
            if monster_name.contains("green slime") {
                effect.sliming = true;
                effect.message = "You begin to feel slimy.".to_string();
            }
        }

        //
        ':' => {
            if monster_name.contains("newt") && rng.rn2(3) == 0 {
                effect.message = "You feel a mild buzz.".to_string();
                //
            }
        }

        //
        'n' => {
            if rng.rn2(4) == 0 {
                effect.teleportitis = true;
                effect.message = "You feel very jumpy.".to_string();
            }
        }

        //
        'W' => {
            if monster_name.contains("wraith") {
                //
                effect.nutrition_bonus = 100;
                effect.message = "You feel yourself rising...".to_string();
            }
        }

        //
        'T' => {
            if !blessed && rng.rn2(3) == 0 {
                effect.food_poisoning = true;
                effect.message = "You feel sick.".to_string();
            }
        }

        //
        '@' => {
            effect.message = "You cannibal! You feel terrible!".to_string();
            effect.hp_change = -rng.rn2(5) as i32;
            //
        }

        //
        'h' if monster_name.contains("mind flayer") => {
            if rng.rn2(2) == 0 {
                effect.message = "You feel intellectually stimulated.".to_string();
                effect.nutrition_bonus = 50;
            } else {
                effect.hallucination = true;
                effect.message = "Your head spins wildly!".to_string();
            }
        }

        //
        'H' => {
            if rng.rn2(4) == 0 {
                effect.message = "You feel stronger.".to_string();
                //
            }
        }

        //
        'a' => {
            if rng.rn2(5) == 0 {
                effect.poisoned = true;
                effect.hp_change = -rng.rn2(4) as i32;
                effect.message = "You feel slightly ill.".to_string();
            }
        }

        _ => {}
    }

    effect
}

// =============================================================================
//
// =============================================================================

///
///
pub fn is_corpse_tainted(
    corpse_age: u64,
    current_turn: u64,
    is_lizard: bool,
    is_blessed: bool,
) -> bool {
    //
    if is_lizard {
        return false;
    }
    //
    let threshold = if is_blessed { 200 } else { 100 };
    let age = current_turn.saturating_sub(corpse_age);
    age > threshold
}

///
pub fn eat_tainted_corpse(rng: &mut NetHackRng) -> CorpseEffect {
    let mut e = CorpseEffect::none("Ulch - that food was tainted!");
    e.food_poisoning = true;
    e.hp_change = -(rng.rn2(10) as i32 + 5);
    e.status_effect = Some(("sick", 20));
    e
}

// =============================================================================
//
// =============================================================================

///
pub fn can_eat_metal(eater_symbol: char) -> bool {
    matches!(eater_symbol, 'X' | '\'')
}

///
pub fn metal_nutrition(item_weight: i32) -> i32 {
    (item_weight * 2).max(10)
}

// =============================================================================
//
// =============================================================================

///
pub fn special_food_effect(food_type: FoodType, blessed: bool) -> &'static str {
    match food_type {
        FoodType::Carrot => "Your vision sharpens momentarily.",
        FoodType::Garlic => "You feel a strong sense of garlic.",
        FoodType::Wolfsbane => "You feel purified.",
        FoodType::Eucalyptus => "You feel much better.",
        FoodType::RoyalJelly => {
            if blessed {
                "You feel a surge of vitality!"
            } else {
                "You feel slightly better."
            }
        }
        FoodType::Wafer => "This food is very filling!",
        _ => "",
    }
}

///
pub fn eat_carrot_effect(is_blind: bool) -> Option<&'static str> {
    if is_blind {
        Some("Your vision clears!")
    } else {
        None
    }
}

///
pub fn eat_garlic_effect() -> &'static str {
    "You now repel undead."
}

///
pub fn eat_eucalyptus_effect(is_sick: bool) -> Option<&'static str> {
    if is_sick {
        Some("You feel cured. What a relief!")
    } else {
        None
    }
}

///
pub fn eat_wolfsbane_effect(is_lycanthrope: bool) -> Option<&'static str> {
    if is_lycanthrope {
        Some("You feel purified of the curse.")
    } else {
        None
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn food_name_to_type(name: &str) -> FoodType {
    let l = name.to_lowercase();
    if l.contains("corpse") {
        FoodType::Corpse
    } else if l.contains("tin") {
        FoodType::Tin
    } else if l.contains("egg") {
        FoodType::Egg
    } else if l.contains("ration") || l.contains("k-ration") || l.contains("c-ration") {
        FoodType::Ration
    } else if l.contains("carrot") {
        FoodType::Carrot
    } else if l.contains("garlic") {
        FoodType::Garlic
    } else if l.contains("royal jelly") {
        FoodType::RoyalJelly
    } else if l.contains("wolfsbane") || l.contains("sprig") {
        FoodType::Wolfsbane
    } else if l.contains("eucalyptus") {
        FoodType::Eucalyptus
    } else if l.contains("lembas") || l.contains("cram") || l.contains("wafer") {
        FoodType::Wafer
    } else if l.contains("glob") {
        FoodType::Glob
    } else if l.contains("fruit")
        || l.contains("apple")
        || l.contains("melon")
        || l.contains("banana")
        || l.contains("orange")
        || l.contains("pear")
        || l.contains("slime mold")
    {
        FoodType::Fruit
    } else {
        FoodType::Other
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn tin_opening_turns(has_can_opener: bool) -> i32 {
    if has_can_opener {
        1
    } else {
        //
        10
    }
}

///
pub fn tin_contents(rng: &mut NetHackRng) -> (&'static str, i32) {
    let roll = rng.rn2(10);
    match roll {
        0 => ("It contains spinach!", 600),
        1 => ("It contains deep fried food.", 300),
        2 => ("It contains sauteed food.", 300),
        3 => ("It contains pickled food.", 200),
        4 => ("It contains soup.", 250),
        5 => ("It contains rotten food.", 50),
        6 => ("It contains pureed food.", 200),
        7 => ("It smells terrible! (cursed)", 50),
        _ => ("It contains food.", 200),
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn egg_might_hatch(egg_age: u64, current_turn: u64, rng: &mut NetHackRng) -> bool {
    let age = current_turn.saturating_sub(egg_age);
    //
    age > 100 && rng.rn2(5) == 0
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_nutrition() {
        assert_eq!(food_nutrition(FoodType::Ration, false, false), 800);
        assert_eq!(food_nutrition(FoodType::Ration, true, false), 1200);
        assert!(food_nutrition(FoodType::Ration, false, true) < 800);
    }

    #[test]
    fn test_hunger_levels() {
        assert_eq!(nutrition_to_hunger(1500), HungerLevel::Satiated);
        assert_eq!(nutrition_to_hunger(500), HungerLevel::Normal);
        assert_eq!(nutrition_to_hunger(100), HungerLevel::Hungry);
        assert_eq!(nutrition_to_hunger(25), HungerLevel::Weak);
        assert_eq!(nutrition_to_hunger(-50), HungerLevel::Fainting);
        assert_eq!(nutrition_to_hunger(-300), HungerLevel::Starved);
    }

    #[test]
    fn test_corpse_tainted() {
        assert!(!is_corpse_tainted(0, 50, false, false));
        assert!(is_corpse_tainted(0, 200, false, false));
        assert!(!is_corpse_tainted(0, 200, true, false));
        assert!(!is_corpse_tainted(0, 150, false, true));
    }

    #[test]
    fn test_food_name_to_type() {
        assert_eq!(food_name_to_type("food ration"), FoodType::Ration);
        assert_eq!(food_name_to_type("a corpse"), FoodType::Corpse);
        assert_eq!(food_name_to_type("tin of spinach"), FoodType::Tin);
        assert_eq!(food_name_to_type("carrot"), FoodType::Carrot);
    }

    #[test]
    fn test_choking() {
        assert!(!is_choking(1500));
        assert!(is_choking(2000));
        assert!(is_choking(3000));
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodAllergy {
    None,
    Mild,
    Moderate,
    Severe,
    Deadly,
}

///
pub fn food_allergy(race: &str, food_type: FoodType) -> FoodAllergy {
    match (race, food_type) {
        ("elf", FoodType::Garlic) => FoodAllergy::Mild,
        ("orc", FoodType::Wafer) => FoodAllergy::Mild,
        ("vampire", FoodType::Garlic) => FoodAllergy::Severe,
        _ => FoodAllergy::None,
    }
}

///
pub fn cooking_bonus(food_type: FoodType) -> i32 {
    match food_type {
        FoodType::Corpse => 50,
        FoodType::Tin => 20,
        FoodType::Egg => 30,
        _ => 0,
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreservationMethod {
    None,
    Blessed,
    IceBox,
    Tin,
    DriedMeat,
}

///
pub fn preservation_delay(method: PreservationMethod) -> u64 {
    match method {
        PreservationMethod::None => 0,
        PreservationMethod::Blessed => 100,
        PreservationMethod::IceBox => u64::MAX,
        PreservationMethod::Tin => u64::MAX,
        PreservationMethod::DriedMeat => 500,
    }
}

///
pub fn eating_speed_modifier(is_ring_hunger: bool, is_starving: bool, is_satiated: bool) -> f32 {
    if is_starving {
        2.0
    }
    //
    else if is_satiated {
        0.5
    }
    //
    else if is_ring_hunger {
        1.5
    } else {
        1.0
    }
}

///
pub fn hunger_transition_message(from: HungerLevel, to: HungerLevel) -> &'static str {
    match (from, to) {
        (HungerLevel::Normal, HungerLevel::Satiated) => "You are beginning to feel full.",
        (HungerLevel::Normal, HungerLevel::Hungry) => "You are beginning to feel hungry.",
        (HungerLevel::Hungry, HungerLevel::Weak) => "You feel weak from lack of food.",
        (HungerLevel::Weak, HungerLevel::Fainting) => "You feel faint from lack of food.",
        (HungerLevel::Fainting, HungerLevel::Starved) => "You die from starvation.",
        (HungerLevel::Hungry, HungerLevel::Normal) => "You no longer feel hungry.",
        (HungerLevel::Weak, HungerLevel::Hungry) => "You feel less weak.",
        (HungerLevel::Satiated, HungerLevel::Normal) => "You no longer feel satiated.",
        _ => "",
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct EatStatistics {
    pub meals_eaten: u32,
    pub corpses_eaten: u32,
    pub resistances_gained: u32,
    pub food_poisonings: u32,
    pub total_nutrition: i64,
    pub times_choked: u32,
}

impl EatStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_meal(&mut self, food_type: FoodType, nutrition: i32) {
        self.meals_eaten += 1;
        self.total_nutrition += nutrition as i64;
        if food_type == FoodType::Corpse {
            self.corpses_eaten += 1;
        }
    }
}

#[cfg(test)]
mod eat_extended_tests {
    use super::*;

    #[test]
    fn test_allergy() {
        assert_eq!(
            food_allergy("vampire", FoodType::Garlic),
            FoodAllergy::Severe
        );
        assert_eq!(food_allergy("human", FoodType::Ration), FoodAllergy::None);
    }

    #[test]
    fn test_cooking() {
        assert!(cooking_bonus(FoodType::Corpse) > 0);
        assert_eq!(cooking_bonus(FoodType::Ration), 0);
    }

    #[test]
    fn test_preservation() {
        assert_eq!(preservation_delay(PreservationMethod::IceBox), u64::MAX);
        assert_eq!(preservation_delay(PreservationMethod::None), 0);
    }

    #[test]
    fn test_eating_speed() {
        assert!(eating_speed_modifier(false, true, false) > 1.0);
    }

    #[test]
    fn test_hunger_message() {
        let msg = hunger_transition_message(HungerLevel::Normal, HungerLevel::Hungry);
        assert!(msg.contains("hungry"));
    }

    #[test]
    fn test_eat_stats() {
        let mut s = EatStatistics::new();
        s.record_meal(FoodType::Corpse, 200);
        assert_eq!(s.corpses_eaten, 1);
        assert_eq!(s.total_nutrition, 200);
    }
}
