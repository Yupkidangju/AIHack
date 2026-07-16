use crate::{domain::map::GameMap, position::Pos};

pub const DEFAULT_VISION_RADIUS: i16 = 8;

pub fn visible_positions(map: &GameMap, origin: Pos) -> Vec<Pos> {
    let mut out = Vec::new();
    for y in (origin.y - DEFAULT_VISION_RADIUS)..=(origin.y + DEFAULT_VISION_RADIUS) {
        for x in (origin.x - DEFAULT_VISION_RADIUS)..=(origin.x + DEFAULT_VISION_RADIUS) {
            let pos = Pos { x, y };
            if !map.contains(pos) || origin.chebyshev_distance(pos) > DEFAULT_VISION_RADIUS {
                continue;
            }
            if has_line_of_sight(map, origin, pos) {
                out.push(pos);
            }
        }
    }
    out
}

pub fn has_line_of_sight(map: &GameMap, from: Pos, to: Pos) -> bool {
    if from == to {
        return true;
    }
    if !map.contains(to) {
        return false;
    }
    for pos in bresenham_line(from, to).into_iter().skip(1) {
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

fn bresenham_line(from: Pos, to: Pos) -> Vec<Pos> {
    let mut points = Vec::new();
    let (mut x0, mut y0) = (from.x, from.y);
    let (x1, y1) = (to.x, to.y);
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
