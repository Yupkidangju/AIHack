use serde::{Deserialize, Serialize};

use crate::core::{ids::EntityId, position::Direction};

/// [v0.2.0] Phase 16: AI가 command 외 목적성 있는 요청을 보낼 때 쓰는 최소 topic이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NarrativeTopic {
    SituationSummary,
}

/// [v0.2.0] Phase 16: 방향 입력을 기다리는 명령의 종류다.
/// Open, Close, Kick 명령은 방향 선택 후에 실제 동작이 실행된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionalAction {
    Open,
    Close,
    Kick,
}

/// [v0.2.0] Phase 16: 인벤토리 항목 선택을 기다리는 명령의 종류다.
/// Drop, Wield, Wear, Quaff, Read 명령은 인벤토리 letter 선택 후에 실제 동작이 실행된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryAction {
    Drop,
    Wield,
    Wear,
    Quaff,
    Read,
}

/// [v0.2.0] Phase 16: RunState와 CommandIntent 계약을 정렬한다.
/// AcknowledgeMore, DirectionalAction, InventoryAction을 추가한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandIntent {
    Wait,
    Quit,
    Move(Direction),
    Search,
    Kick(Direction),
    Open(Direction),
    Close(Direction),
    Pickup,
    Drop {
        item: EntityId,
    },
    Throw {
        item: EntityId,
        direction: Direction,
    },
    ShowInventory,
    Wield {
        item: EntityId,
    },
    Wear {
        item: EntityId,
    },
    Quaff {
        item: EntityId,
    },
    Zap {
        item: EntityId,
        direction: Direction,
    },
    Read {
        item: EntityId,
    },
    Pray,
    Descend,
    Ascend,
    AcknowledgeMore,
}

/// [v0.1.0] Phase 11 AI write contract다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionIntent {
    Command(CommandIntent),
    NarrativeRequest { topic: NarrativeTopic },
    Noop,
}
