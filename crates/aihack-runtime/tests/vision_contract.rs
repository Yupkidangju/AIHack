use aihack_runtime::{systems::vision, world::GameWorld};

#[test]
fn runtime_vision_includes_the_player_origin() {
    let world = GameWorld::fixture_phase5();

    assert!(vision::visible_positions(&world).contains(&world.player_pos()));
}
