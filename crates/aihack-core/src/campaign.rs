use crate::{
    ids::EntityId,
    world::{is_zero_i16, is_zero_u16},
};
use serde::{Deserialize, Serialize};

pub const MAX_XP: u32 = 180;
pub const MIN_ATTRIBUTE: i16 = 3;
pub const MAX_ATTRIBUTE: i16 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Lawful,
    Neutral,
    Chaotic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Race {
    #[default]
    Human,
    Elf,
    Dwarf,
    Gnome,
    Orc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LearnedSpell {
    pub spell: SpellKind,
    pub learned_at_turn: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attributes {
    pub str_: i16,
    pub dex: i16,
    pub con: i16,
    pub int: i16,
    pub wis: i16,
    pub cha: i16,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            str_: 12,
            dex: 12,
            con: 12,
            int: 12,
            wis: 12,
            cha: 12,
        }
    }
}

impl Attributes {
    pub const fn is_valid(self) -> bool {
        self.str_ >= MIN_ATTRIBUTE
            && self.str_ <= MAX_ATTRIBUTE
            && self.dex >= MIN_ATTRIBUTE
            && self.dex <= MAX_ATTRIBUTE
            && self.con >= MIN_ATTRIBUTE
            && self.con <= MAX_ATTRIBUTE
            && self.int >= MIN_ATTRIBUTE
            && self.int <= MAX_ATTRIBUTE
            && self.wis >= MIN_ATTRIBUTE
            && self.wis <= MAX_ATTRIBUTE
            && self.cha >= MIN_ATTRIBUTE
            && self.cha <= MAX_ATTRIBUTE
    }

    pub const fn carrying_capacity(self) -> i16 {
        60 + self.str_.saturating_mul(5)
    }

    pub fn melee_damage_bonus(self) -> i16 {
        if self.str_ >= 14 {
            (self.str_ - 12) / 2
        } else if self.str_ <= 9 {
            -1
        } else {
            0
        }
    }

    pub fn dex_hit_bonus(self) -> i16 {
        if self.dex >= 14 {
            (self.dex - 12) / 2
        } else if self.dex <= 9 {
            -1
        } else {
            0
        }
    }

    pub const fn trap_damage_reduction(self) -> i16 {
        if self.dex >= 15 {
            1
        } else {
            0
        }
    }

    pub const fn con_hp_per_level_bonus(self) -> i16 {
        if self.con > 12 {
            (self.con - 12) / 3
        } else {
            0
        }
    }

    pub const fn con_regen_reduction(self) -> u8 {
        if self.con >= 15 {
            1
        } else {
            0
        }
    }

    pub fn int_spell_bonus(self) -> i16 {
        ((self.int - 12) / 2).max(0)
    }

    pub fn wis_heal_bonus(self) -> i16 {
        ((self.wis - 12) / 2).max(0)
    }

    pub fn wis_prayer_cooldown(self) -> u16 {
        let reduction = u16::try_from((self.wis - 12).max(0)).unwrap_or(0);
        20_u16.saturating_sub(reduction).max(10)
    }

    pub fn cha_buy_price(self, base: u32) -> u32 {
        let base = base.max(10);
        if self.cha <= 8 {
            ((base.saturating_mul(6)) / 5).max(1)
        } else if self.cha >= 15 {
            ((base.saturating_mul(85)) / 100).max(1)
        } else {
            base
        }
    }

