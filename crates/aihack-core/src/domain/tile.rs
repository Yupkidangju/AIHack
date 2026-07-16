use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrapKind {
    Pit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Wall,
    Floor,
    Door(DoorState),
    HiddenDoor,
    Trap(TrapKind),
    HiddenTrap(TrapKind),
    StairsDown,
    StairsUp,
}

impl TileKind {
    pub fn is_movement_passable(self) -> bool {
        matches!(
            self,
            Self::Floor
                | Self::Door(DoorState::Open)
                | Self::Trap(_)
                | Self::HiddenTrap(_)
                | Self::StairsDown
                | Self::StairsUp
        )
    }
    pub fn is_los_transparent(self) -> bool {
        self.is_movement_passable()
    }
    pub fn revealed_equivalent(self) -> Self {
        match self {
            Self::HiddenDoor => Self::Door(DoorState::Closed),
            Self::HiddenTrap(kind) => Self::Trap(kind),
            other => other,
        }
    }
    pub fn observation_equivalent(self) -> Self {
        match self {
            Self::HiddenDoor => Self::Wall,
            Self::HiddenTrap(_) => Self::Floor,
            other => other,
        }
    }
    pub fn is_hidden(self) -> bool {
        matches!(self, Self::HiddenDoor | Self::HiddenTrap(_))
    }
}
