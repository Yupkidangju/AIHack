use aihack_core::position::Pos;
use aihack_runtime::domain::{
    entity::{EntityKind, EntityStore},
    monster::MonsterKind,
};

#[test]
fn runtime_entity_store_builds_a_content_backed_monster() {
    let mut entities = EntityStore::new();
    let monster = entities.spawn_monster(MonsterKind::Jackal, Pos { x: 6, y: 5 });

    let entity = entities.get(monster).expect("spawned monster must exist");
    let stats = entities
        .actor_stats(monster)
        .expect("monster stats must exist");
    assert_eq!(entity.kind(), EntityKind::Monster(MonsterKind::Jackal));
    assert_eq!(
        entities.actor_location(monster).unwrap().1,
        Pos { x: 6, y: 5 }
    );
    assert!(entity.is_alive_actor());
    assert!(stats.hp > 0 && stats.hp <= stats.max_hp);
    assert!(stats.speed > 0);
}
