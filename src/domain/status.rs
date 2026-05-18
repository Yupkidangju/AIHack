use serde::{Deserialize, Serialize};

/// [v0.2.0] Phase 20: 플레이어 상태를 하나의 struct로 통합한다.
/// GameWorld에서 산재하던 nutrition, luck, prayer_cooldown, paralysis_turns, hallucinating을 모두 포함한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub nutrition: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
}

impl Status {
    /// [v0.2.0] Phase 20: 기본 모험가 상태를 생성한다.
    pub fn default_adventurer() -> Self {
        Self {
            nutrition: 900, // ration 800 + 여유
            luck: 0,
            prayer_cooldown: 0,
            paralysis_turns: 0,
            hallucinating: false,
        }
    }

    /// [v0.2.0] Phase 20: 현재 허기 상태를 반환한다.
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

/// [v0.2.0] Phase 20: 허기 상태를 나타내는 enum이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Fainting,
    Weak,
    Hungry,
    Satiated,
    Oversatiated,
}
