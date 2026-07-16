use aihack_core::position::Pos;
use aihack_runtime::domain::{entity::EntityStore, monster::MonsterKind};

#[test]
fn runtime_entity_store_builds_a_content_backed_monster() {
    let mut entities = EntityStore::new();
    let monster = entities.spawn_monster(MonsterKind::Jackal, Pos { x: 6, y: 5 });

    assert!(entities.get(monster).is_some());
}
