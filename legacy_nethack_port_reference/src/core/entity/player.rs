// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use super::skills::{SkillLevel, SkillRecord, WeaponSkill};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Orc,
    Gnome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
    Starved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerClass {
    Valkyrie,
    Wizard,
    Barbarian,
    Healer,
    Knight,
    Rogue,
    Samurai,
    Tourist,
    Monk,
    Priest,
    Ranger,
    Archeologist,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attribute {
    pub base: i32,
    pub max: i32,
}

impl Attribute {
    pub fn new(val: i32) -> Self {
        Self {
            base: val,
            max: val,
        }
    }
}

///
#[derive(Clone, Debug)]
pub struct Player {
    pub x: usize,
    pub y: usize,
    pub level: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub str: Attribute,
    pub dex: Attribute,
    pub con: Attribute,
    pub int: Attribute,
    pub wis: Attribute,
    pub cha: Attribute,
    pub alignment: Alignment,
    pub alignment_record: i32, // Alignment record (u.ualign.record)
    pub experience: u64,
    pub exp_level: i32,
    pub gold: u64,
    pub hunger: HungerState,
    pub ac: i32,
    pub piety: i32, // Piety (favor with god)
    pub role: PlayerClass,
    pub race: Race,
    pub luck: i32,
    pub luck_bonus: i32,               // Extra luck (u.uluck) vs Base luck
    pub luck_turns: i32,               // Luck timeout (u.uluckcnt)
    pub prayer_cooldown: i32,          // Prayer cooldown (u.ublesscnt)
    pub nutrition: i32,                // Hunger nutrition (u.uhunger)
    pub attribute_recovery_turns: i32, // Attribute recovery timer (new)
    pub exercise: [i32; 6], // Strength, Intelligence, Wisdom, Dexterity, Constitution, Charisma
    pub two_weapon: bool,   // #twoweapon active state
    pub skills: HashMap<WeaponSkill, SkillRecord>,
    pub equip_hunger_bonus: i32,
    /// [v2.22.0 R34-P2-1] 상태이상 번들
    pub status_bundle: super::status::StatusBundle,
}

#[derive(Serialize, Deserialize)]
struct PlayerDef {
    x: usize,
    y: usize,
    level: i32,
    hp: i32,
    hp_max: i32,
    energy: i32,
    energy_max: i32,
    str: Attribute,
    dex: Attribute,
    con: Attribute,
    int: Attribute,
    wis: Attribute,
    cha: Attribute,
    alignment: Alignment,
    alignment_record: i32,
    experience: u64,
    exp_level: i32,
    gold: u64,
    hunger: HungerState,
    ac: i32,
    piety: i32,
    role: PlayerClass,
    race: Race,
    luck: i32,
    luck_bonus: i32,
    luck_turns: i32,
    prayer_cooldown: i32,
    nutrition: i32,
    attribute_recovery_turns: i32,
    exercise: [i32; 6],
    two_weapon: bool,
    skills: Vec<(WeaponSkill, SkillRecord)>,
    equip_hunger_bonus: i32,
    status_bundle: super::status::StatusBundle,
}

impl serde::Serialize for Player {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let def = PlayerDef {
            x: self.x,
            y: self.y,
            level: self.level,
            hp: self.hp,
            hp_max: self.hp_max,
            energy: self.energy,
            energy_max: self.energy_max,
            str: self.str,
            dex: self.dex,
            con: self.con,
            int: self.int,
            wis: self.wis,
            cha: self.cha,
            alignment: self.alignment,
            alignment_record: self.alignment_record,
            experience: self.experience,
            exp_level: self.exp_level,
            gold: self.gold,
            hunger: self.hunger,
            ac: self.ac,
            piety: self.piety,
            role: self.role,
            race: self.race,
            luck: self.luck,
            luck_bonus: self.luck_bonus,
            luck_turns: self.luck_turns,
            prayer_cooldown: self.prayer_cooldown,
            nutrition: self.nutrition,
            attribute_recovery_turns: self.attribute_recovery_turns,
            exercise: self.exercise,
            two_weapon: self.two_weapon,
            skills: self.skills.clone().into_iter().collect(),
            equip_hunger_bonus: self.equip_hunger_bonus,
            status_bundle: self.status_bundle.clone(),
        };
        def.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Player {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = PlayerDef::deserialize(deserializer)?;
        Ok(Player {
            x: def.x,
            y: def.y,
            level: def.level,
            hp: def.hp,
            hp_max: def.hp_max,
            energy: def.energy,
            energy_max: def.energy_max,
            str: def.str,
            dex: def.dex,
            con: def.con,
            int: def.int,
            wis: def.wis,
            cha: def.cha,
            alignment: def.alignment,
            alignment_record: def.alignment_record,
            experience: def.experience,
            exp_level: def.exp_level,
            gold: def.gold,
            hunger: def.hunger,
            ac: def.ac,
            piety: def.piety,
            role: def.role,
            race: def.race,
            luck: def.luck,
            luck_bonus: def.luck_bonus,
            luck_turns: def.luck_turns,
            prayer_cooldown: def.prayer_cooldown,
            nutrition: def.nutrition,
            attribute_recovery_turns: def.attribute_recovery_turns,
            exercise: def.exercise,
            two_weapon: def.two_weapon,
            skills: def.skills.into_iter().collect(),
            equip_hunger_bonus: def.equip_hunger_bonus,
            status_bundle: def.status_bundle,
        })
    }
}

impl Player {
    pub fn new() -> Self {
        Self {
            // ...
            x: 0,
            y: 0,
            level: 1,
            hp: 15,
            hp_max: 15,
            energy: 10,
            energy_max: 10,
            str: Attribute::new(16),
            dex: Attribute::new(14),
            con: Attribute::new(15),
            int: Attribute::new(14),
            wis: Attribute::new(14),
            cha: Attribute::new(10),
            alignment: Alignment::Neutral,
            alignment_record: 10,
            experience: 0,
            exp_level: 1,
            gold: 0,
            hunger: HungerState::NotHungry,
            ac: 10,
            piety: 0,
            role: PlayerClass::Valkyrie,
            race: Race::Human,
            luck: 0,
            luck_bonus: 0,
            luck_turns: 600,
            prayer_cooldown: 0,
            nutrition: 900, // NotHungry starts around 900
            attribute_recovery_turns: 1500,
            exercise: [0; 6],
            two_weapon: false,
            equip_hunger_bonus: 0,
            skills: {
                let mut map = HashMap::new();
                for i in 1..=39 {
                    if let Ok(skill) = WeaponSkill::try_from(i) {
                        map.insert(
                            skill,
                            SkillRecord::new(SkillLevel::Unskilled, SkillLevel::Expert),
                        );
                    }
                }
                // BareHanded starts at Basic
                map.insert(
                    WeaponSkill::BareHanded,
                    SkillRecord::new(SkillLevel::Basic, SkillLevel::Expert),
                );
                map
            },
            status_bundle: super::status::StatusBundle::new(),
        }
    }

    pub fn effective_luck(&self) -> i32 {
        (self.luck + self.luck_bonus).clamp(-13, 13)
    }

    //

    ///
    pub fn as_combat_view(&self) -> super::player_view::PlayerCombatView {
        super::player_view::PlayerCombatView::from_player(self)
    }

    ///
    pub fn as_survival_view(&self) -> super::player_view::PlayerSurvivalView {
        super::player_view::PlayerSurvivalView::from_player(self)
    }

    ///
    pub fn as_progress_view(&self) -> super::player_view::PlayerProgressView {
        super::player_view::PlayerProgressView::from_player(self)
    }

    ///
    pub fn as_attribute_view(&self) -> super::player_view::PlayerAttributeView {
        super::player_view::PlayerAttributeView::from_player(self)
    }
}
