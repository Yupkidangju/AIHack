use serde::{Deserialize, Serialize};

use crate::{
    domain::{combat::DeathCause, inventory::Inventory, item::ItemKind, level::LevelRegistry},
    ids::{EntityId, LevelId},
};

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
    pub hallucinating: bool,
    pub kill_count: u32,
    pub gold: u32,
    pub identified_items: Vec<ItemKind>,
    /// 현재 실행 중인 사망 처리만 위한 transient 값이며 v1 save에는 넣지 않는다.
    pub last_death_cause: Option<DeathCause>,
}
