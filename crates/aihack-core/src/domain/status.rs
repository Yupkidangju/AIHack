use serde::{Deserialize, Serialize};

pub const STARVATION_THRESHOLD: i16 = -150;

fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Resistances(pub u16);

impl Resistances {
    pub const NONE: Self = Self(0);
    pub const FIRE: Self = Self(1 << 0);
    pub const COLD: Self = Self(1 << 1);
    pub const SHOCK: Self = Self(1 << 2);
    pub const POISON: Self = Self(1 << 3);
    pub const SLEEP: Self = Self(1 << 4);
    pub const DISINTEGRATION: Self = Self(1 << 5);
    pub const TELEPATHY: Self = Self(1 << 6);
    pub const SEE_INVISIBLE: Self = Self(1 << 7);

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "fire" => Some(Self::FIRE),
            "cold" => Some(Self::COLD),
            "shock" => Some(Self::SHOCK),
            "poison" => Some(Self::POISON),
            "sleep" => Some(Self::SLEEP),
            "disintegration" => Some(Self::DISINTEGRATION),
            "telepathy" => Some(Self::TELEPATHY),
            "see_invisible" => Some(Self::SEE_INVISIBLE),
            _ => None,
        }
    }

    pub fn badge_string(self) -> String {
        let mut out = String::new();
        for (flag, ch) in [
            (Self::FIRE, 'F'),
            (Self::COLD, 'C'),
            (Self::SHOCK, 'S'),
            (Self::POISON, 'P'),
            (Self::SLEEP, 'Z'),
            (Self::DISINTEGRATION, 'D'),
            (Self::TELEPATHY, 'T'),
            (Self::SEE_INVISIBLE, 'I'),
        ] {
            if self.contains(flag) {
                out.push(ch);
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StatusTimers {
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub blind: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub confused: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub stunned: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub hallucinating: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub fast: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub levitation: u8,
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub invisible: u8,
}

impl StatusTimers {
    pub const fn is_empty(&self) -> bool {
        self.blind == 0
            && self.confused == 0
            && self.stunned == 0
            && self.hallucinating == 0
            && self.fast == 0
            && self.levitation == 0
            && self.invisible == 0
    }
}

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
