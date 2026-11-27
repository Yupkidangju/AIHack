use aihack_content::ContentRegistry;
use aihack_core::error::ContentError;

pub use aihack_core::domain::monster::{
    MonsterAiKind, MonsterKind, MonsterPassive, MonsterTemplate,
};

pub fn monster_template(kind: MonsterKind) -> MonsterTemplate {
    try_monster_template(kind)
        .expect("embedded content registry is validated before monster creation")
}

pub fn try_monster_template(kind: MonsterKind) -> Result<MonsterTemplate, ContentError> {
    try_monster_template_from_registry(kind, aihack_content::registry()?)
}

pub fn try_monster_template_from_registry(
    kind: MonsterKind,
    registry: &ContentRegistry,
) -> Result<MonsterTemplate, ContentError> {
    aihack_content::monster_template_from_registry(kind, registry)
}

pub fn monster_kind_from_id(id: &str) -> Result<MonsterKind, ContentError> {
    aihack_content::monster_kind_from_id(id)
}

pub fn monster_ai_kind(kind: MonsterKind) -> MonsterAiKind {
    kind.ai_kind()
}
