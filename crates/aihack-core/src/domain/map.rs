use serde::{Deserialize, Serialize};

use crate::{
    domain::tile::{DoorState, TileKind, TrapKind},
    error::{ContentError, GameError},
    position::Pos,
};

/// Content adapter가 core map 생성에 제공하는 검증된 layout projection이다.
pub trait MapLayout {
    fn level_id(&self) -> &str;
    fn depth(&self) -> i16;
    fn dimensions(&self) -> (i16, i16);
    fn tile_overrides(&self) -> Result<Vec<(Pos, TileKind)>, ContentError>;
}

pub const PHASE2_WIDTH: i16 = 40;
pub const PHASE2_HEIGHT: i16 = 20;
pub const PHASE2_PLAYER_START: Pos = Pos { x: 5, y: 5 };
pub const PHASE2_STAIRS_DOWN: Pos = Pos { x: 34, y: 15 };
pub const PHASE2_CLOSED_DOOR: Pos = Pos { x: 10, y: 5 };
pub const PHASE2_SECOND_CLOSED_DOOR: Pos = Pos { x: 14, y: 5 };
pub const PHASE7_HIDDEN_DOOR: Pos = Pos { x: 12, y: 5 };
pub const PHASE7_HIDDEN_TRAP: Pos = Pos { x: 16, y: 5 };
pub const PHASE5_LEVEL2_STAIRS_UP: Pos = Pos { x: 5, y: 5 };
pub const PHASE5_LEVEL2_CLOSED_DOOR: Pos = Pos { x: 8, y: 5 };

/// row-major tile map이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMap {
    pub width: i16,
    pub height: i16,
    tiles: Vec<TileKind>,
}

impl GameMap {
    /// ContentRegistry definition에서 deterministic tile map을 생성한다.
    pub fn from_level_data(level: &impl MapLayout) -> Result<Self, ContentError> {
        let (width, height) = level.dimensions();
        let mut map = Self {
            width,
            height,
            tiles: vec![TileKind::Floor; (i32::from(width) * i32::from(height)) as usize],
        };
        for (pos, tile) in level.tile_overrides()? {
            map.set_tile(pos, tile)
                .map_err(|_| ContentError::InvalidCoordinate {
                    level: level.level_id().to_owned(),
                    x: pos.x,
                    y: pos.y,
                })?;
        }
        Ok(map)
    }

    pub fn fixture_phase2() -> Self {
        let mut map = Self {
            width: PHASE2_WIDTH,
            height: PHASE2_HEIGHT,
            tiles: vec![TileKind::Floor; (PHASE2_WIDTH * PHASE2_HEIGHT) as usize],
        };

        for x in 0..PHASE2_WIDTH {
            map.set_tile_unchecked(Pos { x, y: 0 }, TileKind::Wall);
            map.set_tile_unchecked(
                Pos {
                    x,
                    y: PHASE2_HEIGHT - 1,
                },
                TileKind::Wall,
            );
        }
        for y in 0..PHASE2_HEIGHT {
            map.set_tile_unchecked(Pos { x: 0, y }, TileKind::Wall);
            map.set_tile_unchecked(
                Pos {
                    x: PHASE2_WIDTH - 1,
                    y,
                },
                TileKind::Wall,
            );
        }
        for y in 4..=8 {
            map.set_tile_unchecked(Pos { x: 12, y }, TileKind::Wall);
        }
        map.set_tile_unchecked(PHASE7_HIDDEN_DOOR, TileKind::HiddenDoor);

        map.set_tile_unchecked(PHASE2_CLOSED_DOOR, TileKind::Door(DoorState::Closed));
        map.set_tile_unchecked(PHASE2_SECOND_CLOSED_DOOR, TileKind::Door(DoorState::Closed));
        map.set_tile_unchecked(PHASE7_HIDDEN_TRAP, TileKind::HiddenTrap(TrapKind::Pit));
        map.set_tile_unchecked(PHASE2_STAIRS_DOWN, TileKind::StairsDown);
        map
    }

    pub fn fixture_phase5_level2() -> Self {
        let mut map = Self {
            width: PHASE2_WIDTH,
            height: PHASE2_HEIGHT,
            tiles: vec![TileKind::Floor; (PHASE2_WIDTH * PHASE2_HEIGHT) as usize],
        };

        for x in 0..PHASE2_WIDTH {
            map.set_tile_unchecked(Pos { x, y: 0 }, TileKind::Wall);
            map.set_tile_unchecked(
                Pos {
                    x,
                    y: PHASE2_HEIGHT - 1,
                },
                TileKind::Wall,
            );
        }
        for y in 0..PHASE2_HEIGHT {
            map.set_tile_unchecked(Pos { x: 0, y }, TileKind::Wall);
            map.set_tile_unchecked(
                Pos {
                    x: PHASE2_WIDTH - 1,
                    y,
                },
                TileKind::Wall,
            );
        }
        for y in 3..=10 {
            map.set_tile_unchecked(Pos { x: 18, y }, TileKind::Wall);
        }

        map.set_tile_unchecked(PHASE5_LEVEL2_STAIRS_UP, TileKind::StairsUp);
        map.set_tile_unchecked(PHASE5_LEVEL2_CLOSED_DOOR, TileKind::Door(DoorState::Closed));
        map
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn contains(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width && pos.y < self.height
    }

    pub fn tile(&self, pos: Pos) -> Result<TileKind, GameError> {
        let idx = self.index(pos)?;
        Ok(self.tiles[idx])
    }

    pub fn set_tile(&mut self, pos: Pos, tile: TileKind) -> Result<(), GameError> {
        let idx = self.index(pos)?;
        self.tiles[idx] = tile;
        Ok(())
    }

    pub fn tiles(&self) -> &[TileKind] {
        &self.tiles
    }

    fn index(&self, pos: Pos) -> Result<usize, GameError> {
        if !self.contains(pos) {
            return Err(GameError::OutOfBounds { pos });
        }
        Ok((pos.y as usize * self.width as usize) + pos.x as usize)
    }

    fn set_tile_unchecked(&mut self, pos: Pos, tile: TileKind) {
        let idx = (pos.y as usize * self.width as usize) + pos.x as usize;
        self.tiles[idx] = tile;
    }
}

#[cfg(test)]
mod tests {
    use super::{GameMap, PHASE2_HEIGHT, PHASE2_WIDTH};
    use crate::position::Pos;

    #[test]
    fn map_fixture_is_40x20() {
        let map = GameMap::fixture_phase2();

        assert_eq!(map.width, PHASE2_WIDTH);
        assert_eq!(map.height, PHASE2_HEIGHT);
        assert_eq!(map.tile_count(), 800);
    }

    #[test]
    fn map_bounds_returns_error() {
        let map = GameMap::fixture_phase2();
        let cases = [
            Pos { x: -1, y: 0 },
            Pos { x: 0, y: -1 },
            Pos { x: 40, y: 0 },
            Pos { x: 0, y: 20 },
        ];

        for pos in cases {
            assert!(map.tile(pos).is_err());
        }
    }
}
