use aihack_core::{
    domain::{
        entity::{EntityKind, EntityLocation},
        inventory::{InventoryEntry, InventoryLetter},
        item::{BucStatus, ItemKind},
        level::{Engraving, SpecialRoomKind},
        monster::MonsterKind,
        status::{Resistances, StatusTimers},
        tile::TileKind,
    },
    event::GameEvent,
    hash::SnapshotHash,
    ids::{EntityId, LevelId},
    position::Pos,
    run_state::RunState,
    world::{is_zero_i16, is_zero_u16, is_zero_u8},
};
use serde::{Deserialize, Serialize};

use crate::world::GameWorld;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn is_zero_i8(value: &i8) -> bool {
    *value == 0
}

fn default_quantity_one() -> u16 {
    1
}

fn is_one_u16(value: &u16) -> bool {
    *value == 1
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: EntityKind,
    pub pos: Option<Pos>,
    pub hp: Option<i16>,
    pub alive: Option<bool>,
    pub item_kind: Option<ItemKind>,
    pub location: Option<EntityLocation>,
    pub assigned_letter: Option<InventoryLetter>,
    pub charges: Option<u8>,
    #[serde(default, skip_serializing_if = "BucStatus::is_uncursed")]
    pub buc: BucStatus,
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub enchantment: i8,
    #[serde(default = "default_quantity_one", skip_serializing_if = "is_one_u16")]
    pub quantity: u16,
    #[serde(default, skip_serializing_if = "is_false")]
    pub buc_known: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub enchant_known: bool,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub energy: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpse_monster: Option<MonsterKind>,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub corpse_age: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contained_items: Vec<EntityId>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub tool_active: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub container_locked: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub container_trapped: bool,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub erosion: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSnapshot {
    pub id: LevelId,
    pub map_width: i16,
    pub map_height: i16,
    pub map_tiles: Vec<TileKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explored: Vec<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lit_tiles: Vec<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_room: Option<SpecialRoomKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engravings: Vec<Engraving>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySnapshot {
    pub owner: EntityId,
    pub entries: Vec<InventoryEntry>,
    pub equipped_melee: Option<EntityId>,
    pub equipped_body: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_ring: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_shield: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_helm: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_cloak: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_gloves: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_boots: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_right_ring: Option<EntityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped_amulet: Option<EntityId>,
    pub next_letter_index: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<aihack_core::campaign::CampaignState>,
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunState,
    pub event_count: usize,
    pub last_event: Option<GameEvent>,
    pub current_level: LevelId,
    pub player_pos: Pos,
    pub levels: Vec<LevelSnapshot>,
    pub entities: Vec<EntitySnapshot>,
    pub inventory: InventorySnapshot,
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
}

impl GameSnapshot {
    pub fn from_world(
        seed: u64,
        turn: u64,
        run_state: RunState,
        event_log: &[GameEvent],
        world: &GameWorld,
    ) -> Self {
        let mut entities = world
            .entities
            .entities()
            .iter()
            .map(|entity| {
                if let Some((_, _, level, pos, stats, alive)) = entity.actor() {
                    EntitySnapshot {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: Some(pos),
                        hp: Some(stats.hp),
                        alive: Some(alive),
                        item_kind: None,
                        location: Some(EntityLocation::OnMap { level, pos }),
                        assigned_letter: None,
                        charges: None,
                        buc: BucStatus::Uncursed,
                        enchantment: 0,
                        quantity: 1,
                        buc_known: false,
                        enchant_known: false,
                        energy: stats.energy,
                        corpse_monster: None,
                        corpse_age: 0,
                        contained_items: Vec::new(),
                        tool_active: false,
                        container_locked: false,
                        container_trapped: false,
                        erosion: 0,
                    }
                } else {
                    let (kind, _, location, assigned_letter, charges) =
                        entity.item().expect("actor가 아니면 item payload여야 한다");
                    EntitySnapshot {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: None,
                        hp: None,
                        alive: None,
                        item_kind: Some(kind),
                        location: Some(location),
                        assigned_letter,
                        charges,
                        buc: entity.item_buc().unwrap_or(BucStatus::Uncursed),
                        enchantment: entity.item_enchantment().unwrap_or(0),
                        quantity: entity.item_quantity().unwrap_or(1),
                        buc_known: entity.item_buc_known().unwrap_or(false),
                        enchant_known: entity.item_enchant_known().unwrap_or(false),
                        energy: 0,
                        corpse_monster: entity.item_corpse_monster(),
                        corpse_age: entity.item_corpse_age().unwrap_or(0),
                        contained_items: entity.item_contained_items().unwrap_or_default().to_vec(),
                        tool_active: entity.item_tool_active().unwrap_or(false),
                        container_locked: entity.item_container_locked().unwrap_or(false),
                        container_trapped: entity.item_container_trapped().unwrap_or(false),
                        erosion: entity.item_erosion().unwrap_or(0),
                    }
                }
            })
            .collect::<Vec<_>>();
        entities.sort_by_key(|entity| entity.id.0);

        let mut levels = world
            .levels
            .levels
            .iter()
            .map(|level| LevelSnapshot {
                id: level.id,
                map_width: level.map.width,
                map_height: level.map.height,
                map_tiles: level.map.tiles().to_vec(),
                explored: level.explored.clone(),
                lit_tiles: level.lit_tiles.clone(),
                special_room: level.special_room,
                engravings: level.engravings.clone(),
            })
            .collect::<Vec<_>>();
        levels.sort_by_key(|level| level.id);

        Self {
            campaign: world.campaign,
            seed,
            turn,
            run_state,
            event_count: event_log.len(),
            last_event: event_log.last().cloned(),
            current_level: world.current_level(),
            player_pos: world.player_pos(),
            levels,
            entities,
            inventory: InventorySnapshot {
                owner: world.inventory.owner,
                entries: world.inventory.entries.clone(),
                equipped_melee: world.inventory.equipped_melee,
                equipped_body: world.inventory.equipped_body,
                equipped_ring: world.inventory.equipped_ring,
                equipped_shield: world.inventory.equipped_shield,
                equipped_helm: world.inventory.equipped_helm,
                equipped_cloak: world.inventory.equipped_cloak,
                equipped_gloves: world.inventory.equipped_gloves,
                equipped_boots: world.inventory.equipped_boots,
                equipped_right_ring: world.inventory.equipped_right_ring,
                equipped_amulet: world.inventory.equipped_amulet,
                next_letter_index: world.inventory.next_letter_index,
            },
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
        }
    }

    pub fn stable_hash(&self) -> SnapshotHash {
        let payload =
            serde_json::to_string(self).expect("GameSnapshot은 직렬화 실패가 없는 닫힌 타입이다");
        SnapshotHash(format!("{:016x}", fnv1a64(payload.as_bytes())))
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
