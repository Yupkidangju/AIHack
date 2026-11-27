use serde::{Deserialize, Serialize};

use crate::domain::{combat::AttackProfile, status::Resistances};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterAiKind {
    Wander,
    ChaseVisiblePlayer,
    Stationary,
    RangedOnSight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterPassive {
    ParalyzeOnMelee,
    PoisonOnMelee,
    Regenerate,
    FireOnMelee,
    FireBreath,
    StealGold,
    StealItem,
    RustMetal,
    SplitOnHit,
    GazeParalyze,
    SummonAllies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Jackal,
    Goblin,
    FloatingEye,
    Kobold,
    Orc,
    Ogre,
    GiantSpider,
    GnollArcher,
    Troll,
    FireAnt,
    Minotaur,
    Dragon,
    Bat,
    Kitten,
    Housecat,
    LittleDog,
    Dog,
    Pony,
    Warhorse,
    Shopkeeper,
    TemplePriest,
    Oracle,
    Watchman,
    Leprechaun,
    Nymph,
    RustMonster,
    BlackPudding,
    Dwarf,
    Medusa,
    OrcCaptain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterTemplate {
    pub kind: MonsterKind,
    pub ai_kind: MonsterAiKind,
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
    pub speed: i16,
    pub passive: Option<MonsterPassive>,
    pub difficulty: u16,
    pub resistances: Resistances,
}

impl MonsterKind {
    pub fn ai_kind(self) -> MonsterAiKind {
        match self {
            Self::Jackal | Self::Bat => MonsterAiKind::Wander,
            Self::Goblin
            | Self::Kobold
            | Self::Orc
            | Self::Ogre
            | Self::GiantSpider
            | Self::Troll
            | Self::FireAnt
            | Self::Minotaur
            | Self::Kitten
            | Self::Housecat
            | Self::LittleDog
            | Self::Dog
            | Self::Pony
            | Self::Warhorse
            | Self::Shopkeeper
            | Self::TemplePriest
            | Self::Watchman
            | Self::Leprechaun
            | Self::Nymph
            | Self::RustMonster
            | Self::BlackPudding
            | Self::Dwarf
            | Self::OrcCaptain => MonsterAiKind::ChaseVisiblePlayer,
            Self::FloatingEye | Self::Oracle => MonsterAiKind::Stationary,
            Self::GnollArcher | Self::Dragon | Self::Medusa => MonsterAiKind::RangedOnSight,
        }
    }
    pub fn difficulty(self) -> u8 {
        match self {
            Self::Jackal => 1,
            Self::Goblin | Self::Kobold | Self::Bat | Self::Kitten | Self::LittleDog => 2,
            Self::Orc | Self::GiantSpider | Self::Pony => 3,
            Self::GnollArcher | Self::Housecat | Self::Dog | Self::Leprechaun | Self::Dwarf => 4,
            Self::FloatingEye
            | Self::Ogre
            | Self::FireAnt
            | Self::Nymph
            | Self::RustMonster
            | Self::OrcCaptain => 5,
            Self::Troll | Self::Warhorse | Self::Oracle | Self::Watchman | Self::BlackPudding => 6,
            Self::Minotaur => 7,
            Self::Dragon | Self::TemplePriest | Self::Medusa => 8,
            Self::Shopkeeper => 10,
        }
    }

    pub const fn is_p11_only(self) -> bool {
        matches!(
            self,
            Self::Kitten
                | Self::Housecat
                | Self::LittleDog
                | Self::Dog
                | Self::Pony
                | Self::Warhorse
                | Self::Shopkeeper
                | Self::TemplePriest
                | Self::Oracle
                | Self::Watchman
                | Self::Leprechaun
                | Self::Nymph
                | Self::RustMonster
                | Self::BlackPudding
                | Self::Dwarf
                | Self::Medusa
                | Self::OrcCaptain
        )
    }

    pub const fn is_pet_capable(self) -> bool {
        matches!(
            self,
            Self::Kitten
                | Self::Housecat
                | Self::LittleDog
                | Self::Dog
                | Self::Pony
                | Self::Warhorse
                | Self::Jackal
        )
    }

    pub const fn is_herbivore_pet(self) -> bool {
        matches!(self, Self::Pony | Self::Warhorse)
    }

    pub const fn evolved_form(self) -> Option<Self> {
        match self {
            Self::Kitten => Some(Self::Housecat),
            Self::LittleDog => Some(Self::Dog),
            Self::Pony => Some(Self::Warhorse),
            _ => None,
        }
    }
}
