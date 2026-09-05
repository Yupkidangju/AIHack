use serde::{Deserialize, Serialize};

use crate::{
    action::{DirectionalAction, InventoryAction},
    domain::combat::DeathCause,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection { action: DirectionalAction },
    AwaitingInventorySelection { action: InventoryAction },
    MorePrompt,
    GameOver { cause: DeathCause, final_score: i32 },
    Victory { final_score: i32 },
}
