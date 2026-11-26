use serde::{Deserialize, Serialize};

use crate::domain::combat::AttackProfile;

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
}

impl MonsterKind {
    pub fn ai_kind(self) -> MonsterAiKind {
        match self {
            Self::Jackal => MonsterAiKind::Wander,
            Self::Goblin
            | Self::Kobold
            | Self::Orc
            | Self::Ogre
            | Self::GiantSpider
            | Self::Troll => MonsterAiKind::ChaseVisiblePlayer,
            Self::FloatingEye => MonsterAiKind::Stationary,
            Self::GnollArcher => MonsterAiKind::RangedOnSight,
        }
    }
    pub fn difficulty(self) -> u8 {
        match self {
            Self::Jackal => 1,
            Self::Goblin | Self::Kobold => 2,
            Self::Orc | Self::GiantSpider => 3,
            Self::GnollArcher => 4,
            Self::FloatingEye | Self::Ogre => 5,
            Self::Troll => 6,
        }
    }
}
