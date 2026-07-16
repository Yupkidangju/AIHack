use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub nutrition: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
}

impl Status {
    pub fn default_adventurer() -> Self {
        Self {
            nutrition: 900,
            luck: 0,
            prayer_cooldown: 0,
            paralysis_turns: 0,
            hallucinating: false,
        }
    }
    pub fn hunger_state(&self) -> HungerState {
        match self.nutrition {
            0..=150 => HungerState::Fainting,
            151..=300 => HungerState::Weak,
            301..=500 => HungerState::Hungry,
            501..=2000 => HungerState::Satiated,
            _ => HungerState::Oversatiated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Fainting,
    Weak,
    Hungry,
    Satiated,
    Oversatiated,
}
