use aihack_core::{
    domain::tile::{DoorState, TileKind},
    event::GameEvent,
    position::{Direction, Pos},
};
use aihack_runtime::{
    systems::{doors, stairs, traps},
    world::GameWorld,
};

#[test]
fn runtime_exposes_environment_interaction_systems() {
    let mut world = GameWorld::fixture_without_monsters();

    world.set_player_pos(Pos { x: 11, y: 5 });
    let hidden = Pos { x: 12, y: 5 };
    assert_eq!(world.current_map().tile(hidden), Ok(TileKind::HiddenDoor));
    let events = traps::search(&mut world);
    assert!(events
        .iter()
        .any(|event| matches!(event, GameEvent::TileRevealed { pos, .. } if *pos == hidden)));
    assert_eq!(
        world.current_map().tile(hidden),
        Ok(TileKind::Door(DoorState::Closed))
    );
    assert_eq!(
        doors::door_state_in_direction(&world, Direction::East),
        Some(DoorState::Closed)
    );
    let level_before = world.current_level();
    assert!(stairs::descend(&mut world).is_err());
    assert_eq!(world.current_level(), level_before);
}
