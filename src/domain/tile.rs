use serde::{Deserialize, Serialize};

/// [v0.1.0] Phase 2 문 상태다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoorState {
    Closed,
    Open,
}

/// [v0.1.0] Phase 2 map/movement/vision에 필요한 최소 tile 종류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Wall,
    Floor,
    Door(DoorState),
    StairsDown,
    StairsUp,
}

impl TileKind {
    pub fn is_movement_passable(self) -> bool {
        matches!(
            self,
            TileKind::Floor
                | TileKind::Door(DoorState::Open)
                | TileKind::StairsDown
                | TileKind::StairsUp
        )
    }

    pub fn is_los_transparent(self) -> bool {
        matches!(
            self,
            TileKind::Floor
                | TileKind::Door(DoorState::Open)
                | TileKind::StairsDown
                | TileKind::StairsUp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{DoorState, TileKind};

    #[test]
    fn tile_blocker_contract_matches_phase2_prd() {
        let cases = [
            (TileKind::Wall, false, false),
            (TileKind::Floor, true, true),
            (TileKind::Door(DoorState::Closed), false, false),
            (TileKind::Door(DoorState::Open), true, true),
            (TileKind::StairsDown, true, true),
            (TileKind::StairsUp, true, true),
        ];

        for (tile, movement, los) in cases {
            assert_eq!(tile.is_movement_passable(), movement);
            assert_eq!(tile.is_los_transparent(), los);
        }
    }
}
