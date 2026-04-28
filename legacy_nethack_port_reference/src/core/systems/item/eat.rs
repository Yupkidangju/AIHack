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

// =============================================================================
// [v2.4.0] eat.c 대량 이식
// 원본: nethack-3.6.7/src/eat.c (3,353줄)
// 통조림 시스템, 내성 획득, 시체 후처리, 식사 전처리, 식용 판정,
// 뇌 먹기, 식인 판정, 채식주의, 영양소 계산 등
// =============================================================================

/// [v2.4.0] 통조림 조리 방식 (원본: tintxts[] 인덱스)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinVariety {
    Rotten = 0,
    Homemade = 1,
    Soup = 2,
    FrenchFried = 3,
    Pickled = 4,
    Boiled = 5,
    Smoked = 6,
    Dried = 7,
    DeepFried = 8,
    Szechuan = 9,
    Broiled = 10,
    StirFried = 11,
    Sauteed = 12,
    Candied = 13,
    Pureed = 14,
    Spinach = 15,
}

/// [v2.4.0] 통조림 조리법 정보 (원본: eat.c:128-148)
#[derive(Debug, Clone)]
pub struct TinText {
    pub txt: &'static str,
    pub nut: i32,
    pub fodder: bool,
    pub greasy: bool,
}

/// [v2.4.0] 통조림 조리법 테이블 (원본: tintxts[])
pub fn tin_text_table() -> Vec<TinText> {
    vec![
        TinText {
            txt: "rotten",
            nut: -50,
            fodder: false,
            greasy: false,
        },
        TinText {
            txt: "homemade",
            nut: 50,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "soup made from",
            nut: 20,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "french fried",
            nut: 40,
            fodder: false,
            greasy: true,
        },
        TinText {
            txt: "pickled",
            nut: 40,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "boiled",
            nut: 50,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "smoked",
            nut: 50,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "dried",
            nut: 55,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "deep fried",
            nut: 60,
            fodder: false,
            greasy: true,
        },
        TinText {
            txt: "szechuan",
            nut: 70,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "broiled",
            nut: 80,
            fodder: false,
            greasy: false,
        },
        TinText {
            txt: "stir fried",
            nut: 80,
            fodder: false,
            greasy: true,
        },
        TinText {
            txt: "sauteed",
            nut: 95,
            fodder: false,
            greasy: false,
        },
        TinText {
            txt: "candied",
            nut: 100,
            fodder: true,
            greasy: false,
        },
        TinText {
            txt: "pureed",
            nut: 500,
            fodder: true,
            greasy: false,
        },
    ]
}

/// [v2.4.0] 통조림 조리 방식 판별 (원본: eat.c:1277-1300 tin_variety)
pub fn tin_variety_from_spe(spe: i32, cursed: bool, rng: &mut NetHackRng) -> TinVariety {
    if spe == 1 {
        return TinVariety::Spinach;
    }
    if cursed {
        return TinVariety::Rotten;
    }
    if spe < 0 {
        let r = ((-spe) - 1) as usize;
        if r < 15 {
            return match r {
                0 => TinVariety::Rotten,
                1 => TinVariety::Homemade,
                2 => TinVariety::Soup,
                3 => TinVariety::FrenchFried,
                4 => TinVariety::Pickled,
                5 => TinVariety::Boiled,
                6 => TinVariety::Smoked,
                7 => TinVariety::Dried,
                8 => TinVariety::DeepFried,
                9 => TinVariety::Szechuan,
                10 => TinVariety::Broiled,
                11 => TinVariety::StirFried,
                12 => TinVariety::Sauteed,
                13 => TinVariety::Candied,
                14 => TinVariety::Pureed,
                _ => TinVariety::Homemade,
            };
        }
    }
    let r = rng.rn2(15) as usize;
    match r {
        0 => TinVariety::Rotten,
        1 => TinVariety::Homemade,
        2 => TinVariety::Soup,
        3 => TinVariety::FrenchFried,
        4 => TinVariety::Pickled,
        5 => TinVariety::Boiled,
        6 => TinVariety::Smoked,
        7 => TinVariety::Dried,
        8 => TinVariety::DeepFried,
        9 => TinVariety::Szechuan,
        10 => TinVariety::Broiled,
        11 => TinVariety::StirFried,
        12 => TinVariety::Sauteed,
        13 => TinVariety::Candied,
        _ => TinVariety::Pureed,
    }
}

