use std::collections::BTreeMap;

use aihack_core::domain::{entity::MAX_ABSOLUTE_ACTOR_STAT, status::Resistances};
use serde::{Deserialize, Serialize};

use crate::core::error::ContentError;

pub const CONTENT_SCHEMA_VERSION: u16 = 1;
pub const MAX_CONTENT_ARMOR_AC_BONUS: i16 = 10_000;
pub const MAX_CONTENT_NUTRITION: i16 = 10_000;
pub const MAX_CONTENT_BASE_PRICE: i32 = 1_000_000;
pub const MAX_CONTENT_MONSTER_HP: i16 = MAX_ABSOLUTE_ACTOR_STAT as i16;
pub const MAX_CONTENT_MONSTER_SPEED: i16 = 36;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MonsterAttackData {
    pub kind: String,
    pub damage: String,
    #[serde(default)]
    pub effect: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ItemData {
    pub id: String,
    pub kind: String,
    pub glyph: String,
    pub weight: i16,
    pub slot: Option<String>,
    pub hit_bonus: Option<i16>,
    pub damage: Option<String>,
    pub effect: Option<String>,
    pub charges: Option<u8>,
    pub nutrition: Option<i16>,
    pub ac_bonus: Option<i16>,
    pub base_price: Option<i32>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub unidentified_pool: Option<String>,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub stackable: Option<bool>,
    #[serde(default)]
    pub damage_small: Option<String>,
    #[serde(default)]
    pub damage_large: Option<String>,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub spawn_depth_range: Option<[i16; 2]>,
    #[serde(default)]
    pub spawn_weight: Option<u16>,
    #[serde(default)]
    pub tool_action: Option<String>,
    #[serde(default)]
    pub container_capacity: Option<i16>,
    #[serde(default)]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MonsterData {
    pub id: String,
    pub glyph: String,
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage: String,
    pub ai: String,
    pub speed: i16,
    pub difficulty: i16,
    pub passive: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub base_level: Option<u8>,
    #[serde(default)]
    pub hp_dice: Option<String>,
    #[serde(default)]
    pub attacks: Vec<MonsterAttackData>,
    #[serde(default)]
    pub resistances: Vec<String>,
    #[serde(default)]
    pub corpse_chance: Option<u8>,
    #[serde(default)]
    pub corpse_nutrition: Option<i16>,
    #[serde(default)]
    pub corpse_effects: Vec<String>,
    #[serde(default)]
    pub ai_flags: Vec<String>,
    #[serde(default)]
    pub spawn_depth_range: Option<[i16; 2]>,
    #[serde(default)]
    pub spawn_weight: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LevelData {
    pub level_id: String,
    pub branch: String,
    pub depth: i16,
    pub width: i16,
    pub height: i16,
    pub player_start: Vec<i16>,
    pub stairs_down: Option<Vec<i16>>,
    pub stairs_up: Option<Vec<i16>>,
    pub wall: Option<Vec<WallData>>,
    pub door: Option<Vec<DoorData>>,
    pub hidden_door: Option<Vec<HiddenDoorData>>,
    pub hidden_trap: Option<Vec<HiddenTrapData>>,
    pub monster: Option<Vec<LevelMonsterData>>,
    pub item: Option<Vec<LevelItemData>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WallData {
    pub x: i16,
    pub y_range: Vec<i16>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DoorData {
    pub pos: Vec<i16>,
    pub state: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HiddenDoorData {
    pub pos: Vec<i16>,
    pub tile: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HiddenTrapData {
    pub pos: Vec<i16>,
    pub trap: String,
    pub tile: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LevelMonsterData {
    pub id: String,
    pub pos: Vec<i16>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LevelItemData {
    pub id: String,
    pub pos: Vec<i16>,
}

#[derive(Debug, Deserialize)]
struct ItemsToml {
    item: Vec<ItemData>,
}
#[derive(Debug, Deserialize)]
struct MonstersToml {
    monster: Vec<MonsterData>,
}

/// 정렬된 immutable registry. Runtime은 이 조회 전용 API만 사용한다.
#[derive(Debug, Clone)]
pub struct ContentRegistry {
    schema_version: u16,
    content_hash: String,
    items: BTreeMap<String, ItemData>,
    monsters: BTreeMap<String, MonsterData>,
    levels: BTreeMap<String, LevelData>,
}

impl ContentRegistry {
    pub fn from_embedded() -> Result<Self, ContentError> {
        Self::from_toml_sources(
            CONTENT_SCHEMA_VERSION,
            include_str!("data/items.toml"),
            include_str!("data/monsters.toml"),
            &[
                (
                    "levels/main_1.toml",
                    include_str!("data/levels/main_1.toml"),
                ),
                (
                    "levels/main_2.toml",
                    include_str!("data/levels/main_2.toml"),
                ),
            ],
        )
    }

    /// 테스트와 content import 경계가 동일한 검증을 사용하도록 제공한다.
    pub fn from_toml_sources(
        schema_version: u16,
        items_toml: &str,
        monsters_toml: &str,
        level_sources: &[(&str, &str)],
    ) -> Result<Self, ContentError> {
        if schema_version != CONTENT_SCHEMA_VERSION {
            return Err(ContentError::Parse {
                file: "content".to_owned(),
                message: format!("unsupported schema_version: {schema_version}"),
            });
        }
        let items = parse::<ItemsToml>("items.toml", items_toml)?.item;
        let monsters = parse::<MonstersToml>("monsters.toml", monsters_toml)?.monster;
        let levels = level_sources
            .iter()
            .map(|(file, source)| parse::<LevelData>(file, source))
            .collect::<Result<Vec<_>, _>>()?;

        let items = index_by_id(items, |entry| &entry.id)?;
        let monsters = index_by_id(monsters, |entry| &entry.id)?;
        let levels = index_by_id(levels, |entry| &entry.level_id)?;
        validate(&items, &monsters, &levels)?;

        let content_hash = canonical_hash(schema_version, &items, &monsters, &levels)?;
        Ok(Self {
            schema_version,
            content_hash,
            items,
            monsters,
            levels,
        })
    }

    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }
    pub fn item(&self, id: &str) -> Option<&ItemData> {
        self.items.get(id)
    }
    pub fn monster(&self, id: &str) -> Option<&MonsterData> {
        self.monsters.get(id)
    }
    pub fn level(&self, id: &str) -> Option<&LevelData> {
        self.levels.get(id)
    }
    pub fn items(&self) -> impl Iterator<Item = &ItemData> {
        self.items.values()
    }
    pub fn monsters(&self) -> impl Iterator<Item = &MonsterData> {
        self.monsters.values()
    }
    pub fn levels(&self) -> impl Iterator<Item = &LevelData> {
        self.levels.values()
    }
}

fn parse<T: for<'a> Deserialize<'a>>(file: &str, source: &str) -> Result<T, ContentError> {
    toml::from_str(source).map_err(|error| ContentError::Parse {
        file: file.to_owned(),
        message: error.to_string(),
    })
}

fn index_by_id<T>(
    entries: Vec<T>,
    id: impl Fn(&T) -> &String,
) -> Result<BTreeMap<String, T>, ContentError> {
    let mut indexed = BTreeMap::new();
    for entry in entries {
        let key = id(&entry).clone();
        if indexed.insert(key.clone(), entry).is_some() {
            return Err(ContentError::DuplicateId { id: key });
        }
    }
    Ok(indexed)
}

fn validate(
    items: &BTreeMap<String, ItemData>,
    monsters: &BTreeMap<String, MonsterData>,
    levels: &BTreeMap<String, LevelData>,
) -> Result<(), ContentError> {
    for item in items.values() {
        let mut glyphs = item.glyph.chars();
        if glyphs.next().is_none() || glyphs.next().is_some() {
            return item_contract_error(item, "glyph must contain exactly one Unicode scalar");
        }
        if let Some(expected_kind) = canonical_item_kind(&item.id) {
            if item.kind != expected_kind {
                return item_contract_error(
                    item,
                    &format!("declared kind must be canonical {expected_kind}"),
                );
            }
        }
        if item.weight < 0 {
            return Err(ContentError::Parse {
                file: "items.toml".to_owned(),
                message: format!("{} weight must be non-negative", item.id),
            });
        }
        if item
            .base_price
            .is_some_and(|price| !(0..=MAX_CONTENT_BASE_PRICE).contains(&price))
        {
            return item_contract_error(item, "base_price exceeds the consumer-safe range");
        }
        if item
            .nutrition
            .is_some_and(|value| !(1..=MAX_CONTENT_NUTRITION).contains(&value))
        {
            return item_contract_error(item, "nutrition exceeds the consumer-safe range");
        }
        if item
            .ac_bonus
            .is_some_and(|value| !(0..=MAX_CONTENT_ARMOR_AC_BONUS).contains(&value))
        {
            return item_contract_error(item, "ac_bonus exceeds the consumer-safe range");
        }
        validate_item_shape(item)?;
        validate_item_metadata(item)?;
        if let Some(damage) = &item.damage {
            validate_dice(damage)?;
        }
    }
    for monster in monsters.values() {
        if !(1..=MAX_CONTENT_MONSTER_HP).contains(&monster.hp) {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!("{} hp must be in 1..={MAX_CONTENT_MONSTER_HP}", monster.id),
            });
        }
        if !(0..=MAX_CONTENT_MONSTER_SPEED).contains(&monster.speed) {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!(
                    "{} speed must be in 0..={MAX_CONTENT_MONSTER_SPEED}",
                    monster.id
                ),
            });
        }
        if monster.difficulty <= 0 {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!("{} difficulty must be positive", monster.id),
            });
        }
        validate_dice(&monster.damage)?;
        validate_monster_metadata(monster)?;
    }
    for level in levels.values() {
        if !(1..i16::MAX).contains(&level.depth)
            || level.branch != "Main"
            || level.level_id != format!("main:{}", level.depth)
        {
            return Err(ContentError::Parse {
                file: "levels".to_owned(),
                message: format!(
                    "{} level identity/depth is outside the consumer-safe main branch",
                    level.level_id
                ),
            });
        }
        validate_level_coordinates(level)?;
        for entry in level.monster.as_deref().unwrap_or_default() {
            if !monsters.contains_key(&entry.id) {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: entry.id.clone(),
                });
            }
        }
        for entry in level.item.as_deref().unwrap_or_default() {
            if !items.contains_key(&entry.id) {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: entry.id.clone(),
                });
            }
        }
        if level.stairs_down.is_some() && !has_paired_up(level, levels) {
            return Err(ContentError::MissingStairsPair {
                level: level.level_id.clone(),
            });
        }
        if level.stairs_up.is_some() && !has_paired_down(level, levels) {
            return Err(ContentError::MissingStairsPair {
                level: level.level_id.clone(),
            });
        }
    }
    Ok(())
}

