use crate::{
    core::{
        error::GameError,
        ids::{EntityId, LevelId},
        position::{Direction, Pos},
        world::GameWorld,
    },
    domain::tile::TileKind,
};

pub fn move_player(world: &mut GameWorld, direction: Direction) -> Result<(), GameError> {
    if world.carried_weight() > 80 {
        return Err(GameError::CommandRejected(
            "movement blocked by encumbrance".to_string(),
        ));
    }
    move_actor(world, world.player_id, direction)
}

/// [v0.1.0] Phase 6 actor 공용 이동 helper다.
/// 기존 player 이동 규칙을 monster에도 재사용하기 위해 entity/level 기반으로 일반화한다.
pub fn move_actor(
    world: &mut GameWorld,
    actor: EntityId,
    direction: Direction,
) -> Result<(), GameError> {
    let (level, from) = actor_origin(world, actor)?;
    let to = from.offset(direction.delta());
    validate_actor_destination_on_level(world, actor, level, from, to, direction)?;

    if actor == world.player_id {
        world.set_player_location(level, to);
    } else if !world.entities.set_actor_location(actor, level, to) {
        return Err(GameError::CommandRejected(format!(
            "actor {actor:?} position update failed"
        )));
    }

    Ok(())
}

pub fn validate_actor_destination(
    world: &GameWorld,
    actor: EntityId,
    from: Pos,
    to: Pos,
    direction: Direction,
) -> Result<(), GameError> {
    let (level, _) = actor_origin(world, actor)?;
    validate_actor_destination_on_level(world, actor, level, from, to, direction)
}

/// [v0.1.0] Phase 6 helper: actor가 속한 level map 기준으로 이동 legality를 판정한다.
pub fn validate_actor_destination_on_level(
    world: &GameWorld,
    actor: EntityId,
    level: LevelId,
    from: Pos,
    to: Pos,
    direction: Direction,
) -> Result<(), GameError> {
    let tile = world.map(level).tile(to)?;
    if !tile.is_movement_passable() {
        return Err(GameError::BlockedMovement { pos: to, tile });
    }
    if occupied_by_other_actor(world, actor, level, to) {
        return Err(GameError::CommandRejected(format!(
            "movement blocked by living entity at {to:?}"
        )));
    }
    if let Some((a, b)) = direction.orthogonal_components() {
        validate_intermediate_step(world, actor, level, from.offset(a.delta()))?;
        validate_intermediate_step(world, actor, level, from.offset(b.delta()))?;
    }
    Ok(())
}

/// [v0.1.0] Phase 6 helper: player 외 actor도 동일한 이동 legality를 질의한다.
pub fn is_passable_for_actor(world: &GameWorld, actor: EntityId, direction: Direction) -> bool {
    let Ok((level, from)) = actor_origin(world, actor) else {
        return false;
    };
    let to = from.offset(direction.delta());
    if occupied_by_other_actor(world, actor, level, to) {
        return false;
    }
    is_walkable_or_attackable_destination(world, actor, level, from, to, direction)
}

pub fn is_passable_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    is_passable_for_actor(world, world.player_id, direction)
}

pub fn is_bump_attack_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    world
        .entities
        .alive_hostile_at(world.current_level(), to)
        .is_some()
        && is_walkable_or_attackable_destination(
            world,
            world.player_id,
            world.current_level(),
            from,
            to,
            direction,
        )
}

fn actor_origin(world: &GameWorld, actor: EntityId) -> Result<(LevelId, Pos), GameError> {
    world
        .entities
        .actor_location(actor)
        .ok_or_else(|| GameError::CommandRejected(format!("actor {actor:?} has no map position")))
}

fn validate_intermediate_step(
    world: &GameWorld,
    actor: EntityId,
    level: LevelId,
    pos: Pos,
) -> Result<(), GameError> {
    let tile = world.map(level).tile(pos)?;
    if !tile.is_movement_passable() {
        return Err(GameError::BlockedMovement { pos, tile });
    }
    if occupied_by_other_actor(world, actor, level, pos) {
        return Err(GameError::CommandRejected(format!(
            "movement blocked by living entity at {pos:?}"
        )));
    }
    Ok(())
}

fn is_walkable_or_attackable_destination(
    world: &GameWorld,
    actor: EntityId,
    level: LevelId,
    from: Pos,
    to: Pos,
    direction: Direction,
) -> bool {
    matches!(
        world.map(level).tile(to),
        Ok(TileKind::Floor
            | TileKind::Door(crate::domain::tile::DoorState::Open)
            | TileKind::StairsDown
            | TileKind::StairsUp)
    ) && direction
        .orthogonal_components()
        .map(|(a, b)| {
            [a, b].into_iter().all(|component| {
                let pos = from.offset(component.delta());
                !occupied_by_other_actor(world, actor, level, pos)
                    && world
                        .map(level)
                        .tile(pos)
                        .map(TileKind::is_movement_passable)
                        .unwrap_or(false)
            })
        })
        .unwrap_or(true)
}

fn occupied_by_other_actor(world: &GameWorld, actor: EntityId, level: LevelId, pos: Pos) -> bool {
    world
        .entities
        .alive_entity_at(level, pos)
        .is_some_and(|occupant| occupant != actor)
}
