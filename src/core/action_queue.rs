use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum GameAction {
    Item(crate::core::systems::item_use::ItemAction),
    Throw(crate::core::systems::throw::ThrowAction),
    Cast(crate::core::systems::spell::CastAction),
    Zap(crate::core::systems::zap::ZapAction),
    Teleport(crate::core::systems::teleport::TeleportAction),
    LevelChange(crate::core::dungeon::LevelChange),
}

#[derive(Default, Clone, Debug)]
pub struct ActionQueue {
    pub actions: VecDeque<GameAction>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            actions: VecDeque::new(),
        }
    }

    pub fn push(&mut self, action: GameAction) {
        self.actions.push_back(action);
    }

    pub fn pop(&mut self) -> Option<GameAction> {
        self.actions.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }
}
