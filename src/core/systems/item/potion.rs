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

use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionType {
    Healing,
    ExtraHealing,
    FullHealing,
    GainAbility,
    GainLevel,
    Speed,
    Invisibility,
    SeeInvisible,
    Levitation,
    Confusion,
    Hallucination,
    Blindness,
    Paralysis,
    Sleeping,
    Sickness,
    Restore,
    Polymorph,
    Booze,
    Enlightenment,
    MonsterDetect,
    ObjectDetect,
    Water,
    Oil,
    Acid,
    Fruit,
    GainEnergy,
}

///
#[derive(Debug, Clone)]
pub struct PotionResult {
    pub potion_type: PotionType,
    pub success: bool,
    pub message: String,
    pub hp_change: i32,
    pub max_hp_change: i32,
    pub duration: i32,
    pub stat_change: i32,
    pub identified: bool,
}

impl PotionResult {
    pub fn new(pt: PotionType, msg: &str) -> Self {
        Self {
            potion_type: pt,
            success: true,
            message: msg.to_string(),
            hp_change: 0,
            max_hp_change: 0,
            duration: 0,
            stat_change: 0,
            identified: true,
        }
    }
}

///
pub fn potion_name_to_type(name: &str) -> Option<PotionType> {
    let l = name.to_lowercase();
    if l.contains("full healing") {
        Some(PotionType::FullHealing)
    } else if l.contains("extra healing") {
        Some(PotionType::ExtraHealing)
    } else if l.contains("healing") {
        Some(PotionType::Healing)
    } else if l.contains("gain ability") {
        Some(PotionType::GainAbility)
    } else if l.contains("gain level") {
        Some(PotionType::GainLevel)
    } else if l.contains("gain energy") {
        Some(PotionType::GainEnergy)
    } else if l.contains("speed") {
        Some(PotionType::Speed)
    } else if l.contains("see invisible") {
        Some(PotionType::SeeInvisible)
    } else if l.contains("invisibility") {
        Some(PotionType::Invisibility)
    } else if l.contains("levitation") {
        Some(PotionType::Levitation)
    } else if l.contains("confusion") {
        Some(PotionType::Confusion)
    } else if l.contains("hallucination") {
        Some(PotionType::Hallucination)
    } else if l.contains("blindness") {
        Some(PotionType::Blindness)
    } else if l.contains("paralysis") {
        Some(PotionType::Paralysis)
    } else if l.contains("sleeping") {
        Some(PotionType::Sleeping)
    } else if l.contains("sickness") {
        Some(PotionType::Sickness)
    } else if l.contains("restore") {
        Some(PotionType::Restore)
    } else if l.contains("polymorph") {
        Some(PotionType::Polymorph)
    } else if l.contains("booze") {
        Some(PotionType::Booze)
    } else if l.contains("enlighten") {
        Some(PotionType::Enlightenment)
    } else if l.contains("monster detection") {
        Some(PotionType::MonsterDetect)
    } else if l.contains("object detection") {
        Some(PotionType::ObjectDetect)
    } else if l.contains("water") {
        Some(PotionType::Water)
    } else if l.contains("oil") {
        Some(PotionType::Oil)
    } else if l.contains("acid") {
        Some(PotionType::Acid)
    } else if l.contains("fruit") {
        Some(PotionType::Fruit)
    } else {
        None
    }
}

