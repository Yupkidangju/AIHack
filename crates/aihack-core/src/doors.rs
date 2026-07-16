use crate::{
    domain::{
        map::GameMap,
        tile::{DoorState, TileKind},
    },
    error::GameError,
    event::GameEvent,
    position::Pos,
};

pub fn change_door(
    map: &mut GameMap,
    pos: Pos,
    expected: DoorState,
    next: DoorState,
) -> Result<(DoorState, DoorState), GameError> {
    match map.tile(pos)? {
        TileKind::Door(current) if current == expected => {
            map.set_tile(pos, TileKind::Door(next))?;
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

pub fn kick_door(map: &mut GameMap, pos: Pos) -> Result<Vec<GameEvent>, GameError> {
    match map.tile(pos)? {
        TileKind::HiddenDoor => {
            map.set_tile(pos, TileKind::Door(DoorState::Open))?;
            Ok(vec![
                GameEvent::TileRevealed {
                    pos,
                    tile: TileKind::Door(DoorState::Closed),
                },
                GameEvent::DoorKicked { pos },
                GameEvent::DoorChanged {
                    pos,
                    from: DoorState::Closed,
                    to: DoorState::Open,
                },
            ])
        }
        TileKind::Door(DoorState::Closed) => {
            map.set_tile(pos, TileKind::Door(DoorState::Open))?;
            Ok(vec![
                GameEvent::DoorKicked { pos },
                GameEvent::DoorChanged {
                    pos,
                    from: DoorState::Closed,
                    to: DoorState::Open,
                },
            ])
        }
        tile => Err(GameError::NoDoor { pos, tile }),
    }
}
