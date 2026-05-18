use serde::{Deserialize, Serialize};

/// [v0.1.0] Phase 2 문 상태다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Open,
}

/// [v0.1.0] Phase 7 최소 trap 종류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrapKind {
    Pit,
}

/// [v0.1.0] Phase 7 map/movement/vision에 필요한 최소 tile 종류다.
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
            TileKind::Floor
                | TileKind::Door(DoorState::Open)
                | TileKind::Trap(_)
                | TileKind::HiddenTrap(_)
                | TileKind::StairsDown
                | TileKind::StairsUp
        )
    }

    pub fn is_los_transparent(self) -> bool {
        matches!(
            self,
            TileKind::Floor
                | TileKind::Door(DoorState::Open)
                | TileKind::Trap(_)
                | TileKind::HiddenTrap(_)
                | TileKind::StairsDown
                | TileKind::StairsUp
        )
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

#[cfg(test)]
mod tests {
    use super::{DoorState, TileKind, TrapKind};

    #[test]
    fn tile_blocker_contract_matches_phase2_prd() {
        let cases = [
            (TileKind::Wall, false, false),
            (TileKind::Floor, true, true),
            (TileKind::Door(DoorState::Closed), false, false),
            (TileKind::Door(DoorState::Open), true, true),
            (TileKind::HiddenDoor, false, false),
            (TileKind::Trap(TrapKind::Pit), true, true),
            (TileKind::HiddenTrap(TrapKind::Pit), true, true),
            (TileKind::StairsDown, true, true),
            (TileKind::StairsUp, true, true),
        ];

        for (tile, movement, los) in cases {
            assert_eq!(tile.is_movement_passable(), movement);
            assert_eq!(tile.is_los_transparent(), los);
        }
    }
}