/// [v2.4.0] 통조림 소비 결과
#[derive(Debug, Clone)]
pub struct TinConsumeResult {
    pub message: String,
    pub nutrition: i32,
    pub greasy: bool,
    pub vomit: bool,
    pub trapped: bool,
    pub empty: bool,
}

/// [v2.4.0] 통조림 소비 로직 (원본: eat.c:1302-1400 consume_tin)
pub fn consume_tin(
    spe: i32,
    cursed: bool,
    trapped: bool,
    corpse_name: Option<&str>,
    rng: &mut NetHackRng,
) -> TinConsumeResult {
    let variety = tin_variety_from_spe(spe, cursed, rng);
    if trapped || (cursed && variety != TinVariety::Homemade && rng.rn2(8) == 0) {
        return TinConsumeResult {
            message: "STREWTH! The tin was booby-trapped!".into(),
            nutrition: 0,
            greasy: false,
            vomit: false,
            trapped: true,
            empty: false,
        };
    }
    if variety == TinVariety::Spinach {
        let msg = if cursed {
            "It contains some decaying green substance."
        } else {
            "It contains spinach."
        };
        return TinConsumeResult {
            message: msg.into(),
            nutrition: 600,
            greasy: false,
            vomit: false,
            trapped: false,
            empty: false,
        };
    }
    let monster = match corpse_name {
        Some(n) if !n.is_empty() => n,
        _ => {
            return TinConsumeResult {
                message: "It turns out to be empty.".into(),
                nutrition: 0,
                greasy: false,
                vomit: false,
                trapped: false,
                empty: true,
            }
        }
    };
    let table = tin_text_table();
    let idx = variety as usize;
    let tt = if idx < table.len() {
        &table[idx]
    } else {
        &table[1]
    };
    TinConsumeResult {
        message: format!("You consume {} {}.", tt.txt, monster),
        nutrition: if tt.nut < 0 { 0 } else { tt.nut },
        greasy: tt.greasy,
        vomit: tt.nut < 0,
        trapped: false,
        empty: false,
    }
}

// ---------------------------------------------------------------------------
// 내성 획득 시스템 (원본: eat.c:766-941)
// ---------------------------------------------------------------------------

/// [v2.4.0] 부여 가능한 내성 종류 (원본: intrinsic_possible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConveyableIntrinsic {
    FireRes,
    SleepRes,
    ColdRes,
    DisintRes,
    ShockRes,
    PoisonRes,
    Teleport,
    TeleportControl,
    Telepathy,
}

/// [v2.4.0] mconveys 플래그로부터 부여 가능 내성 목록 (원본: intrinsic_possible)
pub fn intrinsic_possible(
    conveys: u32,
    can_tp: bool,
    ctrl_tp: bool,
    telepathic: bool,
) -> Vec<ConveyableIntrinsic> {
    let mut r = Vec::new();
    if conveys & 0x01 != 0 {
        r.push(ConveyableIntrinsic::FireRes);
    }
    if conveys & 0x02 != 0 {
        r.push(ConveyableIntrinsic::SleepRes);
    }
    if conveys & 0x04 != 0 {
        r.push(ConveyableIntrinsic::ColdRes);
    }
    if conveys & 0x08 != 0 {
        r.push(ConveyableIntrinsic::DisintRes);
    }
    if conveys & 0x10 != 0 {
        r.push(ConveyableIntrinsic::ShockRes);
    }
    if conveys & 0x20 != 0 {
        r.push(ConveyableIntrinsic::PoisonRes);
    }
    if can_tp {
        r.push(ConveyableIntrinsic::Teleport);
    }
    if ctrl_tp {
        r.push(ConveyableIntrinsic::TeleportControl);
    }
    if telepathic {
        r.push(ConveyableIntrinsic::Telepathy);
    }
    r
}

/// [v2.4.0] 내성 획득 시도 결과 (원본: givit)
#[derive(Debug, Clone)]
pub struct IntrinsicGainResult {
    pub gained: bool,
    pub intrinsic: ConveyableIntrinsic,
    pub message: &'static str,
}

