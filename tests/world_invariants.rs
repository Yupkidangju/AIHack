use aihack::{
    core::{GameSession, GameWorld, LevelId},
    domain::{entity::EntityLocation, inventory::Inventory},
};

fn saved_world() -> aihack::core::save::SavedWorldV1 {
    GameSession::new_for_playing(42).to_save_data().world
}

#[test]
fn fixture_checks_all_six_world_invariants() {
    let report = GameWorld::fixture_phase5().validate_invariants();

    assert_eq!(report.checked, 6);
    assert!(report.errors.is_empty());
}

#[test]
fn persisted_world_constructor_rejects_each_world_violation() {
    let mut missing_level = saved_world();
    missing_level.current_level = LevelId::main(99);
    assert!(GameWorld::from_saved_world(missing_level).is_err());

    let mut missing_player = saved_world();
    missing_player.player_id = aihack::core::EntityId(999);
    assert!(GameWorld::from_saved_world(missing_player).is_err());

    let mut non_player = saved_world();
    non_player.player_id = aihack::core::EntityId(2);
    assert!(GameWorld::from_saved_world(non_player).is_err());

    let mut wrong_level = saved_world();
    assert!(wrong_level.entities.set_actor_location(
        wrong_level.player_id,
        LevelId::main(2),
        aihack::core::Pos { x: 5, y: 5 },
    ));
    assert!(GameWorld::from_saved_world(wrong_level).is_err());

    let mut out_of_bounds = saved_world();
    assert!(out_of_bounds.entities.set_actor_location(
        out_of_bounds.player_id,
        out_of_bounds.current_level,
        aihack::core::Pos { x: -1, y: 5 },
    ));
    assert!(GameWorld::from_saved_world(out_of_bounds).is_err());

    let mut owner_mismatch = saved_world();
    owner_mismatch.inventory = Inventory::new(aihack::core::EntityId(2));
    owner_mismatch.entities.set_item_location(
        aihack::core::EntityId(5),
        EntityLocation::Inventory {
            owner: aihack::core::EntityId(2),
        },
    );
    assert!(GameWorld::from_saved_world(owner_mismatch).is_err());
}
