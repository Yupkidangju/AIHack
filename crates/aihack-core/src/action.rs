use serde::{Deserialize, Serialize};

use crate::{
    campaign::SpellKind,
    domain::item::{EquipmentSlot, ItemKind},
    ids::EntityId,
    position::Direction,
};

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
    PutOn,
    Sell,
    Offer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DipTarget {
    Item(EntityId),
    Tile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngravingMethod {
    Fingers,
    Item(EntityId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngravingText {
    Elbereth,
    Warning,
    Safe,
    XMark,
}

impl EngravingText {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Elbereth => "Elbereth",
            Self::Warning => "Beware!",
            Self::Safe => "Safe",
            Self::XMark => "X",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandIntent {
    StartCampaign {
        role: crate::campaign::Role,
    },
    EnterBranch,
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
    PutOn {
        item: EntityId,
    },
    RemoveRing,
    TakeOff {
        slot: EquipmentSlot,
    },
    Buy {
        kind: ItemKind,
    },
    Sell {
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
    Identify {
        scroll: EntityId,
        target: EntityId,
    },
    Offer {
        item: EntityId,
    },
    Cast {
        spell: SpellKind,
        direction: Option<Direction>,
    },
    Pray,
    Apply {
        item: EntityId,
        direction: Option<Direction>,
    },
    ContainerPut {
        container: EntityId,
        item: EntityId,
    },
    ContainerTake {
        container: EntityId,
        item: EntityId,
    },
    Dip {
        item: EntityId,
        target: DipTarget,
    },
    Engrave {
        method: EngravingMethod,
        text: EngravingText,
    },
    Drink,
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