/// [v2.4.0] 내성 획득 확률 판정 (원본: eat.c:828-941 givit)
pub fn try_give_intrinsic(
    intrinsic: ConveyableIntrinsic,
    monster_level: i32,
    already_has: bool,
    hallucinating: bool,
    rng: &mut NetHackRng,
) -> IntrinsicGainResult {
    let chance = match intrinsic {
        ConveyableIntrinsic::Teleport => 10,
        ConveyableIntrinsic::TeleportControl => 12,
        ConveyableIntrinsic::Telepathy => 1,
        _ => 15,
    };
    if monster_level <= rng.rn2(chance) as i32 || already_has {
        return IntrinsicGainResult {
            gained: false,
            intrinsic,
            message: "",
        };
    }
    let msg = match intrinsic {
        ConveyableIntrinsic::FireRes => {
            if hallucinating {
                "You be chillin'."
            } else {
                "You feel a momentary chill."
            }
        }
        ConveyableIntrinsic::SleepRes => "You feel wide awake.",
        ConveyableIntrinsic::ColdRes => "You feel full of hot air.",
        ConveyableIntrinsic::DisintRes => {
            if hallucinating {
                "You feel totally together, man."
            } else {
                "You feel very firm."
            }
        }
        ConveyableIntrinsic::ShockRes => {
            if hallucinating {
                "You feel grounded in reality."
            } else {
                "Your health currently feels amplified!"
            }
        }
        ConveyableIntrinsic::PoisonRes => "You feel healthy.",
        ConveyableIntrinsic::Teleport => {
            if hallucinating {
                "You feel diffuse."
            } else {
                "You feel very jumpy."
            }
        }
        ConveyableIntrinsic::TeleportControl => {
            if hallucinating {
                "You feel centered in your personal space."
            } else {
                "You feel in control of yourself."
            }
        }
        ConveyableIntrinsic::Telepathy => {
            if hallucinating {
                "You feel in touch with the cosmos."
            } else {
                "You feel a strange mental acuity."
            }
        }
    };
    IntrinsicGainResult {
        gained: true,
        intrinsic,
        message: msg,
    }
}

// ---------------------------------------------------------------------------
// 시체 후처리 (원본: eat.c:943-1156 cpostfx)
// ---------------------------------------------------------------------------

/// [v2.4.0] 시체 섭취 후 특수 효과
#[derive(Debug, Clone, Default)]
pub struct CorpsePostEffect {
    pub level_gain: bool,
    pub lycanthropy: Option<&'static str>,
    pub full_heal: bool,
    pub invisibility: bool,
    pub stun_turns: i32,
    pub speed_change: Option<bool>,
    pub cure_confusion: bool,
    pub polymorph: bool,
    pub mana_gain: i32,
    pub int_gain: bool,
    pub mimic_turns: i32,
    pub check_intrinsics: bool,
    pub message: String,
}

/// [v2.4.0] 시체 후처리 판정 (원본: cpostfx)
pub fn corpse_post_effect(monster_name: &str, rng: &mut NetHackRng) -> CorpsePostEffect {
    let mut e = CorpsePostEffect::default();
    let name = monster_name.to_lowercase();
    if name.contains("newt") {
        if rng.rn2(3) != 0 {
            e.mana_gain = rng.rnd(3) as i32;
            e.message = "You feel a mild buzz.".into();
        }
    } else if name.contains("wraith") {
        e.level_gain = true;
        e.message = "You feel yourself rising...".into();
    } else if name.contains("wererat") {
        e.lycanthropy = Some("wererat");
    } else if name.contains("werejackal") {
        e.lycanthropy = Some("werejackal");
    } else if name.contains("werewolf") {
        e.lycanthropy = Some("werewolf");
    } else if name.contains("nurse") {
        e.full_heal = true;
        e.check_intrinsics = true;
        e.message = "You feel fully healed!".into();
    } else if name.contains("stalker") {
        e.invisibility = true;
        e.stun_turns = 30;
        e.message = "You feel hidden!".into();
    } else if name.contains("yellow light") || name.contains("giant bat") {
        e.stun_turns = 30;
    } else if name.contains("bat") {
        e.stun_turns = 30;
    } else if name.contains("giant mimic") {
        e.mimic_turns = 50;
    } else if name.contains("large mimic") {
        e.mimic_turns = 40;
    } else if name.contains("small mimic") {
        e.mimic_turns = 20;
    } else if name.contains("quantum mechanic") {
        e.speed_change = Some(true);
        e.message = "Your velocity suddenly seems very uncertain!".into();
    } else if name.contains("lizard") {
        e.cure_confusion = true;
    } else if name.contains("chameleon") || name.contains("doppelganger") {
        e.polymorph = true;
        e.message = "You feel a change coming over you.".into();
    } else if name.contains("mind flayer") {
        if rng.rn2(2) == 0 {
            e.int_gain = true;
            e.message = "Yum! That was real brain food!".into();
        } else {
            e.check_intrinsics = true;
            e.message = "For some reason, that tasted bland.".into();
        }
    } else {
        e.check_intrinsics = true;
    }
    e
}

