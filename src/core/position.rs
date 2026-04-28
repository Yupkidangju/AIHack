use serde::{Deserialize, Serialize};

/// [v0.1.0] 던전 격자의 절대 좌표다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pos {
    pub x: i16,
    pub y: i16,
}

impl Pos {
    pub fn offset(self, delta: Delta) -> Self {
        Self {
            x: self.x + delta.dx,
            y: self.y + delta.dy,
        }
    }

    pub fn delta_to(self, other: Self) -> Delta {
        Delta {
            dx: other.x - self.x,
            dy: other.y - self.y,
        }
    }

    pub fn chebyshev_distance(self, other: Self) -> i16 {
        let delta = self.delta_to(other);
        delta.dx.abs().max(delta.dy.abs())
    }
}

/// [v0.1.0] 플레이어 기준 상대 좌표와 방향 계산에 쓰는 delta다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Delta {
    pub dx: i16,
    pub dy: i16,
}

/// [v0.1.0] Phase 2 이동/문 조작에서 사용하는 8방향 입력이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
        Direction::NorthWest,
        Direction::NorthEast,
        Direction::SouthWest,
        Direction::SouthEast,
    ];

    pub fn delta(self) -> Delta {
        match self {
            Direction::North => Delta { dx: 0, dy: -1 },
            Direction::South => Delta { dx: 0, dy: 1 },
            Direction::West => Delta { dx: -1, dy: 0 },
            Direction::East => Delta { dx: 1, dy: 0 },
            Direction::NorthWest => Delta { dx: -1, dy: -1 },
            Direction::NorthEast => Delta { dx: 1, dy: -1 },
            Direction::SouthWest => Delta { dx: -1, dy: 1 },
            Direction::SouthEast => Delta { dx: 1, dy: 1 },
        }
    }

    pub fn is_diagonal(self) -> bool {
        let delta = self.delta();
        delta.dx != 0 && delta.dy != 0
    }

    pub fn orthogonal_components(self) -> Option<(Direction, Direction)> {
        match self {
            Direction::NorthWest => Some((Direction::North, Direction::West)),
            Direction::NorthEast => Some((Direction::North, Direction::East)),
            Direction::SouthWest => Some((Direction::South, Direction::West)),
            Direction::SouthEast => Some((Direction::South, Direction::East)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delta, Direction};

    #[test]
    fn direction_delta_matches_grid_contract() {
        let cases = [
            (Direction::North, Delta { dx: 0, dy: -1 }),
            (Direction::South, Delta { dx: 0, dy: 1 }),
            (Direction::West, Delta { dx: -1, dy: 0 }),
            (Direction::East, Delta { dx: 1, dy: 0 }),
            (Direction::NorthWest, Delta { dx: -1, dy: -1 }),
            (Direction::NorthEast, Delta { dx: 1, dy: -1 }),
            (Direction::SouthWest, Delta { dx: -1, dy: 1 }),
            (Direction::SouthEast, Delta { dx: 1, dy: 1 }),
        ];

        for (direction, expected) in cases {
            assert_eq!(direction.delta(), expected);
        }
    }
}
