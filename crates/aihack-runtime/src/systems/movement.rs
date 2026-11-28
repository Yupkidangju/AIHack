use aihack_core::{
    domain::tile::TileKind,
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

pub(crate) fn sokoban_unplugged_pit_at(world: &GameWorld, level: LevelId, pos: Pos) -> bool {
    level.branch == aihack_core::ids::BranchId::Sokoban
        && matches!(
            world.map(level).tile(pos),
            Ok(TileKind::Trap(aihack_core::domain::tile::TrapKind::Pit)
                | TileKind::HiddenTrap(aihack_core::domain::tile::TrapKind::Pit))
        )
}

fn validate_sokoban_pit_step(
    world: &GameWorld,
    level: LevelId,
    from: Pos,
    to: Pos,
    direction: Direction,
) -> Result<(), GameError> {
    if sokoban_unplugged_pit_at(world, level, to) {
        return Err(GameError::CommandRejected(
            "an open Sokoban pit blocks your way; push a boulder into it to cross".to_string(),
        ));
    }
    if let Some((a, b)) = direction.orthogonal_components() {
        for component in [a, b] {
            let mid = from.offset(component.delta());
            if sokoban_unplugged_pit_at(world, level, mid) {
                return Err(GameError::CommandRejected(
                    "an open Sokoban pit blocks diagonal passage".to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn move_actor(
    world: &mut GameWorld,
    actor: EntityId,
    direction: Direction,
) -> Result<(), GameError> {
    let (level, from) = actor_origin(world, actor)?;
    let to = from.offset(direction.delta());
    validate_sokoban_pit_step(world, level, from, to, direction)?;
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
    if let Some((level, from)) = world.entities.actor_location(actor) {
        let to = from.offset(direction.delta());
        if validate_sokoban_pit_step(world, level, from, to, direction).is_err() {
            return false;
        }
    }
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

pub fn is_friendly_swap_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    if world.campaign.is_none() || world.carried_weight() > world.carrying_capacity() {
        return false;
    }
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    world
        .entities
        .alive_friendly_at(world.current_level(), to)
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

pub fn swap_player_with_friendly(
    world: &mut GameWorld,
    friendly_id: EntityId,
    direction: Direction,
) -> Result<(Pos, Pos), GameError> {
    if world.carried_weight() > world.carrying_capacity() {
        return Err(GameError::CommandRejected(
            "movement blocked by encumbrance".to_string(),
        ));
    }
    let (level, from) = actor_origin(world, world.player_id)?;
    let to = from.offset(direction.delta());
    if !is_walkable_or_attackable_destination(world, world.player_id, level, from, to, direction) {
        return Err(GameError::CommandRejected(
            "cannot swap into blocked destination".to_string(),
        ));
    }
    if world.entities.alive_friendly_at(level, to) != Some(friendly_id) {
        return Err(GameError::CommandRejected(
            "friendly actor is not at destination".to_string(),
        ));
    }
    world.set_player_location(level, to);
    world
        .state_mut()
        .entities
        .set_actor_location(friendly_id, level, from);
    Ok((from, to))
}

pub fn is_boulder_push_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    if world.campaign.is_none() || world.carried_weight() > world.carrying_capacity() {
        return false;
    }
    let level = world.current_level();
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    if world.map(level).tile(to).ok() != Some(TileKind::Boulder) {
        return false;
    }
    if world.entities.alive_actor_at(level, to).is_some() {
        return false;
    }
    if aihack_core::movement::validate_path(world, world.player_id, level, from, direction).is_err()
    {
        return false;
    }
    let beyond = to.offset(direction.delta());
    if world.entities.alive_actor_at(level, beyond).is_some() {
        return false;
    }
    matches!(
        world.map(level).tile(beyond),
        Ok(TileKind::Floor
            | TileKind::Trap(aihack_core::domain::tile::TrapKind::Pit)
            | TileKind::HiddenTrap(aihack_core::domain::tile::TrapKind::Pit))
    )
}

pub fn push_boulder(
    world: &mut GameWorld,
    direction: Direction,
) -> Result<(Pos, Pos, bool), GameError> {
    use aihack_core::domain::tile::TrapKind;
    if world.campaign.is_none() {
        return Err(GameError::CommandRejected(
            "boulders can only be pushed in campaign mode".to_string(),
        ));
    }
    if world.carried_weight() > world.carrying_capacity() {
        return Err(GameError::CommandRejected(
            "movement blocked by encumbrance".to_string(),
        ));
    }
    let (level, from) = actor_origin(world, world.player_id)?;
    let to = from.offset(direction.delta());
    if world.map(level).tile(to).ok() != Some(TileKind::Boulder) {
        return Err(GameError::CommandRejected(
            "there is no boulder in that direction".to_string(),
        ));
    }
    if world.entities.alive_actor_at(level, to).is_some() {
        return Err(GameError::CommandRejected(
            "something is blocking the boulder".to_string(),
        ));
    }
    aihack_core::movement::validate_path(world, world.player_id, level, from, direction)?;
    let beyond = to.offset(direction.delta());
    if world.entities.alive_actor_at(level, beyond).is_some() {
        return Err(GameError::CommandRejected(
            "something is in the way of the boulder".to_string(),
        ));
    }
    let beyond_tile = world
        .map(level)
        .tile(beyond)
        .map_err(|e| GameError::CommandRejected(e.to_string()))?;
    let plugged = matches!(
        beyond_tile,
        TileKind::Trap(TrapKind::Pit) | TileKind::HiddenTrap(TrapKind::Pit)
    );
    if !plugged && beyond_tile != TileKind::Floor {
        return Err(GameError::CommandRejected(
            "the boulder cannot be pushed there".to_string(),
        ));
    }
    world
        .map_mut(level)
        .set_tile(to, TileKind::Floor)
        .map_err(|e| GameError::CommandRejected(e.to_string()))?;
    world
        .map_mut(level)
        .set_tile(
            beyond,
            if plugged {
                TileKind::Floor
            } else {
                TileKind::Boulder
            },
        )
        .map_err(|e| GameError::CommandRejected(e.to_string()))?;
    world.set_player_location(level, to);
    world.mark_current_level_explored();
    Ok((from, to, plugged))
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
    validate_sokoban_pit_step(world, level, from, to, direction).is_ok()
        && world
            .map(level)
            .tile(to)
            .is_ok_and(TileKind::is_movement_passable)
        && aihack_core::movement::validate_path(world, actor, level, from, direction).is_ok()
}
