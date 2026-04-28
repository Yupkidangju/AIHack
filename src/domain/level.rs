use serde::{Deserialize, Serialize};

use crate::{
    core::{ids::LevelId, position::Pos},
    domain::{
        map::{GameMap, PHASE2_STAIRS_DOWN, PHASE5_LEVEL2_STAIRS_UP},
        tile::TileKind,
    },
};

/// [v0.1.0] Phase 5 fixed level map과 식별자를 함께 보관한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameLevel {
    pub id: LevelId,
    pub map: GameMap,
}

/// [v0.1.0] Phase 5는 main:1, main:2 두 층만 deterministic 순서로 보관한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelRegistry {
    pub levels: Vec<GameLevel>,
}

impl LevelRegistry {
    pub fn fixture_phase5() -> Self {
        Self {
            levels: vec![
                GameLevel {
                    id: LevelId::main(1),
                    map: GameMap::fixture_phase2(),
                },
                GameLevel {
                    id: LevelId::main(2),
                    map: GameMap::fixture_phase5_level2(),
                },
            ],
        }
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn get(&self, id: LevelId) -> Option<&GameLevel> {
        self.levels.iter().find(|level| level.id == id)
    }

    pub fn get_mut(&mut self, id: LevelId) -> Option<&mut GameLevel> {
        self.levels.iter_mut().find(|level| level.id == id)
    }

    pub fn map(&self, id: LevelId) -> Option<&GameMap> {
        self.get(id).map(|level| &level.map)
    }

    pub fn map_mut(&mut self, id: LevelId) -> Option<&mut GameMap> {
        self.get_mut(id).map(|level| &mut level.map)
    }

    pub fn contains(&self, id: LevelId) -> bool {
        self.get(id).is_some()
    }

    pub fn stairs_up_pos(&self, id: LevelId) -> Option<Pos> {
        self.find_tile(id, TileKind::StairsUp)
    }

    pub fn stairs_down_pos(&self, id: LevelId) -> Option<Pos> {
        self.find_tile(id, TileKind::StairsDown)
    }

    fn find_tile(&self, id: LevelId, needle: TileKind) -> Option<Pos> {
        let map = self.map(id)?;
        for y in 0..map.height {
            for x in 0..map.width {
                let pos = Pos { x, y };
                if map.tile(pos).ok()? == needle {
                    return Some(pos);
                }
            }
        }
        None
    }
}

pub const PHASE5_LEVEL1_ID: LevelId = LevelId::main(1);
pub const PHASE5_LEVEL2_ID: LevelId = LevelId::main(2);
pub const PHASE5_LEVEL1_STAIRS_DOWN: Pos = PHASE2_STAIRS_DOWN;
pub const PHASE5_LEVEL2_STAIRS_UP_POS: Pos = PHASE5_LEVEL2_STAIRS_UP;
