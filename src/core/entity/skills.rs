use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WeaponSkill {
    None = 0,
    Dagger = 1,
    Knife = 2,
    Axe = 3,
    PickAxe = 4,
    ShortSword = 5,
    BroadSword = 6,
    LongSword = 7,
    TwoHandedSword = 8,
    Scimitar = 9,
    Saber = 10,
    Club = 11,
    Mace = 12,
    MorningStar = 13,
    Flail = 14,
    Hammer = 15,
    Quarterstaff = 16,
    Polearms = 17,
    Spear = 18,
    Trident = 19,
    Lance = 20,
    Bow = 21,
    Sling = 22,
    Crossbow = 23,
    Dart = 24,
    Shuriken = 25,
    Boomerang = 26,
    Whip = 27,
    UnicornHorn = 28,
    AttackSpell = 29,
    HealingSpell = 30,
    DivinationSpell = 31,
    EnchantmentSpell = 32,
    ClericSpell = 33,
    EscapeSpell = 34,
    MatterSpell = 35,
    BareHanded = 36,
    TwoWeapon = 37,
    Riding = 38,
    Swimming = 39,
}

impl TryFrom<u8> for WeaponSkill {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(WeaponSkill::None),
            1 => Ok(WeaponSkill::Dagger),
            2 => Ok(WeaponSkill::Knife),
            3 => Ok(WeaponSkill::Axe),
            4 => Ok(WeaponSkill::PickAxe),
            5 => Ok(WeaponSkill::ShortSword),
            6 => Ok(WeaponSkill::BroadSword),
            7 => Ok(WeaponSkill::LongSword),
            8 => Ok(WeaponSkill::TwoHandedSword),
            9 => Ok(WeaponSkill::Scimitar),
            10 => Ok(WeaponSkill::Saber),
            11 => Ok(WeaponSkill::Club),
            12 => Ok(WeaponSkill::Mace),
            13 => Ok(WeaponSkill::MorningStar),
            14 => Ok(WeaponSkill::Flail),
            15 => Ok(WeaponSkill::Hammer),
            16 => Ok(WeaponSkill::Quarterstaff),
            17 => Ok(WeaponSkill::Polearms),
            18 => Ok(WeaponSkill::Spear),
            19 => Ok(WeaponSkill::Trident),
            20 => Ok(WeaponSkill::Lance),
            21 => Ok(WeaponSkill::Bow),
            22 => Ok(WeaponSkill::Sling),
            23 => Ok(WeaponSkill::Crossbow),
            24 => Ok(WeaponSkill::Dart),
            25 => Ok(WeaponSkill::Shuriken),
            26 => Ok(WeaponSkill::Boomerang),
            27 => Ok(WeaponSkill::Whip),
            28 => Ok(WeaponSkill::UnicornHorn),
            29 => Ok(WeaponSkill::AttackSpell),
            30 => Ok(WeaponSkill::HealingSpell),
            31 => Ok(WeaponSkill::DivinationSpell),
            32 => Ok(WeaponSkill::EnchantmentSpell),
            33 => Ok(WeaponSkill::ClericSpell),
            34 => Ok(WeaponSkill::EscapeSpell),
            35 => Ok(WeaponSkill::MatterSpell),
            36 => Ok(WeaponSkill::BareHanded),
            37 => Ok(WeaponSkill::TwoWeapon),
            38 => Ok(WeaponSkill::Riding),
            39 => Ok(WeaponSkill::Swimming),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum SkillLevel {
    Restricted = 0,
    Unskilled = 1,
    Basic = 2,
    Skilled = 3,
    Expert = 4,
    Master = 5,
    GrandMaster = 6,
}

impl TryFrom<u8> for SkillLevel {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(SkillLevel::Restricted),
            1 => Ok(SkillLevel::Unskilled),
            2 => Ok(SkillLevel::Basic),
            3 => Ok(SkillLevel::Skilled),
            4 => Ok(SkillLevel::Expert),
            5 => Ok(SkillLevel::Master),
            6 => Ok(SkillLevel::GrandMaster),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub level: SkillLevel,
    pub max_level: SkillLevel,
    pub advance: u16,
}

impl SkillRecord {
    pub fn new(level: SkillLevel, max_level: SkillLevel) -> Self {
        Self {
            level,
            max_level,
            advance: 0,
        }
    }

    pub fn can_advance(&self) -> bool {
        if self.level >= self.max_level {
            return false;
        }
        let cost = self.practice_needed();
        // If cost is 0 (Restricted), we can't advance
        if cost == 0 {
            return false;
        }
        self.advance >= cost
    }

    pub fn practice_needed(&self) -> u16 {
        let text_lvl = self.level as u16;
        if text_lvl == 0 {
            return 0;
        } // Restricted
        text_lvl * text_lvl * 20
    }

    pub fn advance_level(&mut self) -> bool {
        if !self.can_advance() {
            return false;
        }

        self.level = match self.level {
            SkillLevel::Unskilled => SkillLevel::Basic,
            SkillLevel::Basic => SkillLevel::Skilled,
            SkillLevel::Skilled => SkillLevel::Expert,
            SkillLevel::Expert => SkillLevel::Master,
            SkillLevel::Master => SkillLevel::GrandMaster,
            _ => self.level,
        };
        true
    }
}
