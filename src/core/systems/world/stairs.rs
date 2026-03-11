// Copyright 2026 방은호 (Eunho Bang). Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//!

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::LevelChange;
use crate::core::entity::{PlayerTag, Position};
use crate::ui::input::Command;
use legion::*;

/// [v3.0.0] GameContext 기반 전환 완료
pub fn stairs_system(ctx: &mut crate::core::context::GameContext) {
    let crate::core::context::GameContext {
        world,
        grid,
        log,
        cmd,
        turn,
        level_req,
        dungeon,
        ..
    } = ctx;
    let current_turn = *turn;

    let mut player_pos = None;
    {
        let mut query = <&Position>::query().filter(component::<PlayerTag>());
        for pos in query.iter(*world) {
            player_pos = Some((pos.x, pos.y));
        }
    }

    if let Some((px, py)) = player_pos {
        if let Some(tile) = grid.get_tile(px as usize, py as usize) {
            match *cmd {
                Command::Descend => {
                    if tile.typ == TileType::StairsDown || tile.typ == TileType::Ladder {
                        if let Some(target) = grid.portals.get(&(px as i32, py as i32)) {
                            log.add(
                                format!(
                                    "You enter a passage leading to level {} ({}).",
                                    target.depth,
                                    match target.branch {
                                        crate::core::dungeon::DungeonBranch::Main => "Main",
                                        crate::core::dungeon::DungeonBranch::Mines => "Mines",
                                        crate::core::dungeon::DungeonBranch::Sokoban => "Sokoban",
                                        _ => "Unknown",
                                    }
                                ),
                                current_turn,
                            );
                            **level_req = Some(LevelChange::Teleport {
                                target: *target,
                                landing: crate::core::dungeon::LandingType::Connection(
                                    dungeon.current_level,
                                ),
                            });
                        } else {
                            log.add("You descend the stairs.", current_turn);
                            **level_req = Some(LevelChange::NextLevel);
                        }
                    } else {
                        log.add("You can't go down here.", current_turn);
                    }
                }
                Command::Ascend => {
                    if tile.typ == TileType::StairsUp {
                        if let Some(target) = grid.portals.get(&(px as i32, py as i32)) {
                            log.add(
                                format!("You climb up to level {}.", target.depth),
                                current_turn,
                            );
                            **level_req = Some(LevelChange::Teleport {
                                target: *target,
                                landing: crate::core::dungeon::LandingType::Connection(
                                    dungeon.current_level,
                                ),
                            });
                        } else {
                            log.add("You ascend the stairs.", current_turn);
                            **level_req = Some(LevelChange::PrevLevel);
                        }
                    } else {
                        log.add("You can't go up here.", current_turn);
                    }
                }
                _ => {}
            }
        }
    }
}
