// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use bitflags::bitflags;

use serde::{Deserialize, Serialize};

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Stone = 0,
    VWall = 1,
    HWall = 2,
    TlCorner = 3,
    TrCorner = 4,
    BlCorner = 5,
    BrCorner = 6,
    CrossWall = 7,
    TuWall = 8,
    TdWall = 9,
    TlWall = 10,
    TrWall = 11,
    DbWall = 12,
    Tree = 13,
    SDoor = 14,
    SCorr = 15,
    Pool = 16,
    Moat = 17,
    Water = 18,
    DrawbridgeUp = 19,
    LavaPool = 20,
    IronBars = 21,
    Door = 22,
    Corr = 23,
    Room = 24,
    StairsUp = 25,
    StairsDown = 36,
    Ladder = 26,
    Fountain = 27,
    Throne = 28,
    Sink = 29,
    Grave = 30,
    Altar = 31,
    Ice = 32,
    DrawbridgeDown = 33,
    Air = 34,
    Cloud = 35,
    OpenDoor = 43,
    Hole = 37,
    TrapDoor = 38,
}

impl TileType {
    pub fn is_wall(&self) -> bool {
        match self {
            TileType::VWall
            | TileType::HWall
            | TileType::TlCorner
            | TileType::TrCorner
            | TileType::BlCorner
            | TileType::BrCorner
            | TileType::CrossWall
            | TileType::TuWall
            | TileType::TdWall
            | TileType::TlWall
            | TileType::TrWall
            | TileType::DbWall => true,
            _ => false,
        }
    }
}

bitflags! {
    ///
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TileFlags: u8 {
        const NONE = 0;
        const LIT = 0x01;
        const WAS_LIT = 0x02;
        const HORIZONTAL = 0x04;
        const EDGE = 0x08;
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngraveType {
    Dust,
    Blood,
    Scratched,
    Burned,
    Etched,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engraving {
    pub text: String,
    pub typ: EngraveType,
    pub age: u64,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub glyph: i32,
    pub typ: TileType,
    pub seenv: u8,
    pub flags: TileFlags,
    pub roomno: u8,
    pub doormas: i8,
    pub altarmask: i8,
    pub shop_type: u8,
    pub engraving: Option<Engraving>,
}

impl Tile {
    pub fn new(typ: TileType) -> Self {
        Self {
            glyph: 0,
            typ,
            seenv: 0,
            flags: TileFlags::NONE,
            roomno: 0,
            doormas: 0,
            altarmask: 0,
            shop_type: 0,
            engraving: None,
        }
    }
}
