// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// =============================================================================
// [v2.0.0
// =============================================================================
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
// =============================================================================

use crate::core::entity::status::StatusFlags;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

///
///
///
///
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    //
    ///
    DamageDealt {
        ///
        attacker: String,
        ///
        defender: String,
        ///
        amount: i32,
        ///
        source: String,
    },

    ///
    AttackMissed { attacker: String, defender: String },

    ///
    MonsterDied {
        ///
        name: String,
        ///
        killer: String,
        ///
        dropped_corpse: bool,
        ///
        x: i32,
        y: i32,
        ///
        xp_gained: u64,
    },

    ///
    PlayerDied {
        ///
        cause: String,
    },

    //
    ///
    ItemPickedUp { item_name: String, quantity: u32 },

    ///
    ItemDropped { item_name: String, x: i32, y: i32 },

    ///
    ItemUsed {
        item_name: String,
        ///
        use_type: String,
    },

    ///
    ItemEquipped { item_name: String, slot: String },

    ///
    ItemUnequipped { item_name: String, slot: String },

    //
    ///
    EquipmentChanged,

    //
    ///
    StatusApplied {
        target: String,
        status: StatusFlags,
        turns: u32,
    },

    ///
    StatusExpired { target: String, status: StatusFlags },

    ///
    HealthChanged {
        target: String,
        old_hp: i32,
        new_hp: i32,
        max_hp: i32,
    },

    //
    ///
    Moved {
        name: String,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },

    ///
    LevelChanged {
        direction: String, // "up", "down", "portal"
        new_depth: i32,
    },

    //
    ///
    TrapTriggered { trap_type: String, x: i32, y: i32 },

    ///
    DoorChanged { x: i32, y: i32, opened: bool },

    //
    ///
    ShopPurchase { item_name: String, price: u32 },

    ///
    ShopSale { item_name: String, price: u32 },

    ///
    Prayed { result: String },

    //
    ///
    LevelUp { new_level: i32 },

    ///
    ExperienceGained { amount: u64 },

    //
    ///
    Message { text: String },
}

impl GameEvent {
    ///
    pub fn category(&self) -> &str {
        match self {
            Self::DamageDealt { .. }
            | Self::AttackMissed { .. }
            | Self::MonsterDied { .. }
            | Self::PlayerDied { .. } => "combat",
            Self::ItemPickedUp { .. }
            | Self::ItemDropped { .. }
            | Self::ItemUsed { .. }
            | Self::ItemEquipped { .. }
            | Self::ItemUnequipped { .. } => "item",
            Self::EquipmentChanged => "equipment",
            Self::StatusApplied { .. }
            | Self::StatusExpired { .. }
            | Self::HealthChanged { .. } => "status",
            Self::Moved { .. } | Self::LevelChanged { .. } => "movement",
            Self::TrapTriggered { .. } | Self::DoorChanged { .. } => "environment",
            Self::ShopPurchase { .. } | Self::ShopSale { .. } | Self::Prayed { .. } => "social",
            Self::LevelUp { .. } | Self::ExperienceGained { .. } => "progression",
            Self::Message { .. } => "message",
        }
    }