fn canonical_item_kind(id: &str) -> Option<&'static str> {
    match id {
        "item.weapon.dagger"
        | "item.weapon.short_sword"
        | "item.weapon.long_sword"
        | "item.weapon.katana"
        | "item.weapon.battle_axe"
        | "item.weapon.quarterstaff"
        | "item.weapon.broad_sword"
        | "item.weapon.mace"
        | "item.weapon.rock" => Some("weapon"),
        "item.food.ration" => Some("food"),
        "item.potion.healing"
        | "item.potion.extra_healing"
        | "item.potion.cure"
        | "item.potion.speed"
        | "item.potion.blindness"
        | "item.potion.confusion"
        | "item.potion.hallucination"
        | "item.potion.invisibility"
        | "item.potion.levitation" => Some("potion"),
        "item.wand.magic_missile" | "item.wand.fire" => Some("wand"),
        "item.scroll.reveal"
        | "item.scroll.identify"
        | "item.scroll.teleport"
        | "item.scroll.charging"
        | "item.scroll.enchant_weapon"
        | "item.scroll.enchant_armor"
        | "item.scroll.remove_curse" => Some("scroll"),
        "item.armor.leather"
        | "item.armor.chain_mail"
        | "item.armor.small_shield"
        | "item.armor.iron_helm"
        | "item.armor.cloak_protection"
        | "item.armor.leather_gloves"
        | "item.armor.low_boots" => Some("armor"),
        "item.ring.poison_resistance" | "item.ring.fire_resistance" | "item.ring.protection" => {
            Some("ring")
        }
        "item.amulet.esp" => Some("amulet"),
        "item.corpse.jackal" | "item.corpse.generic" => Some("corpse"),
        "item.quest.ascension" => Some("quest"),
        "item.tool.skeleton_key"
        | "item.tool.lock_pick"
        | "item.tool.pick_axe"
        | "item.tool.dwarvish_mattock"
        | "item.tool.tin_whistle"
        | "item.tool.oil_lamp"
        | "item.tool.blindfold"
        | "item.tool.unicorn_horn"
        | "item.tool.sack"
        | "item.tool.bag_of_holding"
        | "item.tool.chest" => Some("tool"),
        "item.potion.water" => Some("potion"),
        "item.scroll.blank" => Some("scroll"),
        _ => None,
    }
}

