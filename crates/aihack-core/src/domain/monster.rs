use serde::{Deserialize, Serialize};

use crate::domain::combat::AttackProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterAiKind {
    Wander,
    ChaseVisiblePlayer,
    Stationary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Jackal,
    Goblin,
    FloatingEye,
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
}

impl MonsterKind {
    pub fn ai_kind(self) -> MonsterAiKind {
        match self {
            Self::Jackal => MonsterAiKind::Wander,
            Self::Goblin => MonsterAiKind::ChaseVisiblePlayer,
            Self::FloatingEye => MonsterAiKind::Stationary,
        }
    }
    pub fn difficulty(self) -> u8 {
        match self {
            Self::Jackal => 1,
            Self::Goblin => 2,
            Self::FloatingEye => 5,
        }
    }
}
