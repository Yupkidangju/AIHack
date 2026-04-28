use crate::core::{position::Pos, world::GameWorld};

pub const DEFAULT_VISION_RADIUS: i16 = 8;

pub fn visible_positions(world: &GameWorld) -> Vec<Pos> {
    let origin = world.player_pos();
    let mut out = Vec::new();

    for y in (origin.y - DEFAULT_VISION_RADIUS)..=(origin.y + DEFAULT_VISION_RADIUS) {
        for x in (origin.x - DEFAULT_VISION_RADIUS)..=(origin.x + DEFAULT_VISION_RADIUS) {
            let pos = Pos { x, y };
            if !world.current_map().contains(pos)
                || origin.chebyshev_distance(pos) > DEFAULT_VISION_RADIUS
            {
                continue;
            }
            if has_line_of_sight(world, origin, pos) {
                out.push(pos);
            }
        }
    }

    out
}

pub fn has_line_of_sight(world: &GameWorld, from: Pos, to: Pos) -> bool {
    if from == to {
        return true;
    }
    if !world.current_map().contains(to) {
        return false;
    }

    let line = bresenham_line(from, to);
    for pos in line.iter().copied().skip(1) {
        let Ok(tile) = world.current_map().tile(pos) else {
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
    world.player_pos().chebyshev_distance(pos) <= DEFAULT_VISION_RADIUS
        && has_line_of_sight(world, world.player_pos(), pos)
}
