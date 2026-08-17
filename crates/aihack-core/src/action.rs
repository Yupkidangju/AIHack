use serde::{Deserialize, Serialize};

use crate::{ids::EntityId, position::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NarrativeTopic {
    SituationSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionalAction {
    Open,
    Close,
    Kick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryAction {
    Drop,
    Wield,
    Wear,
    Quaff,
    Eat,
    Read,
}

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
    Eat {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionIntent {
    Command(CommandIntent),
    NarrativeRequest { topic: NarrativeTopic },
    Noop,
}
