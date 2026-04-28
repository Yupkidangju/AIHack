use crate::{
    core::{error::GameError, position::Direction, world::GameWorld},
    domain::tile::TileKind,
};

pub fn move_player(world: &mut GameWorld, direction: Direction) -> Result<(), GameError> {
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    validate_destination(world, to)?;

    if let Some((a, b)) = direction.orthogonal_components() {
        validate_destination(world, from.offset(a.delta()))?;
        validate_destination(world, from.offset(b.delta()))?;
    }

    world.set_player_pos(to);
    Ok(())
}

fn validate_destination(
    world: &GameWorld,
    pos: crate::core::position::Pos,
) -> Result<(), GameError> {
    let tile = world.current_map().tile(pos)?;
    if !tile.is_movement_passable() {
        return Err(GameError::BlockedMovement { pos, tile });
    }
    if world
        .entities
        .alive_entity_at(world.current_level(), pos)
        .is_some()
    {
        return Err(GameError::CommandRejected(format!(
            "movement blocked by living entity at {pos:?}"
        )));
    }
    Ok(())
}

pub fn is_passable_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    if world
        .entities
        .alive_entity_at(world.current_level(), to)
        .is_some()
    {
        return false;
    }
    is_walkable_or_attackable_destination(world, from, to, direction)
}

pub fn is_bump_attack_for_legal_action(world: &GameWorld, direction: Direction) -> bool {
    let from = world.player_pos();
    let to = from.offset(direction.delta());
    world
        .entities
        .alive_hostile_at(world.current_level(), to)
        .is_some()
        && is_walkable_or_attackable_destination(world, from, to, direction)
}

fn is_walkable_or_attackable_destination(
    world: &GameWorld,
    from: crate::core::position::Pos,
    to: crate::core::position::Pos,
    direction: Direction,
) -> bool {
    matches!(
        world.current_map().tile(to),
        Ok(TileKind::Floor
            | TileKind::Door(crate::domain::tile::DoorState::Open)
            | TileKind::StairsDown
            | TileKind::StairsUp)
    ) && direction
        .orthogonal_components()
        .map(|(a, b)| {
            [a, b].into_iter().all(|component| {
                let pos = from.offset(component.delta());
                world
                    .entities
                    .alive_entity_at(world.current_level(), pos)
                    .is_none()
                    && world
                        .current_map()
                        .tile(pos)
                        .map(TileKind::is_movement_passable)
                        .unwrap_or(false)
            })
        })
        .unwrap_or(true)
}
