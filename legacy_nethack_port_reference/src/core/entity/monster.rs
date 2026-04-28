// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use bitflags::bitflags;
use legion::Entity;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct MonsterFlags1: u32 {
        const FLY          = 0x00000001;
        const SWIM         = 0x00000002;
        const AMORPHOUS    = 0x00000004;
        const WALLWALK     = 0x00000008;
        const CLING        = 0x00000010;
        const TUNNEL       = 0x00000020;
        const NEEDPICK     = 0x00000040;
        const CONCEAL      = 0x00000080;
        const HIDE         = 0x00000100;
        const AMPHIBIOUS   = 0x00000200;
        const BREATHLESS   = 0x00000400;
        const NOTAKE       = 0x00000800;
        const NOEYES       = 0x00001000;
        const NOHANDS      = 0x00002000;
        const NOLIMBS      = 0x00006000;
        const NOHEAD       = 0x00008000;
        const MINDLESS     = 0x00010000;
        const HUMANOID     = 0x00020000;
        const ANIMAL       = 0x00040000;
        const SLITHY       = 0x00080000;
        const UNSOLID      = 0x00100000;
        const THICK_HIDE   = 0x00200000;
        const OVIPAROUS    = 0x00400000;
        const REGEN        = 0x00800000;
        const SEE_INVIS    = 0x01000000;
        const TPORT        = 0x02000000;
        const TPORT_CNTRL  = 0x04000000;
        const ACID         = 0x08000000;
        const POIS         = 0x10000000;
        const CARNIVORE    = 0x20000000;
        const HERBIVORE    = 0x40000000;
        const OMNIVORE     = 0x60000000;
        const METALLIVORE  = 0x80000000;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct MonsterFlags2: u32 {
        const NOPOLY        = 0x00000001;
        const UNDEAD        = 0x00000002;
        const WERE          = 0x00000004;
        const HUMAN         = 0x00000008;
        const ELF           = 0x00000010;
        const DWARF         = 0x00000020;
        const GNOME         = 0x00000040;
        const ORC           = 0x00000080;
        const DEMON         = 0x00000100;
        const MERC          = 0x00000200;
        const LORD          = 0x00000400;
        const PRINCE        = 0x00000800;
        const MINION        = 0x00001000;
        const GIANT         = 0x00002000;
        const SHAPESHIFTER  = 0x00004000;
        const MALE          = 0x00010000;
        const FEMALE        = 0x00020000;
        const NEUTER        = 0x00040000;
        const PNAME         = 0x00080000;
        const HOSTILE       = 0x00100000;
        const PEACEFUL      = 0x00200000;
        const DOMESTIC      = 0x00400000;
        const WANDER        = 0x00800000;
        const STALK         = 0x01000000;
        const NASTY         = 0x02000000;
        const STRONG        = 0x04000000;
        const ROCKTHROW     = 0x08000000;
        const GREEDY        = 0x10000000;
        const JEWELS        = 0x20000000;
        const COLLECT       = 0x40000000;
        const MAGIC         = 0x80000000;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct MonsterFlags3: u16 {
        const WANTSAMUL    = 0x0001;
        const WANTSBELL    = 0x0002;
        const WANTSBOOK    = 0x0004;
        const WANTSCAND    = 0x0008;
        const WANTSARTI    = 0x0010;
        const WAITFORU     = 0x0040;
        const CLOSE        = 0x0080;
        const INFRAVISION  = 0x0100;
        const INFRAVISIBLE = 0x0200;
        const DISPLACES    = 0x0400;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Resistances: u16 {
        const FIRE      = 0x0001;
        const COLD      = 0x0002;
        const SLEEP     = 0x0004;
        const DISINT    = 0x0008;
        const ELEC      = 0x0010;
        const POISON    = 0x0020;
        const ACID      = 0x0040;
        const STONE     = 0x0080;
        const SEE_INVIS = 0x0100;
        const LEVITATE  = 0x0200;
        const WATERWALK = 0x0400;
        const MAGBREATH = 0x0800;
        const DISPLACED = 0x1000;
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum AttackType {
    None = 0,
    Claw = 1,
    Bite = 2,
    Kick = 3,
    Butt = 4,
    Touch = 5,
    Sting = 6,
    Hugs = 7,
    Spit = 10,
    Engulf = 11,
    Breath = 12,
    Explode = 13,
    Boom = 14,
    Gaze = 15,
    Tentacles = 16,
    Weapon = 254,
    Magic = 255,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DamageType {
    Phys = 0,
    Magm = 1,
    Fire = 2,
    Cold = 3,
    Slee = 4,
    Disn = 5,
    Elec = 6,
    Drst = 7,
    Acid = 8,
    Blnd = 11,
    Stun = 12,
    Slow = 13,
    Plys = 14,
    Drli = 15,
    Dren = 16,
    Legs = 17,
    Ston = 18,
    Stck = 19,
    Sgld = 20,
    Sitm = 21,
    Sedu = 22,
    Tlpt = 23,
    Rust = 24,
    Conf = 25,
    Dgst = 26,
    Heal = 27,
    Wrap = 28,
    Were = 29,
    Drdx = 30,
    Drco = 31,
    Drin = 32,
    Dise = 33,
    Dcay = 34,
    Halu = 36,
    Deth = 37,
    Pest = 38,
    Famn = 39,
    Slim = 40,
    Ench = 41,
    Corr = 42,
    Poly = 43,
    Open = 44,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub atype: AttackType,
    pub adtype: DamageType,
    pub dice: u8,
    pub sides: u8,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterTemplate {
    pub name: String,
    pub symbol: char,
    pub level: i8,
    pub movement: i8,
    pub ac: i8,
    pub mr: i8,
    pub alignment: i8,
    pub geno: u16,
    #[serde(default)]
    pub attacks: Vec<Attack>,
    pub weight: u16,
    pub nutrition: u16,
    pub msound: u8,
    pub msize: u8,
    pub resists: u16,
    pub conveys: u16,
    pub flags1: u32,
    pub flags2: u32,
    pub flags3: u16,
    pub difficulty: u8,
    pub color: u8,
}

impl MonsterTemplate {
    #[inline]
    pub fn has_flag1(&self, flag: MonsterFlags1) -> bool {
        MonsterFlags1::from_bits_truncate(self.flags1).contains(flag)
    }
    #[inline]
    pub fn has_flag2(&self, flag: MonsterFlags2) -> bool {
        MonsterFlags2::from_bits_truncate(self.flags2).contains(flag)
    }
    #[inline]
    pub fn has_flag3(&self, flag: MonsterFlags3) -> bool {
        MonsterFlags3::from_bits_truncate(self.flags3).contains(flag)
    }
    #[inline]
    pub fn has_resist(&self, resist: Resistances) -> bool {
        Resistances::from_bits_truncate(self.resists).contains(resist)
    }

    //
    pub fn is_flyer(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Fly)
    }
    pub fn is_swimmer(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Swim)
    }
    pub fn is_amorphous(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Amorphous)
    }
    pub fn is_wallwalker(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::WallWalk)
    }
    pub fn is_clinger(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Cling)
    }
    pub fn is_tunneler(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Tunnel)
    }
    pub fn breathless(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Breathless)
    }
    pub fn haseyes(&self) -> bool {
        !self.has_capability(super::capability::MonsterCapability::NoEyes)
    }
    pub fn humanoid(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Humanoid)
    }
    pub fn is_animal(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Animal)
    }
    pub fn slithy(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Slithy)
    }
    pub fn unsolid(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Unsolid)
    }
    pub fn mindless(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Mindless)
    }
    pub fn is_undead(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Undead)
    }
    pub fn is_demon(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Demon)
    }
    pub fn hates_silver(&self) -> bool {
        // NetHack 3.6.7: mondata.c:mon_hates_silver()
        //
        self.is_undead()
            || self.is_demon()
            || self.has_capability(super::capability::MonsterCapability::Were)
    }
    pub fn has_hands(&self) -> bool {
        !self.has_capability(super::capability::MonsterCapability::NoHands)
    }

    pub fn is_were(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Were)
    }
    pub fn is_elf(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Elf)
    }
    pub fn is_dwarf(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Dwarf)
    }
    pub fn is_gnome(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Gnome)
    }
    pub fn is_orc(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Orc)
    }
    pub fn is_giant(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Giant)
    }
    pub fn is_minion(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Minion)
    }

    pub fn is_male(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Male)
    }
    pub fn is_female(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Female)
    }
    pub fn is_neuter(&self) -> bool {
        self.has_capability(super::capability::MonsterCapability::Neuter)
    }

    pub fn resists_fire(&self) -> bool {
        self.has_resist(Resistances::FIRE)
    }
    pub fn resists_cold(&self) -> bool {
        self.has_resist(Resistances::COLD)
    }
    pub fn resists_sleep(&self) -> bool {
        self.has_resist(Resistances::SLEEP)
    }
    pub fn resists_disint(&self) -> bool {
        self.has_resist(Resistances::DISINT)
    }
    pub fn resists_elec(&self) -> bool {
        self.has_resist(Resistances::ELEC)
    }
    pub fn resists_poison(&self) -> bool {
        self.has_resist(Resistances::POISON)
    }
    pub fn resists_acid(&self) -> bool {
        self.has_resist(Resistances::ACID)
    }
    pub fn resists_ston(&self) -> bool {
        self.has_resist(Resistances::STONE)
    }

    pub fn bigmonst(&self) -> bool {
        self.msize >= 4
    } // MZ_MEDIUM=3, MZ_LARGE=4
}

