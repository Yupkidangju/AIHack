use serde::Deserialize;

/// [v0.2.0] Phase 20: items.toml에서 읽어오는 아이템 데이터다.
#[derive(Debug, Clone, Deserialize)]
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

/// [v0.2.0] Phase 20: monsters.toml에서 읽어오는 몬스터 데이터다.
#[derive(Debug, Clone, Deserialize)]
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

/// [v0.2.0] Phase 20: levels/에서 읽어오는 레벨 데이터다.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelData {
    pub level_id: String,
    pub branch: String,
    pub depth: i16,
    pub width: i16,
    pub height: i16,
    pub player_start: Vec<i16>,
    pub stairs_down: Vec<i16>,
    pub wall: Option<Vec<WallData>>,
    pub door: Option<Vec<DoorData>>,
    pub hidden_door: Option<Vec<HiddenDoorData>>,
    pub hidden_trap: Option<Vec<HiddenTrapData>>,
    pub monster: Option<Vec<LevelMonsterData>>,
    pub item: Option<Vec<LevelItemData>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WallData {
    pub x: i16,
    pub y_range: Vec<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DoorData {
    pub pos: Vec<i16>,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HiddenDoorData {
    pub pos: Vec<i16>,
    pub tile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HiddenTrapData {
    pub pos: Vec<i16>,
    pub trap: String,
    pub tile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LevelMonsterData {
    pub id: String,
    pub pos: Vec<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LevelItemData {
    pub id: String,
    pub pos: Vec<i16>,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemsToml {
    item: Vec<ItemData>,
}

#[derive(Debug, Clone, Deserialize)]
struct MonstersToml {
    monster: Vec<MonsterData>,
}

/// [v0.2.0] Phase 20: embed된 items.toml을 파싱한다.
pub fn load_items() -> Vec<ItemData> {
    let toml_str = include_str!("items.toml");
    let parsed: ItemsToml = toml::from_str(toml_str).expect("items.toml 파싱 실패");
    parsed.item
}

/// [v0.2.0] Phase 20: embed된 monsters.toml을 파싱한다.
pub fn load_monsters() -> Vec<MonsterData> {
    let toml_str = include_str!("monsters.toml");
    let parsed: MonstersToml = toml::from_str(toml_str).expect("monsters.toml 파싱 실패");
    parsed.monster
}

/// [v0.2.0] Phase 20: embed된 레벨 TOML을 파싱한다.
pub fn load_level(level_id: &str) -> LevelData {
    let toml_str = match level_id {
        "main:1" => include_str!("levels/main_1.toml"),
        _ => panic!("알 수 없는 레벨 ID: {}", level_id),
    };
    toml::from_str(toml_str).expect("레벨 TOML 파싱 실패")
}