fn is_valid_color(color: &str) -> bool {
    matches!(
        color,
        "white"
            | "gray"
            | "brown"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "cyan"
            | "magenta"
            | "black"
            | "orange"
            | "bright_cyan"
            | "bright_red"
            | "bright_green"
            | "bright_yellow"
            | "bright_magenta"
            | "bright_blue"
    )
}

fn is_valid_material(material: &str) -> bool {
    matches!(
        material,
        "iron"
            | "steel"
            | "leather"
            | "cloth"
            | "wood"
            | "glass"
            | "paper"
            | "mineral"
            | "mithril"
            | "dragonhide"
            | "gold"
            | "silver"
            | "organic"
            | "gemstone"
    )
}

fn validate_depth_range(file: &str, owner: &str, range: [i16; 2]) -> Result<(), ContentError> {
    if range[0] < 1 || range[0] > range[1] || range[1] > 64 {
        return Err(ContentError::Parse {
            file: file.to_owned(),
            message: format!("{owner} has invalid spawn_depth_range"),
        });
    }
    Ok(())
}

fn validate_item_metadata(item: &ItemData) -> Result<(), ContentError> {
    if let Some(color) = &item.color {
        if !is_valid_color(color) {
            return item_contract_error(item, "has unsupported color");
        }
    }
    if let Some(material) = &item.material {
        if !is_valid_material(material) {
            return item_contract_error(item, "has unsupported material");
        }
    }
    if let Some(tool) = &item.tool_action {
        if !matches!(
            tool.as_str(),
            "light" | "unlock" | "dig" | "container" | "music" | "magic" | "blindfold"
        ) {
            return item_contract_error(item, "has unsupported tool_action");
        }
    }
    if let Some(capacity) = item.container_capacity {
        if capacity < 0 {
            return item_contract_error(item, "container_capacity must be non-negative");
        }
    }
    if let Some(damage) = &item.damage_small {
        validate_dice(damage)?;
    }
    if let Some(damage) = &item.damage_large {
        validate_dice(damage)?;
    }
    if let Some(range) = item.spawn_depth_range {
        validate_depth_range("items.toml", &item.id, range)?;
    }
    for effect in &item.effects {
        if Resistances::from_name(effect).is_none()
            && !matches!(effect.as_str(), "protection" | "luck")
        {
            return item_contract_error(item, "has unsupported effects entry");
        }
    }
    Ok(())
}

