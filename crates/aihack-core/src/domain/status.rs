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
            i16::MIN..=0 => HungerState::Fainting,
            1..=50 => HungerState::Weak,
            51..=150 => HungerState::Hungry,
            151..=1000 => HungerState::NotHungry,
            1001..=i16::MAX => HungerState::Satiated,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Fainting,
    Weak,
    Hungry,
    NotHungry,
    Satiated,
    /// v0.2 직렬화/API 호환용 legacy variant. 3.6.7 projection에서는 생성하지 않는다.
    Oversatiated,
}
