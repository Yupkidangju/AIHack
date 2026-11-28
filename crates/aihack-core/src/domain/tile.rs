use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Open,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrapKind {
    Pit,
    Dart,
    Bear,
    Teleport,
    SleepingGas,
    Rust,
    Polymorph,
    Hole,
    AntiMagic,
    FallingRock,
    Magic,
    Fire,
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
    Shop,
    Altar,
    Fountain,
    DryFountain,
    Sink,
    Water,
    Lava,
    Boulder,
    MagicPortal,
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
                | Self::Shop
                | Self::Altar
                | Self::Fountain
                | Self::DryFountain
                | Self::Sink
                | Self::Water
                | Self::Lava
                | Self::MagicPortal
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