///
pub fn drink_potion(
    potion_name: &str,
    blessed: bool,
    cursed: bool,
    _hallucinating: bool,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> PotionResult {
    let pt = match potion_name_to_type(potion_name) {
        Some(t) => t,
        None => {
            log.add("You drink the potion. It tastes like water.", turn);
            return PotionResult::new(PotionType::Water, "Tastes like water.");
        }
    };
    match pt {
        PotionType::Healing => pot_heal(blessed, cursed, rng, log, turn),
        PotionType::ExtraHealing => pot_xheal(blessed, cursed, rng, log, turn),
        PotionType::FullHealing => pot_fheal(blessed, cursed, log, turn),
        PotionType::GainAbility => pot_gain_abil(blessed, cursed, log, turn),
        PotionType::GainLevel => pot_gain_lvl(cursed, log, turn),
        PotionType::Speed => pot_speed(blessed, cursed, rng, log, turn),
        PotionType::Invisibility => pot_invis(blessed, cursed, rng, log, turn),
        PotionType::SeeInvisible => pot_seeinvis(blessed, rng, log, turn),
        PotionType::Levitation => pot_lev(blessed, cursed, rng, log, turn),
        PotionType::Confusion => pot_conf(rng, log, turn),
        PotionType::Hallucination => pot_halluc(rng, log, turn),
        PotionType::Blindness => pot_blind(cursed, rng, log, turn),
        PotionType::Paralysis => pot_para(rng, log, turn),
        PotionType::Sleeping => pot_sleep(rng, log, turn),
        PotionType::Sickness => pot_sick(rng, log, turn),
        PotionType::Restore => pot_restore(blessed, cursed, log, turn),
        PotionType::Polymorph => {
            log.add("You feel a change coming over you.", turn);
            PotionResult::new(PotionType::Polymorph, "You feel a change.")
        }
        PotionType::Booze => pot_booze(rng, log, turn),
        PotionType::Enlightenment => {
            log.add("You feel enlightened.", turn);
            PotionResult::new(PotionType::Enlightenment, "Enlightened.")
        }
        PotionType::MonsterDetect => pot_mondet(blessed, cursed, log, turn),
        PotionType::ObjectDetect => pot_objdet(blessed, cursed, log, turn),
        PotionType::Water => pot_water(blessed, cursed, log, turn),
        PotionType::Oil => {
            log.add("This tastes like oil.", turn);
            PotionResult::new(PotionType::Oil, "Tastes like oil.")
        }
        PotionType::Acid => pot_acid(rng, log, turn),
        PotionType::Fruit => {
            log.add("This tastes like fruit juice.", turn);
            let mut r = PotionResult::new(PotionType::Fruit, "Fruit juice.");
            r.hp_change = if blessed { 5 } else { 2 };
            r
        }
        PotionType::GainEnergy => pot_energy(blessed, cursed, rng, log, turn),
    }
}

//

fn pot_heal(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    let h = if cu {
        rng.rn2(4) + 1
    } else if bl {
        rng.rn2(8) + 8
    } else {
        rng.rn2(8) + 4
    };
    let msg = format!("You feel {}better.", if bl { "much " } else { "" });
    log.add_colored(&msg, [100, 255, 100], t);
    let mut r = PotionResult::new(PotionType::Healing, &msg);
    r.hp_change = h as i32;
    if bl {
        r.max_hp_change = 1;
    }
    r
}

fn pot_xheal(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    let h = if cu {
        rng.rn2(8) + 2
    } else if bl {
        rng.rn2(12) + 20
    } else {
        rng.rn2(12) + 12
    };
    log.add_colored("You feel much better.", [50, 255, 50], t);
    let mut r = PotionResult::new(PotionType::ExtraHealing, "Much better.");
    r.hp_change = h as i32;
    if bl {
        r.max_hp_change = 2;
    }
    r
}

fn pot_fheal(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    let msg = if cu {
        "You feel somewhat better."
    } else {
        "You feel completely healed."
    };
    log.add_colored(msg, [0, 255, 0], t);
    let mut r = PotionResult::new(PotionType::FullHealing, msg);
    r.hp_change = if cu { 50 } else { 999 };
    if bl {
        r.max_hp_change = 5;
    }
    r
}

fn pot_gain_abil(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add_colored("You feel slightly weakened.", [255, 100, 100], t);
        let mut r = PotionResult::new(PotionType::GainAbility, "Weakened.");
        r.stat_change = -1;
        r
    } else {
        let msg = if bl {
            "All attributes improve!"
        } else {
            "A surge of power!"
        };
        log.add_colored(msg, [255, 255, 100], t);
        let mut r = PotionResult::new(PotionType::GainAbility, msg);
        r.stat_change = if bl { 6 } else { 1 };
        r
    }
}

fn pot_gain_lvl(cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You rise up, through the ceiling!", t);
        PotionResult::new(PotionType::GainLevel, "Level teleport up.")
    } else {
        log.add_colored("You feel more experienced.", [255, 255, 100], t);
        let mut r = PotionResult::new(PotionType::GainLevel, "More experienced.");
        r.stat_change = 1;
        r
    }
}

