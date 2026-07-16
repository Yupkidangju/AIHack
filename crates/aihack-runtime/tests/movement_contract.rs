use aihack_core::position::Direction;
use aihack_runtime::{systems::movement, world::GameWorld};

#[test]
fn runtime_movement_updates_the_player_location() {
    let mut world = GameWorld::fixture_without_monsters();
    let before = world.player_pos();

    movement::move_player(&mut world, Direction::East).unwrap();

    assert_eq!(world.player_pos(), before.offset(Direction::East.delta()));
}
