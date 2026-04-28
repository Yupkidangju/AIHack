use aihack::{
    core::Pos,
    domain::map::{GameMap, PHASE2_HEIGHT, PHASE2_PLAYER_START, PHASE2_WIDTH},
};

#[test]
fn map_fixture_is_40x20() {
    let map = GameMap::fixture_phase2();

    assert_eq!(map.width, PHASE2_WIDTH);
    assert_eq!(map.height, PHASE2_HEIGHT);
    assert_eq!(map.tile_count(), 800);
}

#[test]
fn map_bounds_returns_error() {
    let map = GameMap::fixture_phase2();
    for pos in [
        Pos { x: -1, y: 0 },
        Pos { x: 0, y: -1 },
        Pos { x: 40, y: 0 },
        Pos { x: 0, y: 20 },
    ] {
        assert!(map.tile(pos).is_err());
    }
}

#[test]
fn fixture_player_start_is_fixed() {
    assert_eq!(PHASE2_PLAYER_START, Pos { x: 5, y: 5 });
}
