use aihack_core::{
    domain::{level::BranchGraph, tile::TileKind},
    event::GameEvent,
    ids::LevelId,
};

use crate::world::GameWorld;

pub fn enter_branch(world: &mut GameWorld) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("branch navigation is only available in campaign mode".into());
    }
    let from = world.current_level();
    let tile = world
        .current_map()
        .tile(world.player_pos())
        .map_err(|error| error.to_string())?;
    let graph = BranchGraph::campaign_v3();
    let to = graph
        .branch_entry_from(from, tile)
        .or_else(|| {
            (tile == TileKind::MagicPortal)
                .then(|| graph.portal_target(from))
                .flatten()
        })
        .ok_or_else(|| "no branch portal at player position".to_string())?;
    if !world.levels.contains(to) {
        return Err("target branch level does not exist".into());
    }
    transfer(world, to, true)
}

pub fn enter_planes(world: &mut GameWorld) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("planes entry requires campaign mode".into());
    }
    let to = LevelId::new(aihack_core::ids::BranchId::Planes, 1);
    if !world.levels.contains(to) {
        return Err("Plane of Earth does not exist".into());
    }
    transfer(world, to, true)
}

fn transfer(world: &mut GameWorld, to: LevelId, up: bool) -> Result<Vec<GameEvent>, String> {
    let from = world.current_level();
    let from_pos = world.player_pos();
    let landing = if up {
        world.levels.stairs_up_pos(to)
    } else {
        world.levels.stairs_down_pos(to)
    }
    .ok_or("target landing is missing")?;
    let mut events = crate::systems::npc::check_shop_theft_on_move_or_stairs(world, true);
    world.set_player_location(to, landing);
    events.push(GameEvent::LevelChanged {
        entity: world.player_id,
        from,
        to,
    });
    events.extend(crate::systems::pets::follow_player_on_level_change(
        world, from, from_pos, to, landing,
    ));
    Ok(events)
}

/// 현재 위치의 아래층 계단을 통해 고정된 대상 레벨로 이동한다.
pub fn descend(world: &mut GameWorld) -> Result<Vec<GameEvent>, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    match world.current_map().tile(player_pos) {
        Ok(TileKind::StairsDown) => {}
        Ok(_) => return Err("player is not standing on stairs down".to_string()),
        Err(error) => return Err(format!("cannot inspect stairs down tile: {error}")),
    }

    let to = if world.campaign.is_some() {
        let graph = BranchGraph::campaign_v3();
        graph
            .descend_target(from)
            .or_else(|| graph.branch_entry_from(from, TileKind::StairsDown))
            .ok_or_else(|| "target level for stairs down does not exist".to_string())?
    } else {
        let target_depth = from
            .depth
            .checked_add(1)
            .ok_or_else(|| "target level depth overflows".to_string())?;
        LevelId {
            branch: from.branch,
            depth: target_depth,
        }
    };
    if !world.levels.contains(to) {
        return Err("target level for stairs down does not exist".to_string());
    }
    let landing = world
        .levels
        .stairs_up_pos(to)
        .ok_or_else(|| "target level has no stairs up landing".to_string())?;
    let mut events = crate::systems::npc::check_shop_theft_on_move_or_stairs(world, true);
    world.set_player_location(to, landing);
    events.push(GameEvent::LevelChanged {
        entity: world.player_id,
        from,
        to,
    });
    events.extend(crate::systems::pets::follow_player_on_level_change(
        world, from, player_pos, to, landing,
    ));
    Ok(events)
}

/// 현재 위치의 위층 계단이나 매직 포털을 통해 이전/상위 레벨로 이동한다.
pub fn ascend(world: &mut GameWorld) -> Result<Vec<GameEvent>, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    let tile = world
        .current_map()
        .tile(player_pos)
        .map_err(|error| format!("cannot inspect stairs up tile: {error}"))?;
    match tile {
        TileKind::StairsUp => {}
        TileKind::MagicPortal
            if world.campaign.is_some() && from.branch == aihack_core::ids::BranchId::Planes => {}
        _ => return Err("player is not standing on stairs up".to_string()),
    }

    if world.campaign.is_some() {
        let graph = BranchGraph::campaign_v3();
        if from.branch == aihack_core::ids::BranchId::Planes {
            if tile == TileKind::MagicPortal {
                if let Some(to) = graph.portal_target(from) {
                    if !world.levels.contains(to) {
                        return Err("target plane level does not exist".to_string());
                    }
                    return transfer(world, to, true);
                }
            }
            return Err(if from.depth == 5 {
                "offer the Amulet at your high altar to ascend".to_string()
            } else {
                "step onto the magic portal to ascend further".to_string()
            });
        }
        if let Some(to) = graph.ascend_target(from) {
            let returning_from_branch_root = from.depth == 1
                && graph
                    .branch_def(from.branch)
                    .and_then(|def| def.parent)
                    .is_some();
            if returning_from_branch_root {
                let landing_up = graph
                    .has_branch_entry(to, aihack_core::domain::level::BranchEntryKind::StairsUp);
                return transfer(world, to, landing_up);
            }
            if !world.levels.contains(to) {
                return Err("target level for stairs up does not exist".to_string());
            }
            return transfer(world, to, false);
        }
        return Err("cannot ascend above main:1".to_string());
    }
    if from.depth <= 1 {
        return Err("cannot ascend above main:1".to_string());
    }
    let target_depth = from
        .depth
        .checked_sub(1)
        .ok_or_else(|| "target level depth underflows".to_string())?;
    let to = LevelId {
        branch: from.branch,
        depth: target_depth,
    };
    if !world.levels.contains(to) {
        return Err("target level for stairs up does not exist".to_string());
    }
    transfer(world, to, false)
}
