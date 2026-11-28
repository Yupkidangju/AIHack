use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    action::CommandIntent,
    campaign::LearnedSpell,
    domain::{
        inventory::Inventory,
        item::ItemKind,
        level::LevelRegistry,
        status::{Resistances, StatusTimers},
    },
    event::GameEvent,
    hash::SnapshotHash,
    ids::{EntityId, LevelId},
    position::Pos,
    rng::RngStateV1,
    run_state::RunState,
    turn::TurnOutcome,
    world::{is_zero_i16, is_zero_u32, is_zero_u8, WorldState},
};

pub const SAVE_SCHEMA_VERSION_V1: u16 = 1;
pub const SAVE_SCHEMA_VERSION_V2: u16 = 2;
pub const SAVE_SCHEMA_VERSION_V3: u16 = 3;
pub const SAVE_SCHEMA_VERSION_V4: u16 = 4;

/// JSON save에 포함되는 world 상태다. runtime-only 사망 원인은 저장하지 않는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedWorldV1<E> {
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
}

impl<E: Clone> From<&WorldState<E>> for SavedWorldV1<E> {
    fn from(world: &WorldState<E>) -> Self {
        Self {
            campaign: world.campaign,
            levels: world.levels.clone(),
            current_level: world.current_level,
            entities: world.entities.clone(),
            player_id: world.player_id,
            inventory: world.inventory.clone(),
            nutrition: world.nutrition,
            luck: world.luck,
            prayer_cooldown: world.prayer_cooldown,
            paralysis_turns: world.paralysis_turns,
            poison_turns: world.poison_turns,
            hallucinating: world.hallucinating,
            kill_count: world.kill_count,
            gold: world.gold,
            identified_items: world.identified_items.clone(),
            mp: world.mp,
            max_mp: world.max_mp,
            ascension_pursuit_stage: world.ascension_pursuit_stage,
            intrinsic_resistances: world.intrinsic_resistances,
            status_timers: world.status_timers,
            weapon_skills: world.weapon_skills.clone(),
            spell_skill: world.spell_skill.clone(),
            skill_practice: world.skill_practice.clone(),
            learned_spells: world.learned_spells.clone(),
            generated_artifacts: world.generated_artifacts.clone(),
            ascension_altars: world.ascension_altars.clone(),
            shop_debt: world.shop_debt,
            shop_debts: world.shop_debts.clone(),
        }
    }
}

impl<E> From<SavedWorldV1<E>> for WorldState<E> {
    fn from(saved: SavedWorldV1<E>) -> Self {
        Self {
            campaign: saved.campaign,
            levels: saved.levels,
            current_level: saved.current_level,
            entities: saved.entities,
            player_id: saved.player_id,
            inventory: saved.inventory,
            nutrition: saved.nutrition,
            luck: saved.luck,
            prayer_cooldown: saved.prayer_cooldown,
            paralysis_turns: saved.paralysis_turns,
            poison_turns: saved.poison_turns,
            hallucinating: saved.hallucinating,
            kill_count: saved.kill_count,
            gold: saved.gold,
            identified_items: saved.identified_items,
            mp: saved.mp,
            max_mp: saved.max_mp,
            ascension_pursuit_stage: saved.ascension_pursuit_stage,
            intrinsic_resistances: saved.intrinsic_resistances,
            status_timers: saved.status_timers,
            weapon_skills: saved.weapon_skills,
            spell_skill: saved.spell_skill,
            skill_practice: saved.skill_practice,
            learned_spells: saved.learned_spells,
            generated_artifacts: saved.generated_artifacts,
            ascension_altars: saved.ascension_altars,
            shop_debt: saved.shop_debt,
            shop_debts: saved.shop_debts,
            last_death_cause: None,
        }
    }
}

/// v1 JSON save envelope다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDataV1<E> {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunState,
    pub rng_state: RngStateV1,
    pub world: SavedWorldV1<E>,
    pub event_log: Vec<GameEvent>,
}

/// replay JSONL line schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLineV1 {
    pub turn_before: u64,
    pub command: CommandIntent,
    pub outcome: TurnOutcome,
    pub snapshot_hash_after: SnapshotHash,
}
