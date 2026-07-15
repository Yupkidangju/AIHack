use aihack::{
    core::{ContentError, GameSession, LevelId, Pos},
    data::ContentRegistry,
    domain::{
        item::{try_item_data, ItemKind},
        monster::{try_monster_template, MonsterKind},
    },
};

#[test]
fn item_and_monster_factories_use_embedded_registry_values() {
    let dagger = try_item_data(ItemKind::Dagger).unwrap();
    let jackal = try_monster_template(MonsterKind::Jackal).unwrap();
    assert_eq!(
        dagger.weight,
        ContentRegistry::from_embedded()
            .unwrap()
            .item("item.weapon.dagger")
            .unwrap()
            .weight
    );
    assert_eq!(
        jackal.hp,
        ContentRegistry::from_embedded()
            .unwrap()
            .monster("monster.jackal")
            .unwrap()
            .hp
    );
}

#[test]
fn levels_and_initial_entities_are_created_from_embedded_definitions() {
    let session = GameSession::new_for_playing(42);
    assert_eq!(
        session
            .world()
            .map(LevelId::main(1))
            .tile(Pos { x: 34, y: 15 })
            .unwrap(),
        aihack::domain::tile::TileKind::StairsDown
    );
    assert_eq!(
        session
            .world()
            .map(LevelId::main(2))
            .tile(Pos { x: 5, y: 5 })
            .unwrap(),
        aihack::domain::tile::TileKind::StairsUp
    );
    assert_eq!(
        session
            .world()
            .entities()
            .actor_location(aihack::core::EntityId(2))
            .unwrap()
            .1,
        Pos { x: 6, y: 5 }
    );
}

#[test]
fn unknown_content_id_is_a_typed_error() {
    let error = aihack::data::load_level("main:404").unwrap_err();
    assert!(matches!(error, ContentError::UnknownReference { .. }));
}
