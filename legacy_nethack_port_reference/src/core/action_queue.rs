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

    // =========================================================================
    // [S7-A] 타입별 pop 메서드 — 해당 타입의 첫 번째 액션만 꺼내고 나머지 보존
    // 기존 drain-last-one 패턴을 대체하여 다중 액션 손실 방지
    // =========================================================================

    /// Item 타입의 첫 번째 액션을 꺼냄
    pub fn pop_item(&mut self) -> Option<crate::core::systems::item_use::ItemAction> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::Item(_)));
        if let Some(i) = idx {
            if let GameAction::Item(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }

    /// Throw 타입의 첫 번째 액션을 꺼냄
    pub fn pop_throw(&mut self) -> Option<crate::core::systems::throw::ThrowAction> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::Throw(_)));
        if let Some(i) = idx {
            if let GameAction::Throw(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }

    /// Cast 타입의 첫 번째 액션을 꺼냄
    pub fn pop_cast(&mut self) -> Option<crate::core::systems::spell::CastAction> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::Cast(_)));
        if let Some(i) = idx {
            if let GameAction::Cast(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }

    /// Zap 타입의 첫 번째 액션을 꺼냄
    pub fn pop_zap(&mut self) -> Option<crate::core::systems::zap::ZapAction> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::Zap(_)));
        if let Some(i) = idx {
            if let GameAction::Zap(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }

    /// Teleport 타입의 첫 번째 액션을 꺼냄
    pub fn pop_teleport(&mut self) -> Option<crate::core::systems::teleport::TeleportAction> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::Teleport(_)));
        if let Some(i) = idx {
            if let GameAction::Teleport(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }

    /// LevelChange 타입의 첫 번째 액션을 꺼냄
    pub fn pop_level_change(&mut self) -> Option<crate::core::dungeon::LevelChange> {
        let idx = self.actions.iter().position(|a| matches!(a, GameAction::LevelChange(_)));
        if let Some(i) = idx {
            if let GameAction::LevelChange(a) = self.actions.remove(i).unwrap() {
                return Some(a);
            }
        }
        None
    }
}
