use crate::core::entity::monster::MonsterFaction;

fn can_see_pos(vision: &VisionSystem, pos: &Position) -> bool {
    let ux = pos.x as usize;
    let uy = pos.y as usize;
    if ux < crate::core::dungeon::COLNO && uy < crate::core::dungeon::ROWNO {
        vision.viz_array[ux][uy] & crate::core::systems::vision::IN_SIGHT != 0
    } else {
        false
    }
}

fn move_towards(
    grid: &mut Grid,
    m_pos: &mut Position,
    p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
        let mut flags = MoveFlags::empty();
        if !template.has_capability(crate::core::entity::capability::MonsterCapability::Animal) {
            flags |= MoveFlags::OPENDOOR;
        }

        let candidates = AiHelper::mfndpos(
            grid,
            m_pos.x as usize,
            m_pos.y as usize,
            template,
            flags,
            occupancy,
            obstacles,
        );

        if candidates.is_empty() {
            return;
        }

        let mut best_pos = None;
        if let Some(path) = AiHelper::get_path(
            grid,
            (m_pos.x, m_pos.y),
            (p_pos.x, p_pos.y),
            template,
            flags,
            occupancy,
            obstacles,
        ) {
            if path.len() > 1 {
                let next = path[1];
                if !is_pet || !trap_positions.contains(&(next.0, next.1)) {
                    best_pos = Some(crate::core::entity::Position {
                        x: next.0,
                        y: next.1,
                    });
                }
            }
        }

        let best_pos = if let Some(bp) = best_pos {
            bp
        } else {
            let mut greedy_pos = *m_pos;
            let mut min_dist = (m_pos.x - p_pos.x).pow(2) + (m_pos.y - p_pos.y).pow(2);
            for pos in candidates {
                if pos.x == p_pos.x && pos.y == p_pos.y {
                    continue;
                }
                if is_pet && trap_positions.contains(&(pos.x, pos.y)) {
                    continue;
                }
                let dist = (pos.x - p_pos.x).pow(2) + (pos.y - p_pos.y).pow(2);
                if dist < min_dist {
                    min_dist = dist;
                    greedy_pos = pos;
                } else if dist == min_dist && rng.rn2(2) == 0 {
                    greedy_pos = pos;
                }
            }
            greedy_pos
        };

        if let Some(tile) = grid.get_tile_mut(best_pos.x as usize, best_pos.y as usize) {
            if tile.typ == crate::core::dungeon::tile::TileType::Door {
                tile.typ = crate::core::dungeon::tile::TileType::OpenDoor;
            }
        }
        m_pos.x = best_pos.x;
        m_pos.y = best_pos.y;
    }
}

fn move_random(
    grid: &mut Grid,
    m_pos: &mut Position,
    _p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
        let flags = MoveFlags::empty();
        let candidates = AiHelper::mfndpos(
            grid,
            m_pos.x as usize,
            m_pos.y as usize,
            template,
            flags,
            occupancy,
            obstacles,
        );
        if candidates.is_empty() {
            return;
        }

        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|pos| !is_pet || !trap_positions.contains(&(pos.x, pos.y)))
            .collect();

        if filtered.is_empty() {
            return;
        }
        let idx = rng.rn2(filtered.len() as i32) as usize;
        let next_pos = filtered[idx];

        if let Some(tile) = grid.get_tile_mut(next_pos.x as usize, next_pos.y as usize) {
            if tile.typ == crate::core::dungeon::tile::TileType::Door {
                tile.typ = crate::core::dungeon::tile::TileType::OpenDoor;
            }
        }
        m_pos.x = next_pos.x;
        m_pos.y = next_pos.y;
    }
}

fn move_away(
    grid: &mut Grid,
    m_pos: &mut Position,
    p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    // Similar to move_towards but maximizing distance
    // (Simplified implementation for brevity)
    move_random(
        grid,
        m_pos,
        p_pos,
        monster,
        assets,
        rng,
        occupancy,
        obstacles,
        trap_positions,
        is_pet,
    );
}
