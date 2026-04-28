use serde::{Deserialize, Serialize};

use crate::core::ids::EntityId;

/// [v0.1.0] Phase 3 전투 피해 주사위 계약이다. dice=0 또는 sides=0은 피해 없음 데이터에만 사용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageRoll {
    pub dice: i16,
    pub sides: i16,
}

impl DamageRoll {
    pub const fn none() -> Self {
        Self { dice: 0, sides: 0 }
    }

    pub const fn new(dice: i16, sides: i16) -> Self {
        Self { dice, sides }
    }
}

/// [v0.1.0] Phase 3 장비 시스템이 없을 때 사용하는 내장 공격 프로필이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttackProfile {
    #[serde(skip)]
    pub name: &'static str,
    pub hit_bonus: i16,
    pub damage: DamageRoll,
}

impl AttackProfile {
    pub const fn dagger() -> Self {
        Self {
            name: "dagger",
            hit_bonus: 1,
            damage: DamageRoll::new(1, 4),
        }
    }

    pub const fn natural(name: &'static str, damage: DamageRoll) -> Self {
        Self {
            name,
            hit_bonus: 0,
            damage,
        }
    }
}

/// [v0.1.0] 사망 원인은 Phase 3에서 근접 전투만 기록한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeathCause {
    Combat { attacker: EntityId },
}