fn pot_speed(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You feel yourself slowing down.", t);
        let mut r = PotionResult::new(PotionType::Speed, "Slowed.");
        r.duration = -(rng.rn2(50) + 50) as i32;
        r
    } else {
        log.add_colored("You feel yourself speeding up.", [100, 200, 255], t);
        let mut r = PotionResult::new(PotionType::Speed, "Sped up.");
        r.duration = if bl {
            (rng.rn2(100) + 160) as i32
        } else {
            (rng.rn2(50) + 100) as i32
        };
        r
    }
}

fn pot_invis(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You feel transparent for a moment.", t);
        PotionResult::new(PotionType::Invisibility, "Momentarily transparent.")
    } else {
        log.add_colored("You feel quite transparent.", [200, 200, 255], t);
        let mut r = PotionResult::new(PotionType::Invisibility, "Transparent.");
        r.duration = if bl {
            (rng.rn2(200) + 200) as i32
        } else {
            (rng.rn2(100) + 100) as i32
        };
        r
    }
}

fn pot_seeinvis(bl: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("You feel your vision sharpen.", [255, 255, 200], t);
    let mut r = PotionResult::new(PotionType::SeeInvisible, "Vision sharpened.");
    r.duration = if bl { 0 } else { (rng.rn2(100) + 100) as i32 };
    r
}

fn pot_lev(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("You float up.", [200, 200, 255], t);
    let mut r = PotionResult::new(PotionType::Levitation, "Float up.");
    r.duration = if cu {
        (rng.rn2(200) + 200) as i32
    } else if bl {
        (rng.rn2(20) + 20) as i32
    } else {
        (rng.rn2(100) + 100) as i32
    };
    r
}

fn pot_conf(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("What a trippy feeling!", [255, 100, 255], t);
    let mut r = PotionResult::new(PotionType::Confusion, "Trippy!");
    r.duration = (rng.rn2(50) + 20) as i32;
    r
}

fn pot_halluc(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("Oh wow! Everything looks cosmic!", [255, 50, 200], t);
    let mut r = PotionResult::new(PotionType::Hallucination, "Cosmic!");
    r.duration = (rng.rn2(200) + 200) as i32;
    r
}

fn pot_blind(cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("A cloud of darkness falls upon you.", [50, 50, 50], t);
    let mut r = PotionResult::new(PotionType::Blindness, "Blinded.");
    r.duration = if cu {
        (rng.rn2(200) + 200) as i32
    } else {
        (rng.rn2(100) + 100) as i32
    };
    r
}

fn pot_para(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("You are frozen in place!", [100, 100, 255], t);
    let mut r = PotionResult::new(PotionType::Paralysis, "Frozen!");
    r.duration = (rng.rn2(13) + 2) as i32;
    r
}

fn pot_sleep(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("You feel very sleepy...", [150, 150, 200], t);
    let mut r = PotionResult::new(PotionType::Sleeping, "Sleepy.");
    r.duration = (rng.rn2(13) + 7) as i32;
    r
}

fn pot_sick(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("You feel deathly sick!", [100, 200, 50], t);
    let mut r = PotionResult::new(PotionType::Sickness, "Sick!");
    r.hp_change = -(rng.rn2(10) + 5) as i32;
    r.duration = (rng.rn2(20) + 10) as i32;
    r
}

fn pot_restore(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You feel drained.", t);
        PotionResult::new(PotionType::Restore, "Drained.")
    } else {
        let msg = if bl {
            "Abilities fully restored!"
        } else {
            "Abilities restored."
        };
        log.add_colored(msg, [100, 200, 255], t);
        let mut r = PotionResult::new(PotionType::Restore, msg);
        r.stat_change = if bl { 6 } else { 1 };
        r
    }
}

fn pot_booze(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    log.add_colored("Ooph! Liquid fire!", [255, 200, 100], t);
    let mut r = PotionResult::new(PotionType::Booze, "Booze!");
    r.hp_change = 1;
    r.duration = (rng.rn2(25) + 15) as i32;
    r
}

