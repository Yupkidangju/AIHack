use aihack_core::ids::EntityId;
use aihack_runtime::{systems::items, world::GameWorld};

#[test]
fn runtime_exposes_read_only_item_queries() {
    let world = GameWorld::fixture_without_monsters();

    assert!(items::inventory_letter(&world, EntityId(5)).is_some());
}