    ///
    pub fn to_narrative(&self) -> String {
        match self {
            Self::DamageDealt {
                attacker,
                defender,
                amount,
                source,
            } => format!(
                "{} hit {} for {} damage with {}.",
                attacker, defender, amount, source
            ),
            Self::AttackMissed { attacker, defender } => {
                format!("{} missed {}.", attacker, defender)
            }
            Self::MonsterDied {
                name,
                killer,
                xp_gained,
                ..
            } => format!("{} was killed by {}. (+{} XP)", name, killer, xp_gained),
            Self::PlayerDied { cause } => format!("Player died: {}", cause),
            Self::ItemPickedUp {
                item_name,
                quantity,
            } => format!("Picked up {} (x{}).", item_name, quantity),
            Self::ItemDropped { item_name, .. } => format!("Dropped {}.", item_name),
            Self::ItemUsed {
                item_name,
                use_type,
            } => format!("Used {} ({}).", item_name, use_type),
            Self::ItemEquipped { item_name, slot } => {
                format!("Equipped {} in {} slot.", item_name, slot)
            }
            Self::ItemUnequipped { item_name, slot } => {
                format!("Unequipped {} from {} slot.", item_name, slot)
            }
            Self::EquipmentChanged => "Equipment configuration changed.".to_string(),
            Self::StatusApplied {
                target,
                status,
                turns,
            } => format!("{} gained status {:?} for {} turns.", target, status, turns),
            Self::StatusExpired { target, status } => {
                format!("{} lost status {:?}.", target, status)
            }
            Self::HealthChanged {
                target,
                old_hp,
                new_hp,
                max_hp,
            } => format!("{} HP: {} ??{} / {}.", target, old_hp, new_hp, max_hp),
            Self::Moved {
                name, to_x, to_y, ..
            } => format!("{} moved to ({}, {}).", name, to_x, to_y),
            Self::LevelChanged {
                direction,
                new_depth,
            } => format!("Went {} to level {}.", direction, new_depth),
            Self::TrapTriggered { trap_type, x, y } => {
                format!("Trap ({}) triggered at ({}, {}).", trap_type, x, y)
            }
            Self::DoorChanged { opened, x, y } => {
                let action = if *opened { "opened" } else { "closed" };
                format!("Door at ({}, {}) {}.", x, y, action)
            }
            Self::ShopPurchase { item_name, price } => {
                format!("Bought {} for {} gold.", item_name, price)
            }
            Self::ShopSale { item_name, price } => {
                format!("Sold {} for {} gold.", item_name, price)
            }
            Self::Prayed { result } => format!("Prayed. Result: {}.", result),
            Self::LevelUp { new_level } => format!("Leveled up to {}!", new_level),
            Self::ExperienceGained { amount } => format!("Gained {} XP.", amount),
            Self::Message { text } => text.clone(),
        }
    }
}

// =============================================================================
//
// =============================================================================

///
///
///
///
///
#[derive(Debug, Default)]
pub struct EventQueue {
    ///
    events: Vec<GameEvent>,
    ///
    turn: u64,
}

impl EventQueue {
    ///
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            turn: 0,
        }
    }

    ///
    pub fn clear(&mut self, turn: u64) {
        self.events.clear();
        self.turn = turn;
    }

    ///
    pub fn push(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    ///
    pub fn iter(&self) -> impl Iterator<Item = &GameEvent> {
        self.events.iter()
    }

    ///
    pub fn filter_by_category<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = &'a GameEvent> + 'a {
        self.events.iter().filter(move |e| e.category() == category)
    }

    ///
    pub fn len(&self) -> usize {
        self.events.len()
    }

    ///
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    ///
    pub fn current_turn(&self) -> u64 {
        self.turn
    }
}

// =============================================================================
//
// =============================================================================

///
///
///
///
#[derive(Debug)]
pub struct EventHistory {
    ///
    max_size: usize,
    ///
    buffer: VecDeque<(u64, GameEvent)>,
}

impl EventHistory {
    ///
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            buffer: VecDeque::with_capacity(max_size),
        }
    }

    ///
    pub fn record(&mut self, turn: u64, event: GameEvent) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back((turn, event));
    }

    ///
    pub fn record_all(&mut self, queue: &EventQueue) {
        for event in queue.iter() {
            self.record(queue.current_turn(), event.clone());
        }
    }

    ///
    pub fn recent_narrative(&self, count: usize) -> Vec<String> {
        self.buffer
            .iter()
            .rev()
            .take(count)
            .map(|(turn, event)| format!("[T{}] {}", turn, event.to_narrative()))
            .collect()
    }

    ///
    pub fn recent_by_category(&self, category: &str, count: usize) -> Vec<&GameEvent> {
        self.buffer
            .iter()
            .rev()
            .filter(|(_, e)| e.category() == category)
            .take(count)
            .map(|(_, e)| e)
            .collect()
    }

    ///
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    ///
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    ///
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for EventHistory {
    fn default() -> Self {
        //
        Self::new(200)
    }
}
