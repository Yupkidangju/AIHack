use aihack_core::{
    domain::entity::ActorKind,
    ids::{EntityId, LevelId},
    position::Pos,
};

use crate::world::GameWorld;

pub use aihack_core::vision::DEFAULT_VISION_RADIUS;

pub fn visible_positions(world: &GameWorld) -> Vec<Pos> {
    visible_positions_from(world, world.player_pos())
}

pub fn visible_positions_from(world: &GameWorld, origin: Pos) -> Vec<Pos> {
    visible_positions_on_level(world, world.current_level(), origin)
}

pub fn visible_positions_on_level(world: &GameWorld, level: LevelId, origin: Pos) -> Vec<Pos> {
    aihack_core::vision::visible_positions(world.map(level), origin)
}

pub fn has_line_of_sight(world: &GameWorld, from: Pos, to: Pos) -> bool {
    has_line_of_sight_on_level(world, world.current_level(), from, to)
}

pub fn has_line_of_sight_on_level(world: &GameWorld, level: LevelId, from: Pos, to: Pos) -> bool {
    aihack_core::vision::has_line_of_sight(world.map(level), from, to)
}

pub fn monster_has_line_of_sight_to_player(world: &GameWorld, monster: EntityId) -> bool {
    let Some(entity) = world.entities.get(monster) else {
        return false;
    };
    let Some((ActorKind::Monster(_), _, monster_level, monster_pos, _, alive)) = entity.actor()
    else {
        return false;
    };
    if !alive {
        return false;
    }
    let (player_level, player_pos) = world.player_location();
    monster_level == player_level
        && monster_pos.chebyshev_distance(player_pos) <= DEFAULT_VISION_RADIUS
        && has_line_of_sight_on_level(world, monster_level, monster_pos, player_pos)
}

pub fn is_visible(world: &GameWorld, pos: Pos) -> bool {
    is_visible_from(world, world.player_pos(), pos)
}

pub fn is_visible_from(world: &GameWorld, from: Pos, pos: Pos) -> bool {
    from.chebyshev_distance(pos) <= DEFAULT_VISION_RADIUS && has_line_of_sight(world, from, pos)
}
