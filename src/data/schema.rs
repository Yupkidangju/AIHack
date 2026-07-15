use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::error::ContentError;

pub const CONTENT_SCHEMA_VERSION: u16 = 1;

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
            include_str!("items.toml"),
            include_str!("monsters.toml"),
            &[
                ("levels/main_1.toml", include_str!("levels/main_1.toml")),
                ("levels/main_2.toml", include_str!("levels/main_2.toml")),
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
        if let Some(damage) = &item.damage {
            validate_dice(damage)?;
        }
    }
    for monster in monsters.values() {
        validate_dice(&monster.damage)?;
    }
    for level in levels.values() {
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
    levels.values().any(|other| {
        other.branch == level.branch && other.depth == level.depth + 1 && other.stairs_up.is_some()
    })
}
fn has_paired_down(level: &LevelData, levels: &BTreeMap<String, LevelData>) -> bool {
    levels.values().any(|other| {
        other.branch == level.branch
            && other.depth == level.depth - 1
            && other.stairs_down.is_some()
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
