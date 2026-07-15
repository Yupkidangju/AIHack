pub mod schema;

use std::sync::OnceLock;

pub use schema::{
    ContentRegistry, DoorData, HiddenDoorData, HiddenTrapData, ItemData, LevelData, LevelItemData,
    LevelMonsterData, MonsterData, WallData, CONTENT_SCHEMA_VERSION,
};

use crate::core::error::ContentError;

static EMBEDDED_REGISTRY: OnceLock<Result<ContentRegistry, ContentError>> = OnceLock::new();

/// Embed된 콘텐츠는 process당 한 번만 parse/validate한다.
pub fn registry() -> Result<&'static ContentRegistry, ContentError> {
    EMBEDDED_REGISTRY
        .get_or_init(ContentRegistry::from_embedded)
        .as_ref()
        .map_err(Clone::clone)
}

/// 하위 호환 조회 API다. 오류를 panic으로 바꾸지 않고 호출자에게 전파한다.
pub fn load_items() -> Result<Vec<ItemData>, ContentError> {
    Ok(registry()?.items().cloned().collect())
}

/// 하위 호환 조회 API다. 오류를 panic으로 바꾸지 않고 호출자에게 전파한다.
pub fn load_monsters() -> Result<Vec<MonsterData>, ContentError> {
    Ok(registry()?.monsters().cloned().collect())
}

/// 하위 호환 조회 API다. 알려지지 않은 level ID도 typed error로 반환한다.
pub fn load_level(level_id: &str) -> Result<LevelData, ContentError> {
    registry()?
        .level(level_id)
        .cloned()
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "load_level".to_owned(),
            target: level_id.to_owned(),
        })
}
