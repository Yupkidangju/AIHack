use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    campaign::LearnedSpell,
    domain::{
        combat::DeathCause,
        inventory::Inventory,
        item::ItemKind,
        level::LevelRegistry,
        status::{Resistances, StatusTimers},
    },
    ids::{EntityId, LevelId},
    position::Pos,
};

pub fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

pub fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

pub fn is_zero_i16(value: &i16) -> bool {
    *value == 0
}

pub fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

/// UI, content bootstrap, 파일 I/O와 분리된 게임 world의 저장 가능 상태다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState<E> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<crate::campaign::CampaignState>,
    pub levels: LevelRegistry,
    pub current_level: LevelId,
    pub entities: E,
    pub player_id: EntityId,
    pub inventory: Inventory,
    pub nutrition: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub poison_turns: u8,
    pub hallucinating: bool,
    pub kill_count: u32,
    pub gold: u32,
    pub identified_items: Vec<ItemKind>,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub mp: i16,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub max_mp: i16,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub ascension_pursuit_stage: u8,
    #[serde(default, skip_serializing_if = "Resistances::is_empty")]
    pub intrinsic_resistances: Resistances,
    #[serde(default, skip_serializing_if = "StatusTimers::is_empty")]
    pub status_timers: StatusTimers,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub weapon_skills: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub spell_skill: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub skill_practice: BTreeMap<String, u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub learned_spells: Vec<LearnedSpell>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub generated_artifacts: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ascension_altars: Vec<(LevelId, Pos)>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub shop_debt: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shop_debts: Vec<(LevelId, u32)>,
    /// 현재 실행 중인 사망 처리만 위한 transient 값이며 save에는 넣지 않는다.
    pub last_death_cause: Option<DeathCause>,
}