fn pot_mondet(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You feel threatened.", t);
        PotionResult::new(PotionType::MonsterDetect, "Threatened.")
    } else {
        let msg = if bl {
            "Sense all monsters."
        } else {
            "Sense monsters."
        };
        log.add_colored(msg, [200, 200, 255], t);
        let mut r = PotionResult::new(PotionType::MonsterDetect, msg);
        r.duration = if bl { 300 } else { 100 };
        r
    }
}

fn pot_objdet(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add("You feel a pull downward.", t);
        PotionResult::new(PotionType::ObjectDetect, "Pull.")
    } else {
        let msg = if bl {
            "Sense all objects."
        } else {
            "Sense objects."
        };
        log.add_colored(msg, [200, 255, 200], t);
        PotionResult::new(PotionType::ObjectDetect, msg)
    }
}

fn pot_water(bl: bool, cu: bool, log: &mut GameLog, t: u64) -> PotionResult {
    if bl {
        log.add_colored("Full of awe. (Holy water)", [200, 200, 255], t);
        PotionResult::new(PotionType::Water, "Holy water.")
    } else if cu {
        log.add_colored("Dark presence. (Unholy water)", [100, 50, 50], t);
        PotionResult::new(PotionType::Water, "Unholy water.")
    } else {
        log.add("This tastes like water.", t);
        PotionResult::new(PotionType::Water, "Water.")
    }
}

fn pot_acid(rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    let d = rng.rn2(10) + 5;
    let msg = format!("The acid burns! ({} damage)", d);
    log.add_colored(&msg, [255, 200, 0], t);
    let mut r = PotionResult::new(PotionType::Acid, &msg);
    r.hp_change = -(d as i32);
    r
}

fn pot_energy(bl: bool, cu: bool, rng: &mut NetHackRng, log: &mut GameLog, t: u64) -> PotionResult {
    if cu {
        log.add_colored("Magical energy drains!", [200, 100, 200], t);
        let mut r = PotionResult::new(PotionType::GainEnergy, "Drained.");
        r.stat_change = -(rng.rn2(10) + 5) as i32;
        r
    } else {
        let g = if bl {
            rng.rn2(20) + 15
        } else {
            rng.rn2(10) + 5
        };
        let msg = format!("Surge of magical energy! (+{} Pw)", g);
        log.add_colored(&msg, [100, 100, 255], t);
        let mut r = PotionResult::new(PotionType::GainEnergy, &msg);
        r.stat_change = g as i32;
        r
    }
}

///
pub fn potion_splash_effect(pt: PotionType) -> &'static str {
    match pt {
        PotionType::Healing | PotionType::ExtraHealing | PotionType::FullHealing => {
            "Healing vapors."
        }
        PotionType::Confusion => "Cloud of confusion.",
        PotionType::Hallucination => "Colorful sparks!",
        PotionType::Blindness => "Cloud of darkness.",
        PotionType::Sleeping => "Mysterious mist.",
        PotionType::Paralysis => "Numbing mist.",
        PotionType::Acid => "Acid splashes!",
        _ => "The potion shatters.",
    }
}

///
pub fn dip_result(pt: PotionType, item: &str, bl: bool) -> (bool, String) {
    match pt {
        PotionType::Water => {
            if bl {
                (true, format!("{} glows brightly (holy water dip)!", item))
            } else {
                (true, format!("{} seems cleansed.", item))
            }
        }
        PotionType::Invisibility => (true, format!("{} disappears!", item)),
        PotionType::Polymorph => (true, format!("{} changes shape!", item)),
        PotionType::Acid => (true, format!("Acid damages {}!", item)),
        _ => (false, format!("Nothing happens to {}.", item)),
    }
}

