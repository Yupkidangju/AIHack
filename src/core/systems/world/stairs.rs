// Copyright 2026 방은호 (Eunho Bang). Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//!

use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::LevelChange;
use crate::core::entity::{PlayerTag, Position};
use crate::ui::input::Command;
use legion::*;

/// [v3.0.0] GameContext 기반 전환 완료
/// [v3.0.0 E4] LLM 비동기 던전 분위기 묘사 추가
pub fn stairs_system(ctx: &mut crate::core::context::GameContext) {
    let crate::core::context::GameContext {
        world,
        grid,
        log,
        cmd,
        turn,
        level_req,
        dungeon,
        llm,
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
            // 층 이동 발생 여부 + 목표 층 추적
            let mut transition_depth: Option<i32> = None;
            let mut transition_branch = "dungeon";
            let mut transition_direction = "down";

            match *cmd {
                Command::Descend => {
                    if tile.typ == TileType::StairsDown || tile.typ == TileType::Ladder {
                        if let Some(target) = grid.portals.get(&(px as i32, py as i32)) {
                            let branch_name = match target.branch {
                                crate::core::dungeon::DungeonBranch::Main => "Main",
                                crate::core::dungeon::DungeonBranch::Mines => "Mines",
                                crate::core::dungeon::DungeonBranch::Sokoban => "Sokoban",
                                _ => "Unknown",
                            };
                            log.add(
                                format!(
                                    "You enter a passage leading to level {} ({}).",
                                    target.depth, branch_name
                                ),
                                current_turn,
                            );
                            transition_depth = Some(target.depth);
                            transition_branch = match target.branch {
                                crate::core::dungeon::DungeonBranch::Mines => "mines",
                                crate::core::dungeon::DungeonBranch::Sokoban => "sokoban",
                                _ => "dungeon",
                            };
                            **level_req = Some(LevelChange::Teleport {
                                target: *target,
                                landing: crate::core::dungeon::LandingType::Connection(
                                    dungeon.current_level,
                                ),
                            });
                        } else {
                            log.add("You descend the stairs.", current_turn);
                            transition_depth = Some(dungeon.current_level.depth + 1);
                            **level_req = Some(LevelChange::NextLevel);
                        }
                    } else {
                        log.add("You can't go down here.", current_turn);
                    }
                }
                Command::Ascend => {
                    if tile.typ == TileType::StairsUp {
                        transition_direction = "up";
                        if let Some(target) = grid.portals.get(&(px as i32, py as i32)) {
                            log.add(
                                format!("You climb up to level {}.", target.depth),
                                current_turn,
                            );
                            transition_depth = Some(target.depth);
                            **level_req = Some(LevelChange::Teleport {
                                target: *target,
                                landing: crate::core::dungeon::LandingType::Connection(
                                    dungeon.current_level,
                                ),
                            });
                        } else {
                            log.add("You ascend the stairs.", current_turn);
                            transition_depth = Some(dungeon.current_level.depth - 1);
                            **level_req = Some(LevelChange::PrevLevel);
                        }
                    } else {
                        log.add("You can't go up here.", current_turn);
                    }
                }
                _ => {}
            }

            // [v3.0.0 E4] LLM 비동기 던전 분위기 묘사
            // 층 이동이 실제로 발생했을 때만 요청
            if let (Some(depth), Some(llm_engine)) = (transition_depth, llm) {
                let prompt = format!(
                    "You are a roguelike game narrator. The player just moved {} to level {} of the {}.\n\
                     Write a very short atmospheric description (1 sentence, max 15 words) of what they sense.\n\
                     Examples: 'A damp chill rises from below.' / 'The air smells of sulfur and decay.'\n\
                     Be creative, dark, and vary by depth. Deeper = more dangerous atmosphere.\n\
                     Reply with ONLY the description, no quotes.",
                    transition_direction, depth, transition_branch
                );
                let _request = llm_engine.generate_async(prompt, 30);
                // TODO: 향후 game_loop에서 request.try_get() 폴링 → 로그에 분위기 텍스트 추가
                // 현재는 요청만 발사 (비동기이므로 턴 블로킹 없음)
            }
        }
    }
}
