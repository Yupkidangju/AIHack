// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::capability::MonsterCapability;
use crate::core::entity::monster::MonsterTemplate;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MoveFlags: u32 {
        const ALLOW_TRAPS = 0x00020000;
        const ALLOW_U     = 0x00040000;
        const ALLOW_M     = 0x00080000;
        const OPENDOOR    = 0x00400000;
        const UNLOCKDOOR  = 0x00800000;
        const BUSTDOOR    = 0x01000000;
        const ALLOW_WALL  = 0x04000000;
        const ALLOW_DIG   = 0x08000000;
    }
}

pub struct AiHelper;

impl AiHelper {
    ///
    ///
    ///
    ///
    pub fn goodpos(
        grid: &Grid,
        x: usize,
        y: usize,
        template: &MonsterTemplate,
        flags: MoveFlags,
        occupancy: &std::collections::HashSet<(i32, i32)>,
        obstacles: &std::collections::HashSet<(i32, i32)>,
    ) -> bool {
        if x == 0 || y == 0 || x >= COLNO || y >= ROWNO {
            return false;
        }

        //
        let pos = (x as i32, y as i32);
        if occupancy.contains(&pos) {
            //
            //
            if !flags.contains(MoveFlags::ALLOW_M) {
                return false;
            }
        }

        //
        if obstacles.contains(&pos) {
            //
            return false;
        }

        if let Some(tile) = grid.get_tile(x, y) {
            //
            if tile.typ.is_wall() {
                if !template.has_capability(MonsterCapability::WallWalk) {
                    return false;
                }
            }
            if tile.typ == TileType::Stone {
                // Bedrock/Solid
                if !template.has_capability(MonsterCapability::WallWalk) {
                    return false;
                }
            }

            //
            if tile.typ == TileType::Pool || tile.typ == TileType::Water {
                let can_swim = template.has_capability(MonsterCapability::Swim);
                let can_fly = template.has_capability(MonsterCapability::Fly);
                let is_amph = template.has_capability(MonsterCapability::Amphibious);
                if !can_swim && !can_fly && !is_amph {
                    return false;
                }
            }

            //
            if tile.typ == TileType::LavaPool {
                let can_fly = template.has_capability(MonsterCapability::Fly);
                // NetHack: Fire resistance alone isn't enough to walk on lava usually, needed flying or levitation
                if !can_fly {
                    return false;
                }
            }

            //
            if tile.typ == TileType::Door {
                if flags.contains(MoveFlags::OPENDOOR) {
                    return true;
                }
                // TODO: Check UNLOCKDOOR if locked (need doormask)
                // TODO: Check BUSTDOOR (is_giant)
                return false;
            }

            return true;
        }

        false
    }

    ///
    ///
    pub fn mfndpos(
        grid: &Grid,
        mx: usize,
        my: usize,
        template: &MonsterTemplate,
        flags: MoveFlags,
        occupancy: &std::collections::HashSet<(i32, i32)>,
        obstacles: &std::collections::HashSet<(i32, i32)>,
    ) -> Vec<crate::core::entity::Position> {
        let mut candidates = Vec::with_capacity(9);
        use crate::core::entity::Position;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = mx as i32 + dx;
                let ny = my as i32 + dy;

                if nx <= 0 || ny <= 0 {
                    continue;
                }
                let (unx, uny) = (nx as usize, ny as usize);

                if Self::goodpos(grid, unx, uny, template, flags, occupancy, obstacles) {
                    candidates.push(Position { x: nx, y: ny });
                }
            }
        }
        candidates
    }

    ///
    ///
    pub fn distfleeck(
        m_pos: &crate::core::entity::Position,
        p_pos: &crate::core::entity::Position,
        monster: &crate::core::entity::Monster,
        grid: &Grid,
    ) -> (bool, bool, bool) {
        // inrange: within bolt range (NetHack: dist2 <= 64)
        let dist_sq = (m_pos.x - p_pos.x).pow(2) + (m_pos.y - p_pos.y).pow(2);
        let inrange = dist_sq <= 64; // BOLT_LIM * BOLT_LIM

        // nearby: inrange and 'monnear' (dist <= 9? usually smaller)
        // NetHack monnear is typically dist 1 or 2, but distfleeck says:
        // *nearby = *inrange && monnear(mtmp, mtmp->mux, mtmp->muy);
        // Let's assume nearby means very close (<= 2 squares?)
        // Actually monnear checks if distance is small. Let's use <= 4 (distance 2)
        let nearby = inrange && dist_sq <= 4;

        let mut scared = false;

        // Elbereth 泥댄겕
        if nearby {
            if crate::core::systems::engrave::is_protected_by_elbereth((p_pos.x, p_pos.y), grid) {
                if monster.hostile {
                    scared = true;
                }
            }
        }
        // TODO: check for Light (flees_light)
        // TODO: check for SANCTUARY

        (inrange, nearby, scared)
    }

    ///
    ///
    pub fn lined_up(
        grid: &Grid,
        ax: usize,
        ay: usize,
        bx: usize,
        by: usize,
        obstacles: &std::collections::HashSet<(i32, i32)>,
    ) -> bool {
        let dx = bx as isize - ax as isize;
        let dy = by as isize - ay as isize;

        // Must be straight or diagonal
        if dx != 0 && dy != 0 && dx.abs() != dy.abs() {
            return false;
        }

        // Distance check (BOLT_LIM = 8 usually)
        let dist_sq = (dx * dx) + (dy * dy);
        if dist_sq > 64 {
            // 8^2
            return false;
        }

        // Check obstacles
        let steps = std::cmp::max(dx.abs(), dy.abs());
        if steps == 0 {
            return true;
        } // Same tile

        let step_x = dx.signum();
        let step_y = dy.signum();

        let mut cx = ax as isize;
        let mut cy = ay as isize;

        for _ in 0..steps {
            cx += step_x;
            cy += step_y;
            if cx == bx as isize && cy == by as isize {
                break;
            }
            if let Some(tile) = grid.get_tile(cx as usize, cy as usize) {
                if tile.typ.is_wall() || tile.typ == TileType::Door || tile.typ == TileType::SDoor {
                    return false; // Blocked
                }
                if obstacles.contains(&(cx as i32, cy as i32)) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    ///
    pub fn get_path(
        grid: &Grid,
        start: (i32, i32),
        goal: (i32, i32),
        template: &MonsterTemplate,
        flags: MoveFlags,
        occupancy: &std::collections::HashSet<(i32, i32)>,
        obstacles: &std::collections::HashSet<(i32, i32)>,
    ) -> Option<Vec<(i32, i32)>> {
        crate::util::path::PathFinder::find_path(grid, start, goal, |g, x, y| {
            Self::goodpos(
                g, x as usize, y as usize, template, flags, occupancy, obstacles,
            )
        })
    }
}