///
///
pub fn potionhit_damage(
    pt: PotionType,
    target_is_undead: bool,
    target_resists_fire: bool,
    rng: &mut NetHackRng,
) -> (i32, &'static str) {
    match pt {
        PotionType::Acid => {
            let dmg = (rng.rn2(10) + 5) as i32;
            (dmg, "The acid burns!")
        }
        PotionType::Healing | PotionType::ExtraHealing | PotionType::FullHealing => {
            if target_is_undead {
                //
                let dmg = match pt {
                    PotionType::FullHealing => 40,
                    PotionType::ExtraHealing => 20,
                    _ => 10,
                };
                (dmg, "The healing potion sears the undead!")
            } else {
                (0, "Healing vapors comfort the target.")
            }
        }
        PotionType::Oil => {
            //
            if !target_resists_fire {
                let dmg = (rng.rn2(6) + 3) as i32;
                (dmg, "Flaming oil splashes!")
            } else {
                (0, "Oil sizzles harmlessly.")
            }
        }
        PotionType::Sickness => (0, "The target looks sick."),
        PotionType::Confusion => (0, "The target looks confused."),
        PotionType::Blindness => (0, "The target is blinded."),
        PotionType::Sleeping => (0, "The target falls asleep."),
        PotionType::Paralysis => (0, "The target is paralyzed."),
        PotionType::Hallucination => (0, "The target looks disoriented."),
        _ => (0, "The potion shatters."),
    }
}

///
///
///
pub fn potionbreathe(pt: PotionType, rng: &mut NetHackRng) -> (bool, i32, &'static str) {
    match pt {
        PotionType::Confusion => {
            let dur = (rng.rn2(5) + 3) as i32;
            (true, dur, "A cloud of confusion gas surrounds you!")
        }
        PotionType::Hallucination => {
            let dur = (rng.rn2(20) + 10) as i32;
            (true, dur, "Strange fumes fill the air!")
        }
        PotionType::Sleeping => {
            let dur = (rng.rn2(5) + 2) as i32;
            (true, dur, "You feel drowsy from the fumes.")
        }
        PotionType::Blindness => {
            let dur = (rng.rn2(10) + 5) as i32;
            (true, dur, "Stinging vapors cloud your vision!")
        }
        PotionType::Sickness => {
            let dur = (rng.rn2(10) + 5) as i32;
            (true, dur, "Noxious fumes fill the air!")
        }
        PotionType::Acid => (true, 0, "Acidic vapors sting your eyes!"),
        PotionType::Healing | PotionType::ExtraHealing | PotionType::FullHealing => {
            (true, 0, "Healing vapors fill the air.")
        }
        PotionType::Speed => {
            let dur = (rng.rn2(10) + 5) as i32;
            (true, dur, "You feel invigorated!")
        }
        _ => (false, 0, "You smell something unusual."),
    }
}

///
///
///
///
///
///
///
///
///
pub fn mixtype(pot1: PotionType, pot2: PotionType) -> Option<PotionType> {
    //
    let pair = (pot1.min_id(), pot2.min_id(), pot1.max_id(), pot2.max_id());
    let (a, b) = if pair.0 <= pair.2 {
        (pot1, pot2)
    } else {
        (pot2, pot1)
    };

    match (a, b) {
        //
        (PotionType::Healing, PotionType::Speed) => Some(PotionType::ExtraHealing),
        (PotionType::ExtraHealing, PotionType::Speed) => Some(PotionType::FullHealing),
        //
        (PotionType::Healing, PotionType::Sickness) => Some(PotionType::Water),
        //
        (PotionType::GainLevel, PotionType::GainAbility) => Some(PotionType::GainLevel),
        //
        (PotionType::Blindness, PotionType::Enlightenment) => Some(PotionType::SeeInvisible),
        //
        (PotionType::Acid, _) | (_, PotionType::Acid) => None,
        //
        _ => None,
    }
}

impl PotionType {
    ///
    fn min_id(self) -> u8 {
        self as u8
    }
    ///
    fn max_id(self) -> u8 {
        self as u8
    }
}

///
///
///
pub fn ghost_from_bottle(appearance: &str, rng: &mut NetHackRng) -> (bool, &'static str) {
    if appearance == "smoky" && rng.rn2(13) == 0 {
        (true, "You unleash a ghost from the bottle!")
    } else if appearance == "milky" && rng.rn2(13) == 0 {
        (true, "A ghost materializes from the milky potion!")
    } else {
        (false, "")
    }
}