// ---------------------------------------------------------------------------
// 식사 전처리 (원본: eat.c:676-742 cprefx)
// ---------------------------------------------------------------------------

/// [v2.4.0] 시체 식사 전 판정 결과
#[derive(Debug, Clone)]
pub struct CorpsePreEffect {
    pub petrify: bool,
    pub instant_death: bool,
    pub slime: bool,
    pub cannibalism: bool,
    pub pet_penalty: bool,
    pub cure_stoning: bool,
    pub message: String,
}

/// [v2.4.0] 시체 섭취 전 특수 판정 (원본: cprefx)
pub fn corpse_pre_effect(
    monster_name: &str,
    is_stone_resistant: bool,
    is_petrified: bool,
    is_unchanged: bool,
    is_slime_proof: bool,
    is_cannibal_allowed: bool,
    player_race: &str,
    _rng: &mut NetHackRng,
) -> CorpsePreEffect {
    let mut e = CorpsePreEffect {
        petrify: false,
        instant_death: false,
        slime: false,
        cannibalism: false,
        pet_penalty: false,
        cure_stoning: false,
        message: String::new(),
    };
    let name = monster_name.to_lowercase();
    // 석화 시체 (코카트리스/메두사)
    if (name.contains("cockatrice") || name.contains("chickatrice") || name.contains("medusa"))
        && !is_stone_resistant
    {
        e.petrify = true;
        e.message = format!("You turn to stone! (tasting {} meat)", monster_name);
        return e;
    }
    // 라이더 즉사
    if name.contains("death") || name.contains("pestilence") || name.contains("famine") {
        e.instant_death = true;
        e.message = "Eating that is instantly fatal.".into();
        return e;
    }
    // 개/고양이 도덕 패널티
    if (name.contains("dog")
        || name.contains("kitten")
        || name.contains("housecat")
        || name.contains("large cat"))
        && !is_cannibal_allowed
    {
        e.pet_penalty = true;
        e.message = format!("You feel that eating the {} was a bad idea.", monster_name);
    }
    // 도마뱀/산성 → 석화 해소
    if (name.contains("lizard") || name.contains("acid")) && is_petrified {
        e.cure_stoning = true;
        e.message = "You feel limber!".into();
    }
    // 슬라임 감염
    if name.contains("green slime") && !is_unchanged && !is_slime_proof {
        e.slime = true;
        e.message = "You don't feel very well.".into();
    }
    // 식인
    if !is_cannibal_allowed && name.contains(player_race) {
        e.cannibalism = true;
        e.message = "You cannibal! You will regret this!".into();
    }
    e
}

// ---------------------------------------------------------------------------
// 식용 가능 판정 (원본: eat.c:82-111 is_edible)
// ---------------------------------------------------------------------------

/// [v2.4.0] 폴리모프 상태별 식용 가능 판정 (원본: is_edible)
pub fn is_edible_check(
    item_class: &str,
    is_unique: bool,
    is_metallic: bool,
    is_organic: bool,
    is_rustprone: bool,
    has_contents: bool,
    eater_metallivorous: bool,
    eater_rust_monster: bool,
    eater_ghoul: bool,
    eater_gelcube: bool,
    is_corpse: bool,
    is_egg: bool,
    is_vegan: bool,
) -> bool {
    if is_unique {
        return false;
    }
    if eater_metallivorous && is_metallic && (!eater_rust_monster || is_rustprone) {
        return true;
    }
    if eater_ghoul {
        return (is_corpse && !is_vegan) || is_egg;
    }
    if eater_gelcube && is_organic && !has_contents {
        return true;
    }
    item_class == "FOOD"
}

