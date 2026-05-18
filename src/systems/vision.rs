use crate::{
    core::{
        ids::{EntityId, LevelId},
        position::Pos,
        world::GameWorld,
    },
    domain::entity::ActorKind,
};

pub const DEFAULT_VISION_RADIUS: i16 = 8;

pub fn visible_positions(world: &GameWorld) -> Vec<Pos> {
    visible_positions_from(world, world.player_pos())
}

pub fn visible_positions_from(world: &GameWorld, origin: Pos) -> Vec<Pos> {
    visible_positions_on_level(world, world.current_level(), origin)
}

/// [v0.1.0] Phase 6 helper: 특정 level의 임의 origin 기준 visible tile을 구한다.
pub fn visible_positions_on_level(world: &GameWorld, level: LevelId, origin: Pos) -> Vec<Pos> {
    let map = world.map(level);
    let mut out = Vec::new();

    for y in (origin.y - DEFAULT_VISION_RADIUS)..=(origin.y + DEFAULT_VISION_RADIUS) {
        for x in (origin.x - DEFAULT_VISION_RADIUS)..=(origin.x + DEFAULT_VISION_RADIUS) {
            let pos = Pos { x, y };
            if !map.contains(pos) || origin.chebyshev_distance(pos) > DEFAULT_VISION_RADIUS {
                continue;
            }
            if has_line_of_sight_on_level(world, level, origin, pos) {
                out.push(pos);
            }
        }
    }

    out
}

pub fn has_line_of_sight(world: &GameWorld, from: Pos, to: Pos) -> bool {
    has_line_of_sight_on_level(world, world.current_level(), from, to)
}

/// [v0.1.0] Phase 6 helper: 특정 level map 기준 LOS를 판정한다.
pub fn has_line_of_sight_on_level(world: &GameWorld, level: LevelId, from: Pos, to: Pos) -> bool {
    if from == to {
        return true;
    }

    let map = world.map(level);
    if !map.contains(to) {
        return false;
    }

    let line = bresenham_line(from, to);
    for pos in line.iter().copied().skip(1) {
        let Ok(tile) = map.tile(pos) else {
            return false;
        };
        if pos == to {
            return true;
        }
        if !tile.is_los_transparent() {
            return false;
        }
    }
    true
}

/// [v0.1.0] Phase 6 helper: monster origin에서 현재 player가 LOS 안에 있는지 판정한다.
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

fn bresenham_line(from: Pos, to: Pos) -> Vec<Pos> {
    let mut points = Vec::new();
    let mut x0 = from.x;
    let mut y0 = from.y;
    let x1 = to.x;
    let y1 = to.y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(Pos { x: x0, y: y0 });
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

pub fn is_visible(world: &GameWorld, pos: Pos) -> bool {
    is_visible_from(world, world.player_pos(), pos)
}

pub fn is_visible_from(world: &GameWorld, from: Pos, pos: Pos) -> bool {
    from.chebyshev_distance(pos) <= DEFAULT_VISION_RADIUS && has_line_of_sight(world, from, pos)
}