///
pub fn potion_appearances() -> Vec<&'static str> {
    vec![
        "ruby",
        "pink",
        "orange",
        "yellow",
        "emerald",
        "dark green",
        "cyan",
        "sky blue",
        "brilliant blue",
        "magenta",
        "purple-red",
        "puce",
        "milky",
        "swirly",
        "bubbly",
        "smoky",
        "cloudy",
        "effervescent",
        "black",
        "golden",
        "brown",
        "fizzy",
        "dark",
        "white",
        "murky",
        "clear",
        "viscous",
        "soapy",
        "sparkling",
        "luminescent",
        "muddy",
        "icy",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_potion_mapping() {
        assert_eq!(
            potion_name_to_type("potion of healing"),
            Some(PotionType::Healing)
        );
        assert_eq!(
            potion_name_to_type("potion of full healing"),
            Some(PotionType::FullHealing)
        );
        assert_eq!(potion_name_to_type("unknown"), None);
    }
    #[test]
    fn test_healing() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let r = pot_heal(false, false, &mut rng, &mut log, 1);
        assert!(r.hp_change > 0);
    }
    #[test]
    fn test_blessed_healing() {
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let r = pot_heal(true, false, &mut rng, &mut log, 1);
        assert!(r.hp_change > 0);
        assert_eq!(r.max_hp_change, 1);
    }
}

// =============================================================================
// [v2.3.5
// =============================================================================

///
pub fn potion_rarity(potion: PotionType) -> i32 {
    match potion {
        PotionType::Healing => 57,
        PotionType::ExtraHealing => 47,
        PotionType::FullHealing => 10,
        PotionType::GainAbility => 42,
        PotionType::GainEnergy => 42,
        PotionType::GainLevel => 20,
        PotionType::Speed => 42,
        PotionType::Invisibility => 40,
        PotionType::SeeInvisible => 42,
        PotionType::Blindness => 40,
        PotionType::Confusion => 42,
        PotionType::Hallucination => 40,
        PotionType::Sleeping => 42,
        PotionType::Paralysis => 42,
        PotionType::Sickness => 42,
        PotionType::Acid => 10,
        PotionType::Polymorph => 10,
        PotionType::Levitation => 42,
        PotionType::Restore => 45,
        PotionType::ObjectDetect => 42,
        PotionType::MonsterDetect => 42,
        PotionType::Water => 92,
        PotionType::Booze => 42,
        PotionType::Fruit => 42,
        PotionType::Oil => 30,
        PotionType::Enlightenment => 15,
    }
}

///
pub fn potion_shop_value(potion: PotionType) -> u32 {
    match potion {
        PotionType::Healing => 100,
        PotionType::ExtraHealing => 200,
        PotionType::FullHealing => 400,
        PotionType::GainAbility => 300,
        PotionType::GainEnergy => 150,
        PotionType::GainLevel => 300,
        PotionType::Speed => 200,
        PotionType::Invisibility => 150,
        PotionType::SeeInvisible => 50,
        PotionType::Paralysis => 300,
        PotionType::Polymorph => 200,
        PotionType::Levitation => 200,
        PotionType::Restore => 350,
        PotionType::Water => 0,
        _ => 50,
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixResult {
    NoEffect,
    Healing,
    FullHealing,
    GainAbility,
    Explosion,
    Acid,
    HolyWater,
    UnholyWater,
    Cancellation,
}

///
pub fn mix_potions(base: PotionType, additive: PotionType) -> MixResult {
    match (base, additive) {
        (PotionType::Healing, PotionType::GainEnergy) => MixResult::Healing,
        (PotionType::ExtraHealing, PotionType::GainEnergy) => MixResult::FullHealing,
        (PotionType::Healing, PotionType::Speed) => MixResult::Healing,
        (PotionType::Acid, PotionType::Sickness) => MixResult::Explosion,
        (PotionType::Water, PotionType::Healing) => MixResult::Healing,
        _ => MixResult::NoEffect,
    }
}

///
pub fn potion_splash_radius(potion: PotionType) -> i32 {
    match potion {
        PotionType::Acid | PotionType::Sickness => 2,
        PotionType::Confusion | PotionType::Hallucination => 3,
        PotionType::Sleeping | PotionType::Paralysis => 2,
        PotionType::Oil => 1,
        _ => 1,
    }
}

///
pub fn vapor_effect_message(potion: PotionType) -> &'static str {
    match potion {
        PotionType::Confusion => "A cloud of confusion gas engulfs you!",
        PotionType::Hallucination => "You inhale magical vapors!",
        PotionType::Sleeping => "You smell something soporific...",
        PotionType::Acid => "Acid vapor burns your skin!",
        PotionType::Sickness => "You inhale a poisonous cloud!",
        PotionType::Blindness => "Vapor stings your eyes!",
        PotionType::Healing => "You feel slightly better.",
        PotionType::Speed => "You feel quick for a moment.",
        _ => "",
    }
}

///
pub fn potion_buc_multiplier(blessed: bool, cursed: bool) -> f32 {
    if blessed {
        1.5
    } else if cursed {
        0.5
    } else {
        1.0
    }
}

///
pub fn dip_weapon_effect(potion: PotionType) -> &'static str {
    match potion {
        PotionType::Sickness => "Your weapon is coated with poison!",
        PotionType::Acid => "Your weapon sizzles with acid!",
        PotionType::Oil => "Your weapon gleams with oil.",
        PotionType::Healing => "Your weapon glows softly for a moment.",
        PotionType::Water => "Your weapon is wet.",
        _ => "Nothing seems to happen.",
    }
}

