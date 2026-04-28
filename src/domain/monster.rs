use serde::{Deserialize, Serialize};

use crate::domain::combat::{AttackProfile, DamageRoll};

/// [v0.1.0] Phase 3 최소 몬스터 종류다. FloatingEye는 데이터만 보유하고 특수 능력은 후속 Phase로 미룬다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Jackal,
    Goblin,
    FloatingEye,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterTemplate {
    pub kind: MonsterKind,
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
}

pub fn monster_template(kind: MonsterKind) -> MonsterTemplate {
    match kind {
        MonsterKind::Jackal => MonsterTemplate {
            kind,
            hp: 4,
            ac: 0,
            hit_bonus: 0,
            damage_bonus: 0,
            attack_profile: AttackProfile::natural("bite", DamageRoll::new(1, 2)),
        },
        MonsterKind::Goblin => MonsterTemplate {
            kind,
            hp: 6,
            ac: 1,
            hit_bonus: 1,
            damage_bonus: 0,
            attack_profile: AttackProfile::natural("short sword", DamageRoll::new(1, 4)),
        },
        MonsterKind::FloatingEye => MonsterTemplate {
            kind,
            hp: 8,
            ac: 2,
            hit_bonus: 0,
            damage_bonus: 0,
            attack_profile: AttackProfile::natural("gaze", DamageRoll::none()),
        },
    }
}
