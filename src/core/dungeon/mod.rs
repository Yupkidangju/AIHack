pub mod dungeon;
pub mod gen;
pub mod mkroom;
pub mod rect;
pub mod tile;

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
#[derive(Clone, Serialize, Deserialize)]
pub struct Grid {
    pub locations: Vec<[Tile; ROWNO]>,
    pub portals: HashMap<(i32, i32), LevelID>,
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
