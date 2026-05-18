use crate::{
    core::{error::GameError, event::GameEvent, position::Direction, world::GameWorld},
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

pub fn kick_door(world: &mut GameWorld, direction: Direction) -> Result<Vec<GameEvent>, GameError> {
    let pos = world.player_pos().offset(direction.delta());
    let tile = world.current_map().tile(pos)?;
    let mut events = Vec::new();
    match tile {
        TileKind::HiddenDoor => {
            world
                .current_map_mut()
                .set_tile(pos, TileKind::Door(DoorState::Open))?;
            events.push(GameEvent::TileRevealed {
                pos,
                tile: TileKind::Door(DoorState::Closed),
            });
            events.push(GameEvent::DoorKicked { pos });
            events.push(GameEvent::DoorChanged {
                pos,
                from: DoorState::Closed,
                to: DoorState::Open,
            });
            Ok(events)
        }
        TileKind::Door(DoorState::Closed) => {
            world
                .current_map_mut()
                .set_tile(pos, TileKind::Door(DoorState::Open))?;
            events.push(GameEvent::DoorKicked { pos });
            events.push(GameEvent::DoorChanged {
                pos,
                from: DoorState::Closed,
                to: DoorState::Open,
            });
            Ok(events)
        }
        other => Err(GameError::NoDoor { pos, tile: other }),
    }
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