fn validate_monster_metadata(monster: &MonsterData) -> Result<(), ContentError> {
    let mut glyphs = monster.glyph.chars();
    if glyphs.next().is_none() || glyphs.next().is_some() {
        return Err(ContentError::Parse {
            file: "monsters.toml".to_owned(),
            message: format!("{} glyph must contain one scalar", monster.id),
        });
    }
    if let Some(color) = &monster.color {
        if !is_valid_color(color) {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!("{} has unsupported color {color}", monster.id),
            });
        }
    }
    if let Some(dice) = &monster.hp_dice {
        validate_dice(dice)?;
    }
    for attack in &monster.attacks {
        validate_dice(&attack.damage)?;
    }
    for resistance in &monster.resistances {
        if Resistances::from_name(resistance).is_none() {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!("{} has unsupported resistance {resistance}", monster.id),
            });
        }
    }
    if monster.corpse_chance.is_some_and(|chance| chance > 100) {
        return Err(ContentError::Parse {
            file: "monsters.toml".to_owned(),
            message: format!("{} corpse_chance must be in 0..=100", monster.id),
        });
    }
    if monster
        .corpse_nutrition
        .is_some_and(|n| !(0..=MAX_CONTENT_NUTRITION).contains(&n))
    {
        return Err(ContentError::Parse {
            file: "monsters.toml".to_owned(),
            message: format!("{} corpse_nutrition out of range", monster.id),
        });
    }
    for effect in &monster.corpse_effects {
        if !matches!(
            effect.as_str(),
            "poisonous"
                | "grant_fire_resistance"
                | "grant_poison_resistance"
                | "grant_cold_resistance"
                | "grant_shock_resistance"
                | "grant_telepathy"
                | "hallucinogenic"
                | "revive_troll"
        ) {
            return Err(ContentError::Parse {
                file: "monsters.toml".to_owned(),
                message: format!("{} has unsupported corpse_effect {effect}", monster.id),
            });
        }
    }
    if let Some(range) = monster.spawn_depth_range {
        validate_depth_range("monsters.toml", &monster.id, range)?;
    }
    Ok(())
}

