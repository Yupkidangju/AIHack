use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LevelId {
    pub branch: BranchId,
    pub depth: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BranchId {
    Main,
    Mines,
    Sokoban,
    Quest,
    Gehennom,
    Planes,
}

impl BranchId {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Main => "Main",
            Self::Mines => "Mines",
            Self::Sokoban => "Sokoban",
            Self::Quest => "Quest",
            Self::Gehennom => "Gehennom",
            Self::Planes => "Planes",
        }
    }
}

impl LevelId {
    pub const fn main(depth: i16) -> Self {
        Self {
            branch: BranchId::Main,
            depth,
        }
    }

    pub const fn new(branch: BranchId, depth: i16) -> Self {
        Self { branch, depth }
    }
}
