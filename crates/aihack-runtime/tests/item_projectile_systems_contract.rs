use aihack_core::{ids::EntityId, position::Direction, rng::GameRng};
use aihack_runtime::{
    systems::{items, projectiles},
    world::GameWorld,
};

#[test]
fn runtime_exposes_item_and_projectile_systems() {
    let mut world = GameWorld::fixture_without_monsters();
    let mut rng = GameRng::new(42);

    assert!(items::inventory_letter(&world, EntityId(5)).is_some());
    assert!(projectiles::throw_item(&mut world, &mut rng, EntityId(9), Direction::East,).is_ok());
}