fn item_contract_error(item: &ItemData, message: &str) -> Result<(), ContentError> {
    Err(ContentError::Parse {
        file: "items.toml".to_owned(),
        message: format!("{} {message}", item.id),
    })
}

fn validate_item_shape(item: &ItemData) -> Result<(), ContentError> {
    let dedicated_fields_valid = match item.kind.as_str() {
        "quest" => {
            item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.effect.is_none()
                && item.charges.is_none()
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        "weapon" => {
            item.slot.as_deref() == Some("melee")
                && item.hit_bonus.is_some()
                && item.damage.is_some()
                && item.effect.is_none()
                && item.charges.is_none()
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        "food" | "corpse" => {
            item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.nutrition.is_some()
                && item.effect.is_none()
                && item.charges.is_none()
                && item.ac_bonus.is_none()
        }
        "potion" => {
            item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && matches!(
                    item.effect.as_deref(),
                    Some(
                        "heal_1d8_plus_4"
                            | "heal_2d8_plus_8"
                            | "cure_status"
                            | "speed"
                            | "blindness"
                            | "confusion"
                            | "hallucination"
                            | "invisibility"
                            | "levitation"
                            | "water"
                    )
                )
                && item.charges.is_none()
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        "wand" => {
            item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && matches!(item.effect.as_deref(), Some("magic_missile" | "fire"))
                && item.charges.is_some_and(|charges| charges > 0)
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        "scroll" => {
            item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && matches!(
                    item.effect.as_deref(),
                    Some(
                        "reveal"
                            | "identify"
                            | "teleport"
                            | "charging"
                            | "enchant_weapon"
                            | "enchant_armor"
                            | "remove_curse"
                            | "blank"
                    )
                )
                && item.charges.is_none()
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        "armor" => {
            matches!(
                item.slot.as_deref(),
                Some("body" | "shield" | "helm" | "cloak" | "gloves" | "boots")
            ) && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.ac_bonus.is_some()
                && item.effect.is_none()
                && item.charges.is_none()
                && item.nutrition.is_none()
        }
        "ring" => {
            item.slot.as_deref() == Some("ring")
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.charges.is_none()
                && item.nutrition.is_none()
                && match item.effect.as_deref() {
                    Some("poison_resistance" | "fire_resistance") => item.ac_bonus.is_none(),
                    Some("protection") => item.ac_bonus.is_some(),
                    _ => false,
                }
        }
        "amulet" => {
            item.slot.as_deref() == Some("amulet")
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.charges.is_none()
                && item.nutrition.is_none()
        }
        "tool" => {
            let valid_tool_fields = match item.tool_action.as_deref() {
                Some("light") => {
                    item.charges.is_some_and(|c| c > 0) && item.container_capacity.is_none()
                }
                Some("container") => {
                    item.container_capacity.is_some_and(|c| c > 0) && item.charges.is_none()
                }
                Some(_) => item.charges.is_none() && item.container_capacity.is_none(),
                None => false,
            };
            valid_tool_fields
                && item.slot.is_none()
                && item.hit_bonus.is_none()
                && item.damage.is_none()
                && item.effect.is_none()
                && item.nutrition.is_none()
                && item.ac_bonus.is_none()
        }
        _ => false,
    };
    if dedicated_fields_valid {
        Ok(())
    } else {
        item_contract_error(item, "has an invalid kind-specific field shape")
    }
}

fn validate_dice(value: &str) -> Result<(), ContentError> {
    if value == "0" {
        return Ok(());
    }
    let Some((dice, sides)) = value.split_once('d') else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    let Ok(dice) = dice.parse::<i16>() else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    let Ok(sides) = sides.parse::<i16>() else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    if dice <= 0 || sides <= 0 {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_level_coordinates(level: &LevelData) -> Result<(), ContentError> {
    let check = |pos: &[i16]| coordinate(level, pos);
    check(&level.player_start)?;
    if let Some(pos) = &level.stairs_down {
        check(pos)?;
    }
    if let Some(pos) = &level.stairs_up {
        check(pos)?;
    }
    for wall in level.wall.as_deref().unwrap_or_default() {
        if wall.y_range.len() != 2 {
            return Err(ContentError::InvalidCoordinate {
                level: level.level_id.clone(),
                x: wall.x,
                y: 0,
            });
        }
        for y in wall.y_range[0]..=wall.y_range[1] {
            check(&[wall.x, y])?;
        }
    }
    for pos in level
        .door
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|entry| &entry.pos)
        .chain(
            level
                .hidden_door
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|entry| &entry.pos),
        )
        .chain(
            level
                .hidden_trap
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|entry| &entry.pos),
        )
        .chain(
            level
                .monster
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|entry| &entry.pos),
        )
        .chain(
            level
                .item
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|entry| &entry.pos),
        )
    {
        check(pos)?;
    }
    Ok(())
}

fn coordinate(level: &LevelData, value: &[i16]) -> Result<(), ContentError> {
    let (x, y) = match value {
        [x, y] => (*x, *y),
        [x, ..] => (*x, 0),
        [] => (0, 0),
    };
    if value.len() != 2 || x < 0 || y < 0 || x >= level.width || y >= level.height {
        return Err(ContentError::InvalidCoordinate {
            level: level.level_id.clone(),
            x,
            y,
        });
    }
    Ok(())
}

fn has_paired_up(level: &LevelData, levels: &BTreeMap<String, LevelData>) -> bool {
    let Some(next_depth) = level.depth.checked_add(1) else {
        return false;
    };
    levels.values().any(|other| {
        other.branch == level.branch && other.depth == next_depth && other.stairs_up.is_some()
    })
}
fn has_paired_down(level: &LevelData, levels: &BTreeMap<String, LevelData>) -> bool {
    let Some(previous_depth) = level.depth.checked_sub(1) else {
        return false;
    };
    levels.values().any(|other| {
        other.branch == level.branch && other.depth == previous_depth && other.stairs_down.is_some()
    })
}

fn canonical_hash(
    schema_version: u16,
    items: &BTreeMap<String, ItemData>,
    monsters: &BTreeMap<String, MonsterData>,
    levels: &BTreeMap<String, LevelData>,
) -> Result<String, ContentError> {
    let canonical =
        serde_json::to_vec(&(schema_version, items, monsters, levels)).map_err(|error| {
            ContentError::Parse {
                file: "content".to_owned(),
                message: error.to_string(),
            }
        })?;
    let hash = canonical.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    Ok(format!("{hash:016x}"))
}
