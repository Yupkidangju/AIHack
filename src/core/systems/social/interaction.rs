// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//!
//!

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::{PlayerTag, Position};
use crate::core::game_state::{Direction, DirectionAction};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::*;

///
///
pub fn try_open_door(
    world: &mut World,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    direction: Direction,
) -> bool {
    //
    let mut player_pos: Option<(i32, i32)> = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    let (px, py) = match player_pos {
        Some(p) => p,
        None => {
            log.add("You don't exist!", turn);
            return false;
        }
    };

    //
    let (dx, dy) = direction.to_delta();
    let tx = px + dx;
    let ty = py + dy;

    //
    if tx < 0 || tx >= COLNO as i32 || ty < 0 || ty >= ROWNO as i32 {
        log.add("You see no door there.", turn);
        return false;
    }

    //
    if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
        match tile.typ {
            TileType::Door => {
                //
                //
                //
                tile.typ = TileType::OpenDoor;
                tile.doormas = 1;
                log.add("The door opens.", turn);
                true
            }
            TileType::OpenDoor => {
                log.add("This door is already open.", turn);
                false
            }
            _ => {
                log.add("You see no door there.", turn);
                false
            }
        }
    } else {
        log.add("You see no door there.", turn);
        false
    }
}

///
///
pub fn try_close_door(
    world: &mut World,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    direction: Direction,
) -> bool {
    //
    let mut player_pos: Option<(i32, i32)> = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    let (px, py) = match player_pos {
        Some(p) => p,
        None => {
            log.add("You don't exist!", turn);
            return false;
        }
    };

    //
    let (dx, dy) = direction.to_delta();
    let tx = px + dx;
    let ty = py + dy;

    //
    if tx < 0 || tx >= COLNO as i32 || ty < 0 || ty >= ROWNO as i32 {
        log.add("You see no door there.", turn);
        return false;
    }

    //
    if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
        match tile.typ {
            TileType::OpenDoor => {
                //
                //
                tile.typ = TileType::Door;
                tile.doormas = 0;
                log.add("The door closes.", turn);
                true
            }
            TileType::Door => {
                log.add("This door is already closed.", turn);
                false
            }
            _ => {
                log.add("You see no door there.", turn);
                false
            }
        }
    } else {
        log.add("You see no door there.", turn);
        false
    }
}

///
pub fn execute_direction_action(
    action: DirectionAction,
    direction: Direction,
    world: &mut World,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) -> bool {
    match action {
        DirectionAction::Open => try_open_door(world, grid, log, turn, direction),
        DirectionAction::Close => try_close_door(world, grid, log, turn, direction),
        DirectionAction::Kick => {
            //
            crate::core::systems::kick::try_kick(world, grid, log, turn, direction, rng)
        }
        DirectionAction::Search => {
            //
            log.add("You search the area but find nothing.", turn);
            true
        }
        DirectionAction::Throw { .. } => {
            //
            true
        }
        DirectionAction::Cast { .. } => {
            //
            true
        }
        DirectionAction::Talk => {
            //
            true
        }
        DirectionAction::Zap { .. } => {
            //
            true
        }
        DirectionAction::Apply { .. } => {
            //
            true
        }
        DirectionAction::Loot => {
            //
            true
        }
    }
}
