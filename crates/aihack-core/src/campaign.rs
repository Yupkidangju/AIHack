use crate::ids::EntityId;
use serde::{Deserialize, Serialize};

pub const MAX_XP: u32 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Knight,
    Scout,
    Mage,
}

impl Role {
    pub const fn base_hp(self) -> i16 {
        match self {
            Self::Knight => 28,
            Self::Scout => 22,
            Self::Mage => 18,
        }
    }
    pub const fn base_hit(self) -> i16 {
        match self {
            Self::Knight => 4,
            Self::Scout => 5,
            Self::Mage => 3,
        }
    }
    pub const fn base_damage(self) -> i16 {
        match self {
            Self::Knight => 2,
            Self::Scout => 1,
            Self::Mage => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignState {
    pub role: Role,
    pub xp: u32,
    pub amulet: EntityId,
}

impl CampaignState {
    pub fn level(self) -> u8 {
        (1 + self.xp.min(MAX_XP) / 20) as u8
    }
    pub fn max_hp(self) -> i16 {
        self.role.base_hp() + 4 * i16::from(self.level() - 1)
    }
    pub fn hit_bonus(self) -> i16 {
        self.role.base_hit() + i16::from(self.level() - 1)
    }
    pub fn damage_bonus(self) -> i16 {
        self.role.base_damage() + i16::from(self.level() - 1)
    }
}
