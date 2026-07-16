//! 터미널 UI adapter package다.

pub mod tui;

pub const ADAPTER_NAME: &str = "aihack";

mod core {
    pub use aihack_ai_contract::{
        ActionIntent, BranchId, CommandIntent, Direction, EntityObservation, GameEvent,
        Observation, Pos,
    };
    pub mod action {
        pub use aihack_ai_contract::{CommandIntent, DirectionalAction, InventoryAction};
    }

    pub mod observation {
        pub use aihack_ai_contract::*;
    }

    pub mod position {
        pub use aihack_ai_contract::Pos;
    }

    pub mod session {
        pub use aihack_ai_contract::RunState;
    }
}

mod domain {
    pub mod combat {
        pub use aihack_ai_contract::DeathCause;
    }

    pub mod entity {
        pub use aihack_ai_contract::EntityKind;
    }

    pub mod item {
        pub use aihack_ai_contract::ItemKind;
    }

    pub mod monster {
        pub use aihack_ai_contract::MonsterKind;
    }

    pub mod tile {
        pub use aihack_ai_contract::TileKind;
    }
}

mod llm {
    pub use aihack_llm::{decision, narrative};
}

mod ui {
    pub mod tui {
        pub use crate::tui::UiPanel;
    }
}
