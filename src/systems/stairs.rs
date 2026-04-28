use crate::{
    core::{event::GameEvent, ids::LevelId, world::GameWorld},
    domain::tile::TileKind,
};

/// [v0.1.0] 현재 위치의 아래층 계단을 통해 fixed target level로 이동한다.
pub fn descend(world: &mut GameWorld) -> Result<GameEvent, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    match world.current_map().tile(player_pos) {
        Ok(TileKind::StairsDown) => {}
        Ok(_) => return Err("player is not standing on stairs down".to_string()),
        Err(error) => return Err(format!("cannot inspect stairs down tile: {error}")),
    }

    let to = LevelId {
        branch: from.branch,
        depth: from.depth + 1,
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

/// [v0.1.0] 현재 위치의 위층 계단을 통해 fixed previous level로 이동한다.
pub fn ascend(world: &mut GameWorld) -> Result<GameEvent, String> {
    let from = world.current_level();
    let player_pos = world.player_pos();
    match world.current_map().tile(player_pos) {
        Ok(TileKind::StairsUp) => {}
        Ok(_) => return Err("player is not standing on stairs up".to_string()),
        Err(error) => return Err(format!("cannot inspect stairs up tile: {error}")),
    }

    if from.depth <= 1 {
        return Err("cannot ascend above main:1".to_string());
    }
    let to = LevelId {
        branch: from.branch,
        depth: from.depth - 1,
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
