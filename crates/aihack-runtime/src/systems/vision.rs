use aihack_core::{
    domain::{entity::ActorKind, status::Resistances},
    ids::{EntityId, LevelId},
    position::Pos,
};

use crate::world::GameWorld;

pub use aihack_core::vision::DEFAULT_VISION_RADIUS;

pub const DARK_VISION_RADIUS: i16 = 2;

pub fn visible_positions(world: &GameWorld) -> Vec<Pos> {
    if world.status_timers.blind > 0 || crate::systems::tools::is_player_blindfolded(world) {
        return vec![world.player_pos()];
    }
    let origin = world.player_pos();
    let level = world.current_level();
    let max_radius = if world.campaign.is_some()
        && !crate::systems::tools::has_active_light_source(world)
        && world
            .levels
            .get(level)
            .is_some_and(|lvl| !lvl.is_lit(origin))
    {
        DARK_VISION_RADIUS
    } else {
        DEFAULT_VISION_RADIUS
    };
    visible_positions_on_level(world, level, origin)
        .into_iter()
        .filter(|pos| origin.chebyshev_distance(*pos) <= max_radius)
        .collect()
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
    let Some((ActorKind::Monster(mk), _, monster_level, monster_pos, stats, alive)) =
        entity.actor()
    else {
        return false;
    };
    if !alive {
        return false;
    }
    let (player_level, player_pos) = world.player_location();
    if monster_level != player_level {
        return false;
    }
    let dist = monster_pos.chebyshev_distance(player_pos);
    let sees_invis = stats.resistances.contains(Resistances::SEE_INVISIBLE)
        || aihack_content::monster_has_ai_flag(mk, "see_invisible", world.content_registry());
    if world.is_invisible() && !sees_invis && dist > 1 {
        return false;
    }
    dist <= DEFAULT_VISION_RADIUS
        && has_line_of_sight_on_level(world, monster_level, monster_pos, player_pos)
}

pub fn is_visible(world: &GameWorld, pos: Pos) -> bool {
    if world.status_timers.blind > 0 || crate::systems::tools::is_player_blindfolded(world) {
        return pos == world.player_pos();
    }
    is_visible_from(world, world.player_pos(), pos)
}

pub fn is_visible_from(world: &GameWorld, from: Pos, pos: Pos) -> bool {
    from.chebyshev_distance(pos) <= DEFAULT_VISION_RADIUS && has_line_of_sight(world, from, pos)
}