// ---------------------------------------------------------------------------
// 뇌 먹기 (원본: eat.c:494-638 eat_brains)
// ---------------------------------------------------------------------------

/// [v2.4.0] 뇌 먹기 결과
#[derive(Debug, Clone)]
pub struct EatBrainResult {
    pub hit: bool,
    pub attacker_died: bool,
    pub extra_damage: i32,
    pub nutrition_gain: i32,
    pub message: String,
}

/// [v2.4.0] 뇌 먹기 로직 (원본: eat_brains)
pub fn eat_brains(
    noncorporeal: bool,
    mindless: bool,
    is_rider: bool,
    target_name: &str,
    attacker_is_player: bool,
    target_is_player: bool,
    rng: &mut NetHackRng,
) -> EatBrainResult {
    let xtra = rng.rnd(10) as i32;
    if noncorporeal {
        return EatBrainResult {
            hit: false,
            attacker_died: false,
            extra_damage: 0,
            nutrition_gain: 0,
            message: format!("{}'s brain is unharmed.", target_name),
        };
    }
    if mindless && !target_is_player {
        return EatBrainResult {
            hit: false,
            attacker_died: false,
            extra_damage: 0,
            nutrition_gain: 0,
            message: format!("{} doesn't notice.", target_name),
        };
    }
    if is_rider {
        return EatBrainResult {
            hit: true,
            attacker_died: true,
            extra_damage: xtra,
            nutrition_gain: 0,
            message: "Ingesting that is fatal.".into(),
        };
    }
    let msg = if attacker_is_player {
        format!("You eat {}'s brain!", target_name)
    } else if target_is_player {
        "Your brain is eaten!".into()
    } else {
        format!("{}'s brain is eaten!", target_name)
    };
    EatBrainResult {
        hit: true,
        attacker_died: false,
        extra_damage: xtra,
        nutrition_gain: if !attacker_is_player {
            rng.rnd(60) as i32
        } else {
            0
        },
        message: msg,
    }
}

// ---------------------------------------------------------------------------
// 식인/채식 판정 (원본: eat.c:640-674, 1158-1167)
// ---------------------------------------------------------------------------

/// [v2.4.0] 식인 판정 (원본: maybe_cannibal)
pub fn maybe_cannibal(
    food_race: &str,
    player_race: &str,
    is_caveman: bool,
    is_orc: bool,
    rng: &mut NetHackRng,
) -> (bool, i32, &'static str) {
    if is_caveman || is_orc {
        return (false, 0, "");
    }
    if food_race.eq_ignore_ascii_case(player_race) {
        let penalty = -(rng.rn1(4, 2) as i32);
        return (true, penalty, "You cannibal! You will regret this!");
    }
    (false, 0, "")
}

/// [v2.4.0] 채식주의 위반 (원본: violated_vegetarian)
pub fn violated_vegetarian(is_monk: bool) -> (i32, &'static str) {
    if is_monk {
        (-1, "You feel guilty.")
    } else {
        (0, "")
    }
}

// ---------------------------------------------------------------------------
// 유틸리티 (원본: eat.c 각종 매크로/유틸)
// ---------------------------------------------------------------------------

/// [v2.4.0] 영양 보정 (원본: obj_nutrition — 종족별 렘바스/크램 보정)
pub fn obj_nutrition_adjusted(
    base: i32,
    name: &str,
    is_elf: bool,
    is_orc: bool,
    is_dwarf: bool,
) -> i32 {
    let n = name.to_lowercase();
    let mut nut = base;
    if n.contains("lembas") {
        if is_elf {
            nut += nut / 4;
        } else if is_orc {
            nut -= nut / 4;
        }
    } else if n.contains("cram") && is_dwarf {
        nut += nut / 6;
    }
    nut
}

/// [v2.4.0] 비건 판정 (원본: vegan macro)
pub fn is_vegan_monster(sym: char) -> bool {
    matches!(sym, 'F' | 'P' | ':' | 'J' | 'b')
}

/// [v2.4.0] 채식 판정 (원본: vegetarian macro)
pub fn is_vegetarian_monster(sym: char) -> bool {
    is_vegan_monster(sym) || matches!(sym, 'B' | 'j')
}

