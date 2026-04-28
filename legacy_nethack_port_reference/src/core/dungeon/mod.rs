pub mod boundary_ext;
pub mod dungeon;
pub mod dungeon_ext;
pub mod dungeon_feature_ext;
pub mod extralev_ext;
pub mod gen;
pub mod level_change_ext;
pub mod mapseen_ext;
pub mod mklev_ext;
pub mod mklev_ext2;
pub mod mkmaze_ext;
pub mod mkroom;
pub mod mkroom_ext;
pub mod rect;
pub mod sp_lev;
pub mod stairs_ext;
pub mod tile;
// [v2.29.0 Phase 93] 던전 생성 확장
pub mod mklev_phase93_ext;
// [v2.31.0 Phase 95] 특수방/레벨 확장
pub mod mkroom_phase95_ext;
pub mod sp_lev_phase95_ext;
// [v2.34.0 Phase 98] 던전 분기 확장
pub mod branch_phase98_ext;
// [v2.36.0 Phase 100] 🏆 맵 생성 통합
pub mod mapgen_phase100_ext;
// [v2.39.0 Phase 103] 특수 층/분기
pub mod special_level_phase103_ext;

use crate::core::dungeon::tile::{Tile, TileType};

pub const COLNO: usize = 80;
pub const ROWNO: usize = 21;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DungeonBranch {
    Main,
    Mines,
    Sokoban,
    ///
    Gehennom,
    ///
    Quest,
    ///
    VladTower,
    ///
    FortKnox,
    ///
    Astral,
    ///
    EndGame,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LevelID {
    pub branch: DungeonBranch,
    pub depth: i32,
}

impl LevelID {
    pub fn new(branch: DungeonBranch, depth: i32) -> Self {
        Self { branch, depth }
    }
}

///
#[derive(Clone)]
pub struct Grid {
    pub locations: Vec<[Tile; ROWNO]>,
    pub portals: HashMap<(i32, i32), LevelID>,
}

#[derive(Serialize, Deserialize)]
struct GridDef {
    locations: Vec<[Tile; ROWNO]>,
    portals: Vec<((i32, i32), LevelID)>,
}

impl serde::Serialize for Grid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let def = GridDef {
            locations: self.locations.clone(),
            portals: self.portals.clone().into_iter().collect(),
        };
        def.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Grid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = GridDef::deserialize(deserializer)?;
        Ok(Grid {
            locations: def.locations,
            portals: def.portals.into_iter().collect(),
        })
    }
}

impl Grid {
    pub fn new() -> Self {
        //
        let mut locations = Vec::with_capacity(COLNO);
        for _ in 0..COLNO {
            //
            let col = [(); ROWNO].map(|_| Tile::new(TileType::Stone));
            locations.push(col);
        }

        Self {
            locations,
            portals: HashMap::new(),
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < COLNO && y < ROWNO {
            Some(&self.locations[x][y])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < COLNO && y < ROWNO {
            Some(&mut self.locations[x][y])
        } else {
            None
        }
    }

    ///
    pub fn light_room_at(&mut self, x: usize, y: usize) {
        if let Some(tile) = self.get_tile(x, y) {
            let room_id = tile.roomno;
            if room_id > 0 {
                //
                for cx in 0..COLNO {
                    for cy in 0..ROWNO {
                        if self.locations[cx][cy].roomno == room_id {
                            self.locations[cx][cy].flags |=
                                crate::core::dungeon::tile::TileFlags::LIT;
                        }
                    }
                }
            } else {
                //
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        let tx = x as i32 + dx;
                        let ty = y as i32 + dy;
                        if tx >= 0 && tx < COLNO as i32 && ty >= 0 && ty < ROWNO as i32 {
                            self.locations[tx as usize][ty as usize].flags |=
                                crate::core::dungeon::tile::TileFlags::LIT;
                        }
                    }
                }
            }
        }
    }
}

///

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandingType {
    StairsUp,
    StairsDown,
    Coordinate(i32, i32),
    Random,
    Connection(LevelID),
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelChange {
    NextLevel,
    PrevLevel,
    Teleport {
        target: LevelID,
        landing: LandingType,
    },
}
