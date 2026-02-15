// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//!

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, LevelChange};
use crate::core::entity::{PlayerTag, Position};
use crate::ui::input::Command;
use crate::ui::log::GameLog;
use legion::*;

pub fn stairs_system() -> impl systems::Runnable {
    SystemBuilder::new("stairs_system")
        .read_resource::<Command>()
        .read_resource::<Grid>()
        .read_resource::<crate::core::dungeon::dungeon::Dungeon>()
        .read_resource::<u64>() // Turn
        .write_resource::<GameLog>()
        .write_resource::<Option<LevelChange>>() // Level change request
        .with_query(<&Position>::query().filter(component::<PlayerTag>()))
        .build(
            |_command, world, (current_cmd, grid, dungeon, turn, log, level_req), query| {
                let mut player_pos = None;
                for pos in query.iter(world) {
                    player_pos = Some((pos.x, pos.y));
                }

                if let Some((px, py)) = player_pos {
                    if let Some(tile) = grid.get_tile(px as usize, py as usize) {
                        let cmd: Command = **current_cmd; // &AtomicRef -> Command
                        let current_turn: u64 = **turn; // &AtomicRef -> u64

                        match cmd {
                            Command::Descend => {
                                if tile.typ == TileType::StairsDown || tile.typ == TileType::Ladder
                                {
                                    if let Some(target) = grid.portals.get(&(px as i32, py as i32))
                                    {
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
                                    if let Some(target) = grid.portals.get(&(px as i32, py as i32))
                                    {
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
            },
        )
}
