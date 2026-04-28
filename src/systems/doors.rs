use crate::{
    core::{error::GameError, position::Direction, world::GameWorld},
    domain::tile::{DoorState, TileKind},
};

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
    match world.current_map().tile(pos)? {
        TileKind::Door(current) if current == expected => {
            world
                .current_map_mut()
                .set_tile(pos, TileKind::Door(next))?;
            Ok((current, next))
        }
        TileKind::Door(current) => Err(GameError::InvalidDoorState {
            pos,
            expected,
            actual: current,
        }),
        tile => Err(GameError::NoDoor { pos, tile }),
    }
}
