use aihack_core::{
    domain::tile::{DoorState, TileKind},
    error::GameError,
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
};

use crate::world::GameWorld;

pub fn move_player(world: &mut GameWorld, direction: Direction) -> Result<(), GameError> {
    if world.carried_weight() > world.carrying_capacity() {
        return Err(GameError::CommandRejected(
            "movement blocked by encumbrance".to_string(),
        ));
    }
    move_actor(world, world.player_id, direction)
}

pub fn move_actor(
    world: &mut GameWorld,
    actor: EntityId,
    direction: Direction,
) -> Result<(), GameError> {
    let (level, from) = actor_origin(world, actor)?;
    let to = from.offset(direction.delta());
    aihack_core::movement::validate_actor_destination(world, actor, direction)?;
    if actor != world.player_id {
        if let Ok(tile) = world.map(level).tile(to) {
            if !crate::systems::terrain::actor_kind_can_traverse(
                world.entities.get(actor).and_then(|e| e.actor_kind()),
                tile,
            ) {
                return Err(GameError::CommandRejected(
                    "monster avoids hazardous terrain".to_string(),
                ));
            }
        }
    }

    if actor == world.player_id {
        world.set_player_location(level, to);
    } else if !world
        .state_mut()
        .entities
        .set_actor_location(actor, level, to)
    {
        return Err(GameError::CommandRejected(format!(
            "actor {actor:?} position update failed"
        )));
    }
    Ok(())
}

pub fn is_passable_for_actor(world: &GameWorld, actor: EntityId, direction: Direction) -> bool {
    if !aihack_core::movement::is_passable_for_actor(world, actor, direction) {
        return false;
    }
    if actor != world.player_id {
        if let Some((level, from)) = world.entities.actor_location(actor) {
            let to = from.offset(direction.delta());
            if let Ok(tile) = world.map(level).tile(to) {
                if !crate::systems::terrain::actor_kind_can_traverse(
                    world.entities.get(actor).and_then(|e| e.actor_kind()),
                    tile,
                ) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn is_passable_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    world.carried_weight() <= world.carrying_capacity()
        && is_passable_for_actor(world, world.player_id, direction)
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
            | TileKind::Door(DoorState::Open)
            | TileKind::Trap(_)
            | TileKind::HiddenTrap(_)
            | TileKind::StairsDown
            | TileKind::StairsUp
            | TileKind::Shop
            | TileKind::Altar)
    ) && aihack_core::movement::validate_path(world, actor, level, from, direction).is_ok()
}
