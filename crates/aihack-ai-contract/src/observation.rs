use aihack_core::{
    action::{ActionIntent, CommandIntent},
    campaign::{Alignment, Attributes, LearnedSpell, Race, SkillCategory, SkillRank, SpellKind},
    domain::{
        entity::{EntityKind, Faction},
        inventory::InventoryLetter,
        item::{BucStatus, EquipmentSlot, ItemClass, ItemKind},
        level::SpecialRoomKind,
        tile::TileKind,
    },
    event::GameEvent,
    ids::{EntityId, LevelId},
    position::{Delta, Pos},
};
use serde::{Deserialize, Serialize};

pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;

fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

fn is_zero_i8(value: &i8) -> bool {
    *value == 0
}

fn is_zero_i16(value: &i16) -> bool {
    *value == 0
}

fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

fn default_quantity_one() -> u16 {
    1
}

fn is_one_u16(value: &u16) -> bool {
    *value == 1
}

fn is_true(value: &bool) -> bool {
    *value
}

fn default_true() -> bool {
    true
}

fn default_player_ac() -> i16 {
    10
}

fn is_default_player_ac(value: &i16) -> bool {
    *value == 10
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStateSummary {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection,
    AwaitingInventorySelection,
    MorePrompt,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerObservation {
    pub entity: EntityId,
    pub pos: Pos,
    pub hp: i16,
    pub max_hp: i16,
    #[serde(
        default = "default_player_ac",
        skip_serializing_if = "is_default_player_ac"
    )]
    pub ac: i16,
    pub current_level: LevelId,
    pub hunger: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub poison_turns: u8,
    pub hallucinating: bool,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub gold: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub fire_resistant: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub poison_resistant: bool,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub mp: i16,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub max_mp: i16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub resistances: u16,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub blind_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confused_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub stunned_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub fast_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub levitation_turns: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub invisible_turns: u8,
    #[serde(default, skip_serializing_if = "is_false")]
    pub blindfolded: bool,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub shop_debt: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityObservation {
    pub entity: EntityId,
    pub kind: EntityKind,
    pub pos: Pos,
    pub hp: Option<i16>,
    pub alive: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glyph: Option<char>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<Faction>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unpaid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionSpace {
    pub commands: Vec<ActionIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerItemObservation {
    pub container: EntityId,
    pub item: EntityId,
    pub kind: ItemKind,
    pub name: String,
    pub quantity: u16,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub buc_known: bool,
    #[serde(default, skip_serializing_if = "BucStatus::is_uncursed")]
    pub buc: BucStatus,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enchant_known: bool,
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub enchantment: i8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub erosion: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign: Option<CampaignObservation>,
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub current_level: LevelId,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub player_pos: Pos,
    pub visible_tiles: Vec<TileObservation>,
    pub visible_entities: Vec<EntityObservation>,
    pub inventory: Vec<ItemObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub container_items: Vec<ContainerItemObservation>,
    pub last_events: Vec<GameEvent>,
    pub action_space: ActionSpace,
    pub legal_actions: Vec<CommandIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillObservation {
    pub category: SkillCategory,
    pub rank: SkillRank,
    pub cap: SkillRank,
    pub practice: u16,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignObservation {
    pub carried_weight: i16,
    pub carrying_capacity: i16,
    pub role: aihack_core::campaign::Role,
    pub xp: u32,
    pub level: u8,
    pub has_amulet: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identified_items: Vec<ItemKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub race: Option<Race>,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub god_anger: i16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub prayer_timeout: u16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub artifact_invoke_cooldown: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub learned_spells: Vec<LearnedSpell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spell_failure_rates: Vec<(SpellKind, u8)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_level_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_room: Option<SpecialRoomKind>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub on_magic_portal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_high_altar: Option<Alignment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemObservation {
    pub item: EntityId,
    pub kind: ItemKind,
    pub letter: InventoryLetter,
    pub equipped_slot: Option<EquipmentSlot>,
    pub identified: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub buc_known: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enchant_known: bool,
    #[serde(default, skip_serializing_if = "BucStatus::is_uncursed")]
    pub buc: BucStatus,
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub enchantment: i8,
    #[serde(default = "default_quantity_one", skip_serializing_if = "is_one_u16")]
    pub quantity: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glyph: Option<char>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_class: Option<ItemClass>,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub contained_count: u16,
    #[serde(default, skip_serializing_if = "is_false")]
    pub tool_active: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub container_locked: bool,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub erosion: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unpaid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileObservation {
    pub pos: Pos,
    pub rel: Delta,
    pub tile: TileKind,
    pub visible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engraving: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altar_alignment: Option<Alignment>,
}
