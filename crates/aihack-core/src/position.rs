use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Delta {
    pub dx: i16,
    pub dy: i16,
}

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
        Self::North,
        Self::South,
        Self::West,
        Self::East,
        Self::NorthWest,
        Self::NorthEast,
        Self::SouthWest,
        Self::SouthEast,
    ];
    pub fn delta(self) -> Delta {
        match self {
            Self::North => Delta { dx: 0, dy: -1 },
            Self::South => Delta { dx: 0, dy: 1 },
            Self::West => Delta { dx: -1, dy: 0 },
            Self::East => Delta { dx: 1, dy: 0 },
            Self::NorthWest => Delta { dx: -1, dy: -1 },
            Self::NorthEast => Delta { dx: 1, dy: -1 },
            Self::SouthWest => Delta { dx: -1, dy: 1 },
            Self::SouthEast => Delta { dx: 1, dy: 1 },
        }
    }
    pub fn is_diagonal(self) -> bool {
        let delta = self.delta();
        delta.dx != 0 && delta.dy != 0
    }
    pub fn orthogonal_components(self) -> Option<(Direction, Direction)> {
        match self {
            Self::NorthWest => Some((Self::North, Self::West)),
            Self::NorthEast => Some((Self::North, Self::East)),
            Self::SouthWest => Some((Self::South, Self::West)),
            Self::SouthEast => Some((Self::South, Self::East)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delta, Direction};
    #[test]
    fn direction_delta_matches_grid_contract() {
        assert_eq!(Direction::North.delta(), Delta { dx: 0, dy: -1 });
        assert_eq!(Direction::SouthEast.delta(), Delta { dx: 1, dy: 1 });
    }
}
