use aihack_core::{
    domain::tile::{DoorState, TileKind},
    error::GameError,
    event::GameEvent,
    position::Direction,
};

use crate::world::GameWorld;

pub fn open_door(
    world: &mut GameWorld,
    direction: Direction,
) -> Result<(DoorState, DoorState), GameError> {
    change_door(world, direction, DoorState::Closed, DoorState::Open)
}

pub fn close_door(
    world: &mut GameWorld,
    direction: Direction,
) -> Result<(DoorState, DoorState), GameError> {
    change_door(world, direction, DoorState::Open, DoorState::Closed)
}

pub fn kick_door(world: &mut GameWorld, direction: Direction) -> Result<Vec<GameEvent>, GameError> {
    let pos = world.player_pos().offset(direction.delta());
    aihack_core::doors::kick_door(world.current_map_mut(), pos)
}

pub fn door_state_in_direction(world: &GameWorld, direction: Direction) -> Option<DoorState> {
    let pos = world.player_pos().offset(direction.delta());
    match world.current_map().tile(pos) {
        Ok(TileKind::Door(state)) => Some(state),
        _ => None,
    }
}

fn change_door(
    world: &mut GameWorld,
    direction: Direction,
    expected: DoorState,
    next: DoorState,
) -> Result<(DoorState, DoorState), GameError> {
    let pos = world.player_pos().offset(direction.delta());
    aihack_core::doors::change_door(world.current_map_mut(), pos, expected, next)
}
