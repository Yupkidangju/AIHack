// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//!
//!

use legion::Entity;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionAction {
    ///
    Open,
    ///
    Close,
    ///
    Kick,
    ///
    Search,
    ///
    Throw { item: legion::Entity },
    ///
    Cast { spell_key: char },
    ///
    Talk,
    ///
    Zap { item: legion::Entity },
    ///
    Apply { item: legion::Entity },
    ///
    Loot,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetAction {
    ///
    Zap { wand: Entity },
    ///
    Throw { item: Entity },
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    ///
    Normal,

    ///
    ///
    WaitingForDirection {
        action: DirectionAction,
    },

    ///
    ///
    Targeting {
        action: TargetAction,
    },

    ///
    Inventory,

    ///
    Looting {
        container: Entity,
    },

    ///
    WaitingForSpell,

    ///
    OfferSelection,

    ///
    Help,

    ///
    GameOver {
        message: String,
    },

    ///
    More,

    ///
    Naming {
        entity: Option<Entity>,
        is_call: bool,
    },

    ///
    Enhance,

    ///
    SelectOffhand,
    SelectQuiver,
    SelectInvoke,
    ConfirmDrinkFountain,
    IdentifySelect {
        scroll: Entity,
        count: u32,
    },
    SelectEngraveTool,
    EngravingText {
        tool: Option<Entity>,
    },
    ConfirmRefill {
        lamp: Entity,
        oil: Entity,
    },
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Normal
    }
}

impl GameState {
    ///
    pub fn needs_direction(&self) -> bool {
        matches!(self, GameState::WaitingForDirection { .. })
    }

    ///
    pub fn needs_target(&self) -> bool {
        matches!(self, GameState::Targeting { .. })
    }

    ///
    pub fn reset(&mut self) {
        *self = GameState::Normal;
    }

    ///
    pub fn request_direction(&mut self, action: DirectionAction) {
        *self = GameState::WaitingForDirection { action };
    }

    ///
    pub fn request_target(&mut self, action: TargetAction) {
        *self = GameState::Targeting { action };
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Here,
}

impl Direction {
    ///
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
            Direction::Here => (0, 0),
        }
    }

    ///
    pub fn from_command(cmd: &crate::ui::input::Command) -> Option<Direction> {
        use crate::ui::input::Command;
        match cmd {
            Command::MoveN => Some(Direction::North),
            Command::MoveS => Some(Direction::South),
            Command::MoveE => Some(Direction::East),
            Command::MoveW => Some(Direction::West),
            Command::MoveNE => Some(Direction::NorthEast),
            Command::MoveNW => Some(Direction::NorthWest),
            Command::MoveSE => Some(Direction::SouthEast),
            Command::MoveSW => Some(Direction::SouthWest),
            Command::Wait => Some(Direction::Here),
            _ => None,
        }
    }
}