// NetHack generation flags (monst.h)
pub const G_UNIQ: u16 = 0x1000; // unique monster
pub const G_NOHELL: u16 = 0x0800; // not generated in "hell"
pub const G_HELL: u16 = 0x0400; // generated only in "hell"
pub const G_NOGEN: u16 = 0x0200; // generated only specially
pub const G_SGROUP: u16 = 0x0080; // appear in small groups normally
pub const G_LGROUP: u16 = 0x0040; // appear in large groups normally
pub const G_GENO: u16 = 0x0020; // can be genocided
pub const G_NOCORPSE: u16 = 0x0010; // no corpse left ever
pub const G_FREQ: u16 = 0x0007; // creation frequency mask

///
#[derive(Clone)]
pub struct MonsterManager {
    ///
    pub templates: HashMap<String, MonsterTemplate>,
    ///
    kind_index: HashMap<crate::generated::MonsterKind, String>,
}

impl MonsterManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            kind_index: HashMap::new(),
        }
    }

    ///
    pub fn get_template(&self, name: &str) -> Option<&MonsterTemplate> {
        self.templates.get(name)
    }

    ///
    pub fn get_by_kind(&self, kind: crate::generated::MonsterKind) -> Option<&MonsterTemplate> {
        self.kind_index
            .get(&kind)
            .and_then(|name| self.templates.get(name))
    }

    ///
    pub fn build_kind_index(&mut self) {
        self.kind_index.clear();
        for (name, _template) in &self.templates {
            let kind = crate::generated::MonsterKind::from_str(name);
            //
            if kind != crate::generated::MonsterKind::Unknown {
                self.kind_index.insert(kind, name.clone());
            }
        }
    }

    pub fn load_defaults(&mut self) {
        //
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterState {
    pub mflee: bool,
    pub mfleetim: u8,
    pub msleeping: bool,
    pub mstun: bool,
    pub mconf: bool,
    pub mblinded: u16,
    pub mcanmove: bool,
    pub mstrategy: u32,
    pub mspec_used: u16,
    pub mtarget: Option<(i32, i32)>,
}

impl MonsterState {
    pub fn new() -> Self {
        Self {
            mflee: false,
            mfleetim: 0,
            msleeping: false,
            mstun: false,
            mconf: false,
            mblinded: 0,
            mcanmove: true,
            mstrategy: 0,
            mspec_used: 0,
            mtarget: None,
        }
    }
}

impl Default for MonsterState {
    fn default() -> Self {
        Self::new()
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    None,
    Orc,
    Elf,
    Dwarf,
    Gnome,
    Demon,
    Undead,
    Human,
    Soldier,
    Shopkeeper,
    Animal,
}

impl Faction {
    ///
    pub fn is_hostile_to(&self, other: Faction) -> bool {
        if *self == Faction::None || other == Faction::None {
            return false;
        }
        if *self == other {
            return false;
        }

        match (self, other) {
            (Faction::Orc, Faction::Elf) | (Faction::Elf, Faction::Orc) => true,
            (Faction::Orc, Faction::Dwarf) | (Faction::Dwarf, Faction::Orc) => true,
            (Faction::Orc, Faction::Gnome) | (Faction::Gnome, Faction::Orc) => true,
            (Faction::Demon, Faction::Human) | (Faction::Human, Faction::Demon) => true,
            (Faction::Undead, Faction::Human) | (Faction::Human, Faction::Undead) => true,
            _ => false,
        }
    }
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonsterFaction {
    pub faction: Faction,
    pub leader: Option<Entity>,
}

///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pet {
    pub owner: Entity,
    pub name: Option<String>,
    pub hunger: i32,
    pub loyalty: i32,
}
