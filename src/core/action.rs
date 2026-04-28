use serde::{Deserialize, Serialize};

use crate::core::{ids::EntityId, position::Direction};

/// [v0.1.0] Phase 5는 계단 이동 명령을 추가한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandIntent {
    Wait,
    Quit,
    Move(Direction),
    Open(Direction),
    Close(Direction),
    Pickup,
    ShowInventory,
    Wield { item: EntityId },
    Quaff { item: EntityId },
    Descend,
    Ascend,
}
