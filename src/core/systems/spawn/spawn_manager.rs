// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
//
// [v2.0.0
// =============================================================================

use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::spawn::{SpawnRequest, Spawner, NO_MM_FLAGS};
use crate::util::rng::NetHackRng;
use legion::{Entity, IntoQuery, World};

///
///
pub fn run_spawn_requests(
    world: &mut World,
    grid: &Grid,
    rng: &mut NetHackRng,
    assets: &AssetManager,
) {
    //
    let mut requests = Vec::new();
    let mut request_entities = Vec::new();

    {
        let mut query = <(Entity, &SpawnRequest)>::query();
        for (ent, req) in query.iter(world) {
            requests.push(req.clone());
            request_entities.push(*ent);
        }
    }

    //
    for req in requests {
        if let Some(template) = assets.monsters.get_template(&req.template) {
            //
            let level_id = get_player_level(world);
            Spawner::makemon(
                Some(template),
                req.x as usize,
                req.y as usize,
                NO_MM_FLAGS,
                world,
                grid,
                &assets.monsters.templates.values().collect(),
                &assets.items,
                rng,
                level_id,
            );
        }
    }

    //
    for ent in request_entities {
        world.remove(ent);
    }
}

///
///
///
pub fn turn_respawn(
    world: &mut World,
    grid: &Grid,
    rng: &mut NetHackRng,
    assets: &AssetManager,
    turn: u64,
) {
    //
    //
    if rng.rn2(50) != 0 {
        return;
    }

    //
    let mut player_x = 0i32;
    let mut player_y = 0i32;
    let mut player_level =
        crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1);

    {
        let mut p_query = <(&crate::core::entity::Position, &crate::core::entity::Level)>::query()
            .filter(legion::query::component::<crate::core::entity::PlayerTag>());

        if let Some((pos, lvl)) = p_query.iter(world).next() {
            player_x = pos.x;
            player_y = pos.y;
            player_level = lvl.0;
        }
    }

    //
    let mut current_monsters = 0;
    {
        let mut m_query = <&crate::core::entity::Level>::query()
            .filter(legion::query::component::<crate::core::entity::MonsterTag>());
        for lvl in m_query.iter(world) {
            if lvl.0 == player_level {
                current_monsters += 1;
            }
        }
    }

    //
    if current_monsters >= 40 {
        return;
    }

    //
    let colno = grid.locations.len();
    let rowno = if colno > 0 {
        grid.locations[0].len()
    } else {
        0
    };

    for _ in 0..50 {
        let x = rng.rn2(colno as i32) as usize;
        let y = rng.rn2(rowno as i32) as usize;

        //
        use crate::core::dungeon::tile::TileType;
        let tile_typ = grid.locations[x][y].typ;
        if tile_typ != TileType::Room && tile_typ != TileType::Corr {
            continue;
        }

        //
        let dx = (x as i32 - player_x).abs();
        let dy = (y as i32 - player_y).abs();
        if dx + dy < 15 {
            continue;
        }

        //
        let mut occupied = false;
        {
            let mut pos_query = <&crate::core::entity::Position>::query()
                .filter(legion::query::component::<crate::core::entity::MonsterTag>());
            for pos in pos_query.iter(world) {
                if pos.x == x as i32 && pos.y == y as i32 {
                    occupied = true;
                    break;
                }
            }
        }
        if occupied {
            continue;
        }

        //
        let templates: Vec<&crate::core::entity::monster::MonsterTemplate> =
            assets.monsters.templates.values().collect();
        Spawner::makemon(
            None,
            x,
            y,
            NO_MM_FLAGS,
            world,
            grid,
            &templates,
            &assets.items,
            rng,
            player_level,
        );

        break;
    }
}

///
fn get_player_level(world: &World) -> crate::core::dungeon::LevelID {
    let mut p_query = <&crate::core::entity::Level>::query()
        .filter(legion::query::component::<crate::core::entity::PlayerTag>());
    p_query
        .iter(world)
        .next()
        .map(|l| l.0)
        .unwrap_or(crate::core::dungeon::LevelID::new(
            crate::core::dungeon::DungeonBranch::Main,
            1,
        ))
}