///
pub fn potion_identify_hint(potion: PotionType) -> &'static str {
    match potion {
        PotionType::Healing => "You feel better.",
        PotionType::ExtraHealing => "You feel much better.",
        PotionType::FullHealing => "You feel completely healed!",
        PotionType::Speed => "You feel yourself speed up.",
        PotionType::Confusion => "Huh?",
        PotionType::Paralysis => "You can't move!",
        PotionType::Blindness => "Everything goes dark!",
        PotionType::Sleeping => "You fall asleep.",
        PotionType::Acid => "This burns!",
        PotionType::Levitation => "You float up!",
        PotionType::Invisibility => "You feel transparent.",
        PotionType::Hallucination => "Oh wow, everything looks cosmic!",
        PotionType::GainAbility => "You feel empowered!",
        PotionType::GainLevel => "You feel more experienced!",
        _ => "You have a peculiar feeling.",
    }
}

///
#[derive(Debug, Clone, Default)]
pub struct PotionStatistics {
    pub potions_quaffed: u32,
    pub potions_thrown: u32,
    pub potions_mixed: u32,
    pub potions_dipped: u32,
    pub healing_total: i32,
    pub explosions: u32,
}

impl PotionStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_quaff(&mut self) {
        self.potions_quaffed += 1;
    }
    pub fn record_throw(&mut self) {
        self.potions_thrown += 1;
    }
    pub fn record_mix(&mut self) {
        self.potions_mixed += 1;
    }
}

#[cfg(test)]
mod potion_extended_tests {
    use super::*;

    #[test]
    fn test_potion_rarity() {
        assert!(potion_rarity(PotionType::Water) > potion_rarity(PotionType::FullHealing));
    }

    #[test]
    fn test_potion_value() {
        assert!(
            potion_shop_value(PotionType::FullHealing) > potion_shop_value(PotionType::Healing)
        );
    }

    #[test]
    fn test_splash_radius() {
        assert!(
            potion_splash_radius(PotionType::Confusion) > potion_splash_radius(PotionType::Healing)
        );
    }

    #[test]
    fn test_vapor_msg() {
        assert!(vapor_effect_message(PotionType::Confusion).contains("confusion"));
    }

    #[test]
    fn test_buc() {
        assert!(potion_buc_multiplier(true, false) > 1.0);
        assert!(potion_buc_multiplier(false, true) < 1.0);
    }

    #[test]
    fn test_dip() {
        assert!(dip_weapon_effect(PotionType::Sickness).contains("poison"));
    }

    #[test]
    fn test_identify_hint() {
        assert!(potion_identify_hint(PotionType::Speed).contains("speed"));
    }

    #[test]
    fn test_potion_stats() {
        let mut s = PotionStatistics::new();
        s.record_quaff();
        s.record_throw();
        assert_eq!(s.potions_quaffed, 1);
        assert_eq!(s.potions_thrown, 1);
    }
}
