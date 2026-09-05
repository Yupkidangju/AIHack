use aihack_core::{domain::tile::TileKind, event::GameEvent, ids::LevelId};

use crate::world::GameWorld;

pub fn enter_branch(world: &mut GameWorld) -> Result<GameEvent, String> {
    if world.campaign.is_none()
        || world.current_level() != LevelId::main(3)
        || world.current_map().tile(world.player_pos()).ok() != Some(TileKind::StairsUp)
    {
        return Err("branch entrance is at Main 3 stairs up".into());
    }
    transfer(
        world,
        LevelId {
            branch: aihack_core::ids::BranchId::Mines,
            depth: 1,
        },
        true,
    )
}

fn transfer(world: &mut GameWorld, to: LevelId, up: bool) -> Result<GameEvent, String> {
    let from = world.current_level();
    let landing = if up {
        world.levels.stairs_up_pos(to)
    } else {
        world.levels.stairs_down_pos(to)
    }
    .ok_or("target landing is missing")?;
    world.set_player_location(to, landing);
    Ok(GameEvent::LevelChanged {
        entity: world.player_id,
        from,
        to,
    })
}

/// 현재 위치의 아래층 계단을 통해 고정된 대상 레벨로 이동한다.
pub fn descend(world: &mut GameWorld) -> Result<GameEvent, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    match world.current_map().tile(player_pos) {
        Ok(TileKind::StairsDown) => {}
        Ok(_) => return Err("player is not standing on stairs down".to_string()),
        Err(error) => return Err(format!("cannot inspect stairs down tile: {error}")),
    }

    let target_depth = from
        .depth
        .checked_add(1)
        .ok_or_else(|| "target level depth overflows".to_string())?;
    let to = LevelId {
        branch: from.branch,
        depth: target_depth,
    };
    if !world.levels.contains(to) {
        return Err("target level for stairs down does not exist".to_string());
    }
    let landing = world
        .levels
        .stairs_up_pos(to)
        .ok_or_else(|| "target level has no stairs up landing".to_string())?;
    world.set_player_location(to, landing);
    Ok(GameEvent::LevelChanged {
        entity: world.player_id,
        from,
        to,
    })
}

/// 현재 위치의 위층 계단을 통해 고정된 이전 레벨로 이동한다.
pub fn ascend(world: &mut GameWorld) -> Result<GameEvent, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    match world.current_map().tile(player_pos) {
        Ok(TileKind::StairsUp) => {}
        Ok(_) => return Err("player is not standing on stairs up".to_string()),
        Err(error) => return Err(format!("cannot inspect stairs up tile: {error}")),
    }

    if world.campaign.is_some()
        && from
            == (LevelId {
                branch: aihack_core::ids::BranchId::Mines,
                depth: 1,
            })
    {
        return transfer(world, LevelId::main(3), true);
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
    let landing = world
        .levels
        .stairs_down_pos(to)
        .ok_or_else(|| "target level has no stairs down landing".to_string())?;
    world.set_player_location(to, landing);
    Ok(GameEvent::LevelChanged {
        entity: world.player_id,
        from,
        to,
    })
}
