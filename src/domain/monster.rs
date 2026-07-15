use serde::{Deserialize, Serialize};

use crate::{
    core::error::ContentError,
    data,
    domain::combat::{AttackProfile, DamageRoll},
};

/// [v0.1.0] Phase 6 최소 monster AI 정책이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterAiKind {
    Wander,
    ChaseVisiblePlayer,
    Stationary,
}

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
    pub ai_kind: MonsterAiKind,
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
}

impl MonsterKind {
    /// [v0.1.0] monster kind에서 직접 AI 정책을 조회한다.
    pub fn ai_kind(self) -> MonsterAiKind {
        monster_ai_kind(self)
    }

    pub fn difficulty(self) -> u8 {
        match self {
            Self::Jackal => 1,
            Self::Goblin => 2,
            Self::FloatingEye => 5,
        }
    }
}

pub fn monster_template(kind: MonsterKind) -> MonsterTemplate {
    try_monster_template(kind)
        .expect("embedded content registry is validated before monster creation")
}

pub fn try_monster_template(kind: MonsterKind) -> Result<MonsterTemplate, ContentError> {
    let id = monster_id(kind);
    let definition =
        data::registry()?
            .monster(id)
            .ok_or_else(|| ContentError::UnknownReference {
                owner: "monster factory".to_owned(),
                target: id.to_owned(),
            })?;
    let ai_kind = match definition.ai.as_str() {
        "wander" => MonsterAiKind::Wander,
        "chase_on_sight" => MonsterAiKind::ChaseVisiblePlayer,
        "stationary" => MonsterAiKind::Stationary,
        ai => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: ai.to_owned(),
            })
        }
    };
    let damage = parse_damage(&definition.damage)?;
    let name = match kind {
        MonsterKind::Jackal => "bite",
        MonsterKind::Goblin => "short sword",
        MonsterKind::FloatingEye => "gaze",
    };
    Ok(MonsterTemplate {
        kind,
        ai_kind,
        hp: definition.hp,
        ac: definition.ac,
        hit_bonus: definition.hit_bonus,
        damage_bonus: 0,
        attack_profile: AttackProfile::natural(name, damage),
    })
}

pub fn monster_kind_from_id(id: &str) -> Result<MonsterKind, ContentError> {
    match id {
        "monster.jackal" => Ok(MonsterKind::Jackal),
        "monster.goblin" => Ok(MonsterKind::Goblin),
        "monster.floating_eye" => Ok(MonsterKind::FloatingEye),
        _ => Err(ContentError::UnknownReference {
            owner: "monster kind".to_owned(),
            target: id.to_owned(),
        }),
    }
}

fn monster_id(kind: MonsterKind) -> &'static str {
    match kind {
        MonsterKind::Jackal => "monster.jackal",
        MonsterKind::Goblin => "monster.goblin",
        MonsterKind::FloatingEye => "monster.floating_eye",
    }
}

fn parse_damage(value: &str) -> Result<DamageRoll, ContentError> {
    if value == "0" {
        return Ok(DamageRoll::none());
    }
    let Some((dice, sides)) = value.split_once('d') else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    let (Ok(dice), Ok(sides)) = (dice.parse(), sides.parse()) else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    Ok(DamageRoll::new(dice, sides))
}

pub fn monster_ai_kind(kind: MonsterKind) -> MonsterAiKind {
    monster_template(kind).ai_kind
}