    pub fn cha_sell_price(self, base: u32) -> u32 {
        let base = (base / 2).max(1);
        if self.cha <= 8 {
            ((base.saturating_mul(4)) / 5).max(1)
        } else if self.cha >= 15 {
            ((base.saturating_mul(115)) / 100).max(1)
        } else {
            base
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellKind {
    ForceBolt,
    Heal,
    Detect,
}

impl SpellKind {
    pub const fn mana_cost(self) -> i16 {
        match self {
            Self::ForceBolt => 4,
            Self::Heal => 5,
            Self::Detect => 3,
        }
    }

    pub const fn is_directional(self) -> bool {
        matches!(self, Self::ForceBolt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Knight,
    Scout,
    Mage,
    Cleric,
}

impl Role {
    pub const fn base_hp(self) -> i16 {
        match self {
            Self::Knight => 28,
            Self::Scout => 22,
            Self::Mage => 18,
            Self::Cleric => 24,
        }
    }
    pub const fn base_hit(self) -> i16 {
        match self {
            Self::Knight => 4,
            Self::Scout => 5,
            Self::Mage => 3,
            Self::Cleric => 3,
        }
    }
    pub const fn base_damage(self) -> i16 {
        match self {
            Self::Knight => 2,
            Self::Scout => 1,
            Self::Mage => 0,
            Self::Cleric => 1,
        }
    }
    pub const fn base_mp(self) -> i16 {
        match self {
            Self::Mage => 14,
            Self::Cleric => 10,
            Self::Scout => 6,
            Self::Knight => 4,
        }
    }
    pub const fn mp_per_level(self) -> i16 {
        match self {
            Self::Mage => 3,
            Self::Cleric => 2,
            Self::Scout => 1,
            Self::Knight => 1,
        }
    }
    pub const fn base_attributes(self) -> Attributes {
        match self {
            Self::Knight => Attributes {
                str_: 16,
                dex: 11,
                con: 15,
                int: 9,
                wis: 10,
                cha: 12,
            },
            Self::Scout => Attributes {
                str_: 12,
                dex: 16,
                con: 12,
                int: 11,
                wis: 10,
                cha: 12,
            },
            Self::Mage => Attributes {
                str_: 9,
                dex: 12,
                con: 10,
                int: 17,
                wis: 14,
                cha: 11,
            },
            Self::Cleric => Attributes {
                str_: 12,
                dex: 10,
                con: 14,
                int: 11,
                wis: 16,
                cha: 14,
            },
        }
    }
    pub const fn default_alignment(self) -> Alignment {
        match self {
            Self::Knight | Self::Cleric => Alignment::Lawful,
            Self::Scout | Self::Mage => Alignment::Neutral,
        }
    }
    pub const fn default_race(self) -> Race {
        Race::Human
    }
    pub const fn known_spells(self) -> &'static [SpellKind] {
        match self {
            Self::Mage => &[SpellKind::ForceBolt, SpellKind::Detect],
            Self::Cleric => &[SpellKind::Heal, SpellKind::Detect],
            Self::Scout => &[SpellKind::Detect],
            Self::Knight => &[SpellKind::Heal],
        }
    }
    pub fn knows_spell(self, spell: SpellKind) -> bool {
        self.known_spells().contains(&spell)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignState {
    pub role: Role,
    pub xp: u32,
    pub amulet: EntityId,
    #[serde(default)]
    pub attributes: Attributes,
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default)]
    pub race: Race,
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub god_anger: i16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub prayer_timeout: u16,
}

impl CampaignState {
    pub fn new(role: Role, amulet: EntityId) -> Self {
        Self {
            role,
            xp: 0,
            amulet,
            attributes: role.base_attributes(),
            alignment: role.default_alignment(),
            race: role.default_race(),
            god_anger: 0,
            prayer_timeout: 0,
        }
    }
    pub fn level(self) -> u8 {
        (1 + self.xp.min(MAX_XP) / 20) as u8
    }
    pub fn max_hp(self) -> i16 {
        self.role.base_hp()
            + (4 + self.attributes.con_hp_per_level_bonus()) * i16::from(self.level() - 1)
    }
    pub fn max_mp(self) -> i16 {
        self.role.base_mp() + self.role.mp_per_level() * i16::from(self.level() - 1)
    }
    pub fn hit_bonus(self) -> i16 {
        self.role.base_hit() + i16::from(self.level() - 1)
    }
    pub fn damage_bonus(self) -> i16 {
        self.role.base_damage() + i16::from(self.level() - 1)
    }
    pub fn known_spells(self) -> &'static [SpellKind] {
        self.role.known_spells()
    }
    pub fn knows_spell(self, spell: SpellKind) -> bool {
        self.role.knows_spell(spell)
    }
}
