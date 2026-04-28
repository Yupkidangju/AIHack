// ============================================================================
// [v2.40.0 R28-5] 방향/이동 유틸 (direction_ext.rs)
// 원본: NetHack 3.6.7 cmd.c/hack.c 방향 확장
// 8방향, 벡터, 방향 문자열, 반대 방향
// ============================================================================

/// [v2.40.0 R28-5] 8방향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

pub fn direction_delta(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::N => (0, -1),
        Direction::S => (0, 1),
        Direction::E => (1, 0),
        Direction::W => (-1, 0),
        Direction::NE => (1, -1),
        Direction::NW => (-1, -1),
        Direction::SE => (1, 1),
        Direction::SW => (-1, 1),
    }
}

pub fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::N => Direction::S,
        Direction::S => Direction::N,
        Direction::E => Direction::W,
        Direction::W => Direction::E,
        Direction::NE => Direction::SW,
        Direction::NW => Direction::SE,
        Direction::SE => Direction::NW,
        Direction::SW => Direction::NE,
    }
}

pub fn direction_name(dir: Direction) -> &'static str {
    match dir {
        Direction::N => "north",
        Direction::S => "south",
        Direction::E => "east",
        Direction::W => "west",
        Direction::NE => "northeast",
        Direction::NW => "northwest",
        Direction::SE => "southeast",
        Direction::SW => "southwest",
    }
}

/// [v2.40.0 R28-5] 좌표→방향
pub fn pos_to_direction(dx: i32, dy: i32) -> Option<Direction> {
    match (dx.signum(), dy.signum()) {
        (0, -1) => Some(Direction::N),
        (0, 1) => Some(Direction::S),
        (1, 0) => Some(Direction::E),
        (-1, 0) => Some(Direction::W),
        (1, -1) => Some(Direction::NE),
        (-1, -1) => Some(Direction::NW),
        (1, 1) => Some(Direction::SE),
        (-1, 1) => Some(Direction::SW),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta() {
        assert_eq!(direction_delta(Direction::NE), (1, -1));
    }

    #[test]
    fn test_opposite() {
        assert_eq!(opposite(Direction::N), Direction::S);
        assert_eq!(opposite(Direction::NE), Direction::SW);
    }

    #[test]
    fn test_name() {
        assert_eq!(direction_name(Direction::NW), "northwest");
    }

    #[test]
    fn test_pos_to_dir() {
        assert_eq!(pos_to_direction(3, -2), Some(Direction::NE));
        assert_eq!(pos_to_direction(0, 0), None);
    }
}