/// [v2.4.0] 비부패 시체 판정 (원본: nonrotting_corpse macro)
pub fn is_nonrotting_corpse(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("lizard")
        || n.contains("lichen")
        || n.contains("death")
        || n.contains("pestilence")
        || n.contains("famine")
}

/// [v2.4.0] 비부패 음식 판정 (원본: nonrotting_food macro)
pub fn is_nonrotting_food(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("lembas") || n.contains("cram")
}

/// [v2.4.0] 초기 영양 (원본: init_uhunger)
pub fn init_hunger() -> i32 {
    900
}

/// [v2.4.0] 과식 패널티 (원본: choke 내 기사)
pub fn gluttony_penalty(is_knight: bool, is_lawful: bool) -> (i32, &'static str) {
    if is_knight && is_lawful {
        (-1, "You feel like a glutton!")
    } else {
        (0, "")
    }
}

// ---------------------------------------------------------------------------
// [v2.4.0] 테스트
// ---------------------------------------------------------------------------
#[cfg(test)]
mod eat_v240_tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    #[test]
    fn test_tin_spinach() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            tin_variety_from_spe(1, false, &mut rng),
            TinVariety::Spinach
        );
    }
    #[test]
    fn test_tin_cursed() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(tin_variety_from_spe(0, true, &mut rng), TinVariety::Rotten);
    }
    #[test]
    fn test_tin_encoded() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            tin_variety_from_spe(-5, false, &mut rng),
            TinVariety::Pickled
        );
    }
    #[test]
    fn test_tin_table() {
        let t = tin_text_table();
        assert_eq!(t.len(), 15);
        assert!(t[0].nut < 0);
    }
    #[test]
    fn test_consume_trapped() {
        let mut rng = NetHackRng::new(42);
        let r = consume_tin(0, false, true, Some("kobold"), &mut rng);
        assert!(r.trapped);
    }
    #[test]
    fn test_consume_spinach() {
        let mut rng = NetHackRng::new(42);
        let r = consume_tin(1, false, false, Some("x"), &mut rng);
        assert!(r.message.contains("spinach"));
    }
    #[test]
    fn test_intrinsics() {
        let list = intrinsic_possible(0x01 | 0x10, false, false, true);
        assert!(list.contains(&ConveyableIntrinsic::FireRes));
        assert!(list.contains(&ConveyableIntrinsic::Telepathy));
    }
    #[test]
    fn test_post_wraith() {
        let mut rng = NetHackRng::new(42);
        assert!(corpse_post_effect("wraith", &mut rng).level_gain);
    }
    #[test]
    fn test_post_lizard() {
        let mut rng = NetHackRng::new(42);
        assert!(corpse_post_effect("lizard", &mut rng).cure_confusion);
    }
    #[test]
    fn test_pre_cockatrice() {
        let mut rng = NetHackRng::new(42);
        let e = corpse_pre_effect(
            "cockatrice",
            false,
            false,
            false,
            false,
            false,
            "human",
            &mut rng,
        );
        assert!(e.petrify);
    }
    #[test]
    fn test_pre_rider() {
        let mut rng = NetHackRng::new(42);
        let e = corpse_pre_effect(
            "Death", false, false, false, false, false, "human", &mut rng,
        );
        assert!(e.instant_death);
    }
    #[test]
    fn test_edible_unique() {
        assert!(!is_edible_check(
            "FOOD", true, false, false, false, false, false, false, false, false, false, false,
            false
        ));
    }
    #[test]
    fn test_brains_rider() {
        let mut rng = NetHackRng::new(42);
        let r = eat_brains(false, false, true, "Death", true, false, &mut rng);
        assert!(r.attacker_died);
    }
    #[test]
    fn test_cannibal() {
        let mut rng = NetHackRng::new(42);
        let (is, pen, _) = maybe_cannibal("human", "human", false, false, &mut rng);
        assert!(is);
        assert!(pen < 0);
    }
    #[test]
    fn test_nutrition_lembas() {
        assert_eq!(
            obj_nutrition_adjusted(800, "lembas wafer", true, false, false),
            1000
        );
        assert_eq!(
            obj_nutrition_adjusted(800, "lembas wafer", false, true, false),
            600
        );
    }
    #[test]
    fn test_nonrotting() {
        assert!(is_nonrotting_corpse("lizard"));
        assert!(!is_nonrotting_corpse("kobold"));
    }
}
