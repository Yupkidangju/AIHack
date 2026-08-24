use aihack_runtime::{systems::vision, GameSession};

#[test]
fn runtime_vision_includes_the_player_origin() {
    let session = GameSession::new_for_playing(42);
    let world = session.world();

    assert!(vision::visible_positions(world).contains(&world.player_pos()));
}
