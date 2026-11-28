use std::collections::BTreeSet;

use crate::{
    domain::{item::ItemKind, status::Resistances},
    ids::EntityId,
    world::{is_zero_i16, is_zero_u16},
};
use serde::{Deserialize, Serialize};

pub const MAX_XP: u32 = 180;
pub const MIN_ATTRIBUTE: i16 = 3;
pub const MAX_ATTRIBUTE: i16 = 25;
pub const INFRAVISION_RADIUS: i32 = 8;
pub const ARTIFACT_INVOKE_COOLDOWN_TURNS: u16 = 25;
pub const MIN_LUCK: i16 = -13;
pub const MAX_LUCK: i16 = 13;
pub const MAX_GOD_ANGER: i16 = 100;
pub const MAX_PRAYER_TIMEOUT: u16 = 2000;
pub const SPELL_MEMORY_TURNS: u64 = 20_000;
pub const MAX_LEARNED_SPELLS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Lawful,
    Neutral,
    Chaotic,
}

impl Alignment {
    pub const ALL: [Self; 3] = [Self::Lawful, Self::Neutral, Self::Chaotic];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lawful => "Lawful",
            Self::Neutral => "Neutral",
            Self::Chaotic => "Chaotic",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Lawful => "L",
            Self::Neutral => "N",
            Self::Chaotic => "C",
        }
    }

    /// 교차 성향 제단 배치 등에 사용할 반대 성향을 반환한다.
    pub const fn cross_alignment(self) -> Self {
        match self {
            Self::Lawful | Self::Neutral => Self::Chaotic,
            Self::Chaotic => Self::Lawful,
        }
    }
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

impl Race {
    pub const ALL: [Self; 5] = [Self::Human, Self::Elf, Self::Dwarf, Self::Gnome, Self::Orc];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Human => "Human",
            Self::Elf => "Elf",
            Self::Dwarf => "Dwarf",
            Self::Gnome => "Gnome",
            Self::Orc => "Orc",
        }
    }

    /// `[str_, dex, con, int, wis, cha]` 순서의 종족 능력치 보정치다.
    pub const fn attribute_deltas(self) -> [i16; 6] {
        match self {
            Self::Human => [0, 0, 0, 0, 0, 0],
            Self::Elf => [-1, 2, -1, 2, 0, 1],
            Self::Dwarf => [2, -1, 2, -1, 0, -1],
            Self::Gnome => [-1, 1, 0, 1, 1, 0],
            Self::Orc => [2, 0, 2, -2, -1, -2],
        }
    }

    pub const fn starting_hp_bonus(self) -> i16 {
        match self {
            Self::Human => 0,
            Self::Elf => -1,
            Self::Dwarf => 3,
            Self::Gnome => -1,
            Self::Orc => 2,
        }
    }

    pub const fn starting_mp_bonus(self) -> i16 {
        match self {
            Self::Human => 0,
            Self::Elf => 2,
            Self::Dwarf => -1,
            Self::Gnome => 1,
            Self::Orc => -2,
        }
    }

    pub const fn intrinsic_resistances(self) -> Resistances {
        match self {
            Self::Elf => Resistances::SLEEP,
            Self::Orc => Resistances::POISON,
            Self::Human | Self::Dwarf | Self::Gnome => Resistances::NONE,
        }
    }

    pub const fn has_infravision(self) -> bool {
        !matches!(self, Self::Human)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    Dagger,
    Sword,
    Axe,
    Blunt,
    Staff,
    Thrown,
    BareHands,
    Spellcasting,
}

impl SkillCategory {
    pub const ALL: [Self; 8] = [
        Self::Dagger,
        Self::Sword,
        Self::Axe,
        Self::Blunt,
        Self::Staff,
        Self::Thrown,
        Self::BareHands,
        Self::Spellcasting,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Dagger => "dagger",
            Self::Sword => "sword",
            Self::Axe => "axe",
            Self::Blunt => "blunt",
            Self::Staff => "staff",
            Self::Thrown => "thrown",
            Self::BareHands => "bare_hands",
            Self::Spellcasting => "spellcasting",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Dagger => "Dagger",
            Self::Sword => "Sword",
            Self::Axe => "Axe",
            Self::Blunt => "Blunt",
            Self::Staff => "Quarterstaff",
            Self::Thrown => "Thrown",
            Self::BareHands => "Bare Hands",
            Self::Spellcasting => "Spellcasting",
        }
    }

    pub const fn is_spell(self) -> bool {
        matches!(self, Self::Spellcasting)
    }

    pub fn from_key(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|cat| cat.key() == raw)
    }

    pub const fn for_weapon_kind(kind: ItemKind) -> Option<Self> {
        match kind {
            ItemKind::Dagger | ItemKind::ElvenDagger => Some(Self::Dagger),
            ItemKind::ShortSword
            | ItemKind::LongSword
            | ItemKind::BroadSword
            | ItemKind::Katana
            | ItemKind::SilverSaber
            | ItemKind::TwoHandedSword => Some(Self::Sword),
            ItemKind::BattleAxe => Some(Self::Axe),
            ItemKind::Mace | ItemKind::WarHammer => Some(Self::Blunt),
            ItemKind::Quarterstaff => Some(Self::Staff),
            ItemKind::Rock | ItemKind::Spear => Some(Self::Thrown),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum SkillRank {
    #[default]
    Unskilled = 0,
    Basic = 1,
    Skilled = 2,
    Expert = 3,
    Master = 4,
    GrandMaster = 5,
}

impl SkillRank {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Unskilled),
            1 => Some(Self::Basic),
            2 => Some(Self::Skilled),
            3 => Some(Self::Expert),
            4 => Some(Self::Master),
            5 => Some(Self::GrandMaster),
            _ => None,
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Unskilled => "Unskilled",
            Self::Basic => "Basic",
            Self::Skilled => "Skilled",
            Self::Expert => "Expert",
            Self::Master => "Master",
            Self::GrandMaster => "Grand Master",
        }
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Unskilled => Some(Self::Basic),
            Self::Basic => Some(Self::Skilled),
            Self::Skilled => Some(Self::Expert),
            Self::Expert => Some(Self::Master),
            Self::Master => Some(Self::GrandMaster),
            Self::GrandMaster => None,
        }
    }

    pub const fn practice_threshold(self) -> u16 {
        match self {
            Self::Unskilled => 20,
            Self::Basic => 40,
            Self::Skilled => 60,
            Self::Expert => 80,
            Self::Master => 100,
            Self::GrandMaster => 0,
        }
    }

    pub const fn hit_bonus(self) -> i16 {
        match self {
            Self::Unskilled => -1,
            Self::Basic => 0,
            Self::Skilled => 1,
            Self::Expert => 2,
            Self::Master => 3,
            Self::GrandMaster => 4,
        }
    }

    pub const fn damage_bonus(self) -> i16 {
        match self {
            Self::Unskilled | Self::Basic => 0,
            Self::Skilled => 1,
            Self::Expert => 2,
            Self::Master => 3,
            Self::GrandMaster => 4,
        }
    }

    pub const fn spell_power_bonus(self) -> i16 {
        match self {
            Self::Unskilled | Self::Basic => 0,
            Self::Skilled => 1,
            Self::Expert => 2,
            Self::Master => 4,
            Self::GrandMaster => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactDef {
    pub id: &'static str,
    pub name: &'static str,
    pub base_kind: ItemKind,
    pub alignment: Option<Alignment>,
    pub quest_role: Option<Role>,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub resistances: Resistances,
    pub emits_light: bool,
    pub drains_life: bool,
}

pub const ARTIFACTS: &[ArtifactDef] = &[
    ArtifactDef {
        id: "excalibur",
        name: "Excalibur",
        base_kind: ItemKind::LongSword,
        alignment: Some(Alignment::Lawful),
        quest_role: None,
        hit_bonus: 5,
        damage_bonus: 6,
        resistances: Resistances::SEE_INVISIBLE,
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "sunsword",
        name: "Sunsword",
        base_kind: ItemKind::BroadSword,
        alignment: Some(Alignment::Lawful),
        quest_role: None,
        hit_bonus: 2,
        damage_bonus: 4,
        resistances: Resistances::SEE_INVISIBLE,
        emits_light: true,
        drains_life: false,
    },
    ArtifactDef {
        id: "magicbane",
        name: "Magicbane",
        base_kind: ItemKind::Dagger,
        alignment: Some(Alignment::Neutral),
        quest_role: None,
        hit_bonus: 3,
        damage_bonus: 3,
        resistances: Resistances::SLEEP.union(Resistances::DISINTEGRATION),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "cleaver",
        name: "Cleaver",
        base_kind: ItemKind::BattleAxe,
        alignment: Some(Alignment::Neutral),
        quest_role: None,
        hit_bonus: 3,
        damage_bonus: 5,
        resistances: Resistances::FIRE,
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "stormbringer",
        name: "Stormbringer",
        base_kind: ItemKind::LongSword,
        alignment: Some(Alignment::Chaotic),
        quest_role: None,
        hit_bonus: 5,
        damage_bonus: 5,
        resistances: Resistances::POISON,
        emits_light: false,
        drains_life: true,
    },
    ArtifactDef {
        id: "sting",
        name: "Sting",
        base_kind: ItemKind::Dagger,
        alignment: None,
        quest_role: None,
        hit_bonus: 3,
        damage_bonus: 3,
        resistances: Resistances::SEE_INVISIBLE,
        emits_light: false,
        drains_life: false,
    },
    // --- P13 13 Role Quest Artifacts ---
    ArtifactDef {
        id: "magic_mirror_of_merlin",
        name: "Magic Mirror of Merlin",
        base_kind: ItemKind::AmuletReflection,
        alignment: Some(Alignment::Lawful),
        quest_role: Some(Role::Knight),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::MAGIC_RESISTANCE.union(Resistances::TELEPATHY),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "cloak_of_the_wayfarer",
        name: "Cloak of the Wayfarer",
        base_kind: ItemKind::ArmorCloak,
        alignment: None,
        quest_role: Some(Role::Scout),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::TELEPORT_CONTROL.union(Resistances::SEARCHING),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "tome_of_arcana",
        name: "Tome of Arcana",
        base_kind: ItemKind::Quarterstaff,
        alignment: None,
        quest_role: Some(Role::Mage),
        hit_bonus: 3,
        damage_bonus: 3,
        resistances: Resistances::MAGIC_RESISTANCE.union(Resistances::SEE_INVISIBLE),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "candelabrum_of_grace",
        name: "Candelabrum of Grace",
        base_kind: ItemKind::Mace,
        alignment: None,
        quest_role: Some(Role::Cleric),
        hit_bonus: 3,
        damage_bonus: 3,
        resistances: Resistances::FIRE.union(Resistances::SLEEP),
        emits_light: true,
        drains_life: false,
    },
    ArtifactDef {
        id: "orb_of_fate",
        name: "Orb of Fate",
        base_kind: ItemKind::GemDiamond,
        alignment: None,
        quest_role: Some(Role::Valkyrie),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::TELEPORT_CONTROL.union(Resistances::MAGIC_RESISTANCE),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "eye_of_the_aethiopica",
        name: "Eye of the Aethiopica",
        base_kind: ItemKind::AmuletEsp,
        alignment: None,
        quest_role: Some(Role::Wizard),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::MAGIC_RESISTANCE.union(Resistances::TELEPATHY),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "master_key_of_thievery",
        name: "Master Key of Thievery",
        base_kind: ItemKind::SkeletonKey,
        alignment: Some(Alignment::Chaotic),
        quest_role: Some(Role::Rogue),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::TELEPORT_CONTROL.union(Resistances::SEARCHING),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "mitre_of_holiness",
        name: "Mitre of Holiness",
        base_kind: ItemKind::ArmorHelm,
        alignment: None,
        quest_role: Some(Role::Priest),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::FIRE.union(Resistances::MAGIC_RESISTANCE),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "tsurugi_of_muramasa",
        name: "Tsurugi of Muramasa",
        base_kind: ItemKind::Katana,
        alignment: Some(Alignment::Lawful),
        quest_role: Some(Role::Samurai),
        hit_bonus: 4,
        damage_bonus: 5,
        resistances: Resistances::SEE_INVISIBLE,
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "heart_of_ahriman",
        name: "Heart of Ahriman",
        base_kind: ItemKind::GemRuby,
        alignment: None,
        quest_role: Some(Role::Barbarian),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::MAGIC_RESISTANCE.union(Resistances::SEARCHING),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "longbow_of_diana",
        name: "Longbow of Diana",
        base_kind: ItemKind::Spear,
        alignment: None,
        quest_role: Some(Role::Ranger),
        hit_bonus: 3,
        damage_bonus: 3,
        resistances: Resistances::SEE_INVISIBLE.union(Resistances::REFLECTION),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "eyes_of_the_overworld",
        name: "Eyes of the Overworld",
        base_kind: ItemKind::Blindfold,
        alignment: None,
        quest_role: Some(Role::Monk),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::MAGIC_RESISTANCE
            .union(Resistances::TELEPATHY)
            .union(Resistances::SEE_INVISIBLE),
        emits_light: false,
        drains_life: false,
    },
    ArtifactDef {
        id: "orb_of_detection",
        name: "Orb of Detection",
        base_kind: ItemKind::GemDiamond,
        alignment: None,
        quest_role: Some(Role::Archeologist),
        hit_bonus: 0,
        damage_bonus: 0,
        resistances: Resistances::MAGIC_RESISTANCE.union(Resistances::TELEPATHY),
        emits_light: false,
        drains_life: false,
    },
];

pub fn artifact_def(id: &str) -> Option<&'static ArtifactDef> {
    ARTIFACTS.iter().find(|def| def.id == id)
}

pub fn quest_artifact_for_role(role: Role) -> &'static ArtifactDef {
    ARTIFACTS
        .iter()
        .find(|def| def.quest_role == Some(role))
        .expect("every Role has a unique quest artifact")
}

pub fn eligible_sacrifice_artifacts(
    alignment: Alignment,
    generated: &BTreeSet<String>,
) -> Vec<&'static ArtifactDef> {
    ARTIFACTS
        .iter()
        .filter(|def| {
            def.quest_role.is_none()
                && !generated.contains(def.id)
                && (def.alignment == Some(alignment) || def.alignment.is_none())
        })
        .collect()
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
    pub const fn clamp_attr(val: i16) -> i16 {
        if val < MIN_ATTRIBUTE {
            MIN_ATTRIBUTE
        } else if val > MAX_ATTRIBUTE {
            MAX_ATTRIBUTE
        } else {
            val
        }
    }

    pub const fn with_race_deltas(self, race: Race) -> Self {
        let [ds, dd, dc, di, dw, dh] = race.attribute_deltas();
        Self {
            str_: Self::clamp_attr(self.str_ + ds),
            dex: Self::clamp_attr(self.dex + dd),
            con: Self::clamp_attr(self.con + dc),
            int: Self::clamp_attr(self.int + di),
            wis: Self::clamp_attr(self.wis + dw),
            cha: Self::clamp_attr(self.cha + dh),
        }
    }

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
pub enum SpellSchool {
    Attack,
    Healing,
    Divination,
    Enchantment,
    Clerical,
    Escape,
    Matter,
}

impl SpellSchool {
    pub const ALL: [Self; 7] = [
        Self::Attack,
        Self::Healing,
        Self::Divination,
        Self::Enchantment,
        Self::Clerical,
        Self::Escape,
        Self::Matter,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Healing => "Healing",
            Self::Divination => "Divination",
            Self::Enchantment => "Enchantment",
            Self::Clerical => "Clerical",
            Self::Escape => "Escape",
            Self::Matter => "Matter",
        }
    }
}

macro_rules! define_spells {
    (
        $(
            $variant:ident => {
                school: $school:ident,
                level: $level:expr,
                mana: $mana:expr,
                directional: $dir:expr,
                display: $display:expr,
                wire: $wire:expr,
                toml: $toml:expr $(,)?
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum SpellKind {
            $($variant,)*
        }

        impl SpellKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            pub const fn school(self) -> SpellSchool {
                match self {
                    $(Self::$variant => SpellSchool::$school,)*
                }
            }

            pub const fn spell_level(self) -> u8 {
                match self {
                    $(Self::$variant => $level,)*
                }
            }

            pub const fn mana_cost(self) -> i16 {
                match self {
                    $(Self::$variant => $mana,)*
                }
            }

            pub const fn is_directional(self) -> bool {
                match self {
                    $(Self::$variant => $dir,)*
                }
            }

            pub const fn display_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $display,)*
                }
            }

            pub const fn wire_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire,)*
                }
            }

            pub const fn toml_key(self) -> &'static str {
                match self {
                    $(Self::$variant => $toml,)*
                }
            }

            pub fn from_toml_key(raw: &str) -> Option<Self> {
                match raw {
                    $($toml => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

define_spells! {
    ForceBolt => { school: Attack, level: 1, mana: 4, directional: true, display: "Force Bolt", wire: "FORCE_BOLT", toml: "force_bolt" },
    MagicMissile => { school: Attack, level: 2, mana: 6, directional: true, display: "Magic Missile", wire: "MAGIC_MISSILE", toml: "magic_missile" },
    Fireball => { school: Attack, level: 4, mana: 12, directional: true, display: "Fireball", wire: "FIREBALL", toml: "fireball" },
    ConeOfCold => { school: Attack, level: 4, mana: 12, directional: true, display: "Cone of Cold", wire: "CONE_OF_COLD", toml: "cone_of_cold" },
    LightningBolt => { school: Attack, level: 5, mana: 14, directional: true, display: "Lightning Bolt", wire: "LIGHTNING_BOLT", toml: "lightning_bolt" },
    FingerOfDeath => { school: Attack, level: 7, mana: 20, directional: true, display: "Finger of Death", wire: "FINGER_OF_DEATH", toml: "finger_of_death" },

    Heal => { school: Healing, level: 1, mana: 5, directional: false, display: "Heal", wire: "HEAL", toml: "heal" },
    ExtraHealing => { school: Healing, level: 3, mana: 9, directional: false, display: "Extra Healing", wire: "EXTRA_HEALING", toml: "extra_healing" },
    CureSickness => { school: Healing, level: 3, mana: 8, directional: false, display: "Cure Sickness", wire: "CURE_SICKNESS", toml: "cure_sickness" },
    StoneToFlesh => { school: Healing, level: 3, mana: 9, directional: true, display: "Stone to Flesh", wire: "STONE_TO_FLESH", toml: "stone_to_flesh" },

    Detect => { school: Divination, level: 1, mana: 3, directional: false, display: "Detect", wire: "DETECT", toml: "detect" },
    IdentifySpell => { school: Divination, level: 3, mana: 8, directional: false, display: "Identify", wire: "IDENTIFY_SPELL", toml: "identify_spell" },
    MagicMapping => { school: Divination, level: 5, mana: 14, directional: false, display: "Magic Mapping", wire: "MAGIC_MAPPING", toml: "magic_mapping" },
    DetectTreasure => { school: Divination, level: 4, mana: 10, directional: false, display: "Detect Treasure", wire: "DETECT_TREASURE", toml: "detect_treasure" },

    SleepSpell => { school: Enchantment, level: 1, mana: 5, directional: true, display: "Sleep", wire: "SLEEP_SPELL", toml: "sleep_spell" },
    ConfuseMonster => { school: Enchantment, level: 2, mana: 6, directional: true, display: "Confuse Monster", wire: "CONFUSE_MONSTER", toml: "confuse_monster" },
    CharmMonster => { school: Enchantment, level: 3, mana: 9, directional: true, display: "Charm Monster", wire: "CHARM_MONSTER", toml: "charm_monster" },
    SlowMonster => { school: Enchantment, level: 2, mana: 6, directional: true, display: "Slow Monster", wire: "SLOW_MONSTER", toml: "slow_monster" },

    RemoveCurseSpell => { school: Clerical, level: 3, mana: 9, directional: false, display: "Remove Curse", wire: "REMOVE_CURSE_SPELL", toml: "remove_curse_spell" },
    Protection => { school: Clerical, level: 1, mana: 5, directional: false, display: "Protection", wire: "PROTECTION", toml: "protection" },
    TurnUndead => { school: Clerical, level: 4, mana: 11, directional: false, display: "Turn Undead", wire: "TURN_UNDEAD", toml: "turn_undead" },
    CreateFamiliar => { school: Clerical, level: 5, mana: 15, directional: false, display: "Create Familiar", wire: "CREATE_FAMILIAR", toml: "create_familiar" },

    Jumping => { school: Escape, level: 1, mana: 4, directional: true, display: "Jumping", wire: "JUMPING", toml: "jumping" },
    HasteSelf => { school: Escape, level: 3, mana: 9, directional: false, display: "Haste Self", wire: "HASTE_SELF", toml: "haste_self" },
    TeleportAway => { school: Escape, level: 5, mana: 14, directional: true, display: "Teleport Away", wire: "TELEPORT_AWAY", toml: "teleport_away" },

    Knock => { school: Matter, level: 1, mana: 4, directional: true, display: "Knock", wire: "KNOCK", toml: "knock" },
    WizardLock => { school: Matter, level: 2, mana: 6, directional: true, display: "Wizard Lock", wire: "WIZARD_LOCK", toml: "wizard_lock" },
    DigSpell => { school: Matter, level: 5, mana: 14, directional: true, display: "Dig", wire: "DIG_SPELL", toml: "dig_spell" },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Knight,
    Scout,
    Mage,
    Cleric,
    Valkyrie,
    Wizard,
    Rogue,
    Priest,
    Samurai,
    Barbarian,
    Ranger,
    Monk,
    Archeologist,
}

impl Role {
    pub const ALL: [Self; 13] = [
        Self::Knight,
        Self::Scout,
        Self::Mage,
        Self::Cleric,
        Self::Valkyrie,
        Self::Wizard,
        Self::Rogue,
        Self::Priest,
        Self::Samurai,
        Self::Barbarian,
        Self::Ranger,
        Self::Monk,
        Self::Archeologist,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Knight => "Knight",
            Self::Scout => "Scout",
            Self::Mage => "Mage",
            Self::Cleric => "Cleric",
            Self::Valkyrie => "Valkyrie",
            Self::Wizard => "Wizard",
            Self::Rogue => "Rogue",
            Self::Priest => "Priest",
            Self::Samurai => "Samurai",
            Self::Barbarian => "Barbarian",
            Self::Ranger => "Ranger",
            Self::Monk => "Monk",
            Self::Archeologist => "Archeologist",
        }
    }

    pub const fn allowed_races(self) -> &'static [Race] {
        match self {
            Self::Knight | Self::Samurai | Self::Monk => &[Race::Human],
            Self::Scout | Self::Mage => &[Race::Human, Race::Elf, Race::Gnome],
            Self::Cleric | Self::Priest => &[Race::Human, Race::Elf],
            Self::Valkyrie => &[Race::Human, Race::Dwarf],
            Self::Wizard | Self::Ranger => &[Race::Human, Race::Elf, Race::Gnome, Race::Orc],
            Self::Rogue | Self::Barbarian => &[Race::Human, Race::Orc],
            Self::Archeologist => &[Race::Human, Race::Dwarf, Race::Gnome],
        }
    }

    pub const fn allowed_alignments(self) -> &'static [Alignment] {
        match self {
            Self::Knight | Self::Samurai => &[Alignment::Lawful],
            Self::Scout | Self::Mage | Self::Wizard | Self::Barbarian | Self::Ranger => {
                &[Alignment::Neutral, Alignment::Chaotic]
            }
            Self::Valkyrie | Self::Archeologist => &[Alignment::Lawful, Alignment::Neutral],
            Self::Rogue => &[Alignment::Chaotic],
            Self::Cleric | Self::Priest | Self::Monk => Alignment::ALL.as_slice(),
        }
    }

    pub fn is_valid_combo(self, race: Race, alignment: Alignment) -> bool {
        self.allowed_races().contains(&race) && self.allowed_alignments().contains(&alignment)
    }

    pub const fn base_hp(self) -> i16 {
        match self {
            Self::Knight => 28,
            Self::Scout => 22,
            Self::Mage => 18,
            Self::Cleric => 24,
            Self::Valkyrie => 30,
            Self::Wizard => 16,
            Self::Rogue => 21,
            Self::Priest => 23,
            Self::Samurai => 27,
            Self::Barbarian => 32,
            Self::Ranger => 24,
            Self::Monk => 24,
            Self::Archeologist => 22,
        }
    }
    pub const fn base_hit(self) -> i16 {
        match self {
            Self::Knight => 4,
            Self::Scout => 5,
            Self::Mage => 3,
            Self::Cleric => 3,
            Self::Valkyrie => 4,
            Self::Wizard => 3,
            Self::Rogue => 5,
            Self::Priest => 3,
            Self::Samurai => 5,
            Self::Barbarian => 4,
            Self::Ranger => 5,
            Self::Monk => 4,
            Self::Archeologist => 4,
        }
    }
    pub const fn base_damage(self) -> i16 {
        match self {
            Self::Knight => 2,
            Self::Scout => 1,
            Self::Mage => 0,
            Self::Cleric => 1,
            Self::Valkyrie => 3,
            Self::Wizard => 0,
            Self::Rogue => 1,
            Self::Priest => 1,
            Self::Samurai => 2,
            Self::Barbarian => 3,
            Self::Ranger => 1,
            Self::Monk => 1,
            Self::Archeologist => 1,
        }
    }
    pub const fn base_mp(self) -> i16 {
        match self {
            Self::Mage => 14,
            Self::Cleric => 10,
            Self::Scout => 6,
            Self::Knight => 4,
            Self::Valkyrie => 3,
            Self::Wizard => 16,
            Self::Rogue => 6,
            Self::Priest => 12,
            Self::Samurai => 5,
            Self::Barbarian => 2,
            Self::Ranger => 7,
            Self::Monk => 10,
            Self::Archeologist => 8,
        }
    }
    pub const fn mp_per_level(self) -> i16 {
        match self {
            Self::Mage => 3,
            Self::Cleric => 2,
            Self::Scout => 1,
            Self::Knight => 1,
            Self::Valkyrie => 1,
            Self::Wizard => 4,
            Self::Rogue => 1,
            Self::Priest => 3,
            Self::Samurai => 1,
            Self::Barbarian => 0,
            Self::Ranger => 2,
            Self::Monk => 2,
            Self::Archeologist => 2,
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
            Self::Valkyrie => Attributes {
                str_: 17,
                dex: 13,
                con: 16,
                int: 8,
                wis: 10,
                cha: 10,
            },
            Self::Wizard => Attributes {
                str_: 8,
                dex: 12,
                con: 10,
                int: 18,
                wis: 14,
                cha: 11,
            },
            Self::Rogue => Attributes {
                str_: 12,
                dex: 17,
                con: 11,
                int: 12,
                wis: 9,
                cha: 11,
            },
            Self::Priest => Attributes {
                str_: 11,
                dex: 10,
                con: 13,
                int: 11,
                wis: 17,
                cha: 14,
            },
            Self::Samurai => Attributes {
                str_: 15,
                dex: 15,
                con: 14,
                int: 10,
                wis: 11,
                cha: 11,
            },
            Self::Barbarian => Attributes {
                str_: 18,
                dex: 12,
                con: 17,
                int: 7,
                wis: 8,
                cha: 8,
            },
            Self::Ranger => Attributes {
                str_: 13,
                dex: 16,
                con: 13,
                int: 12,
                wis: 12,
                cha: 10,
            },
            Self::Monk => Attributes {
                str_: 13,
                dex: 16,
                con: 14,
                int: 11,
                wis: 15,
                cha: 12,
            },
            Self::Archeologist => Attributes {
                str_: 12,
                dex: 14,
                con: 13,
                int: 15,
                wis: 13,
                cha: 12,
            },
        }
    }
    pub const fn default_alignment(self) -> Alignment {
        match self {
            Self::Knight | Self::Cleric | Self::Valkyrie | Self::Samurai | Self::Archeologist => {
                Alignment::Lawful
            }
            Self::Scout | Self::Mage | Self::Priest | Self::Barbarian | Self::Monk => {
                Alignment::Neutral
            }
            Self::Wizard | Self::Rogue | Self::Ranger => Alignment::Chaotic,
        }
    }
    pub const fn default_race(self) -> Race {
        Race::Human
    }
    pub const fn known_spells(self) -> &'static [SpellKind] {
        match self {
            Self::Mage | Self::Wizard => &[SpellKind::ForceBolt, SpellKind::Detect],
            Self::Cleric | Self::Priest | Self::Monk => &[SpellKind::Heal, SpellKind::Detect],
            Self::Scout | Self::Rogue | Self::Samurai | Self::Ranger | Self::Archeologist => {
                &[SpellKind::Detect]
            }
            Self::Knight | Self::Valkyrie => &[SpellKind::Heal],
            Self::Barbarian => &[],
        }
    }
    pub fn knows_spell(self, spell: SpellKind) -> bool {
        self.known_spells().contains(&spell)
    }

    pub const fn starting_skill_rank(self, skill: SkillCategory) -> SkillRank {
        match (self, skill) {
            (Self::Knight, SkillCategory::Sword | SkillCategory::Blunt) => SkillRank::Basic,
            (Self::Scout, SkillCategory::Dagger | SkillCategory::Thrown | SkillCategory::Sword) => {
                SkillRank::Basic
            }
            (
                Self::Mage,
                SkillCategory::Dagger | SkillCategory::Staff | SkillCategory::Spellcasting,
            ) => SkillRank::Basic,
            (
                Self::Cleric,
                SkillCategory::Blunt | SkillCategory::Staff | SkillCategory::Spellcasting,
            ) => SkillRank::Basic,
            (Self::Valkyrie, SkillCategory::Sword | SkillCategory::Axe | SkillCategory::Dagger) => {
                SkillRank::Basic
            }
            (Self::Wizard, SkillCategory::Spellcasting) => SkillRank::Skilled,
            (Self::Wizard, SkillCategory::Staff | SkillCategory::Dagger) => SkillRank::Basic,
            (Self::Rogue, SkillCategory::Dagger) => SkillRank::Skilled,
            (Self::Rogue, SkillCategory::Sword | SkillCategory::Thrown) => SkillRank::Basic,
            (
                Self::Priest,
                SkillCategory::Blunt | SkillCategory::Staff | SkillCategory::Spellcasting,
            ) => SkillRank::Basic,
            (Self::Samurai, SkillCategory::Sword) => SkillRank::Skilled,
            (Self::Samurai, SkillCategory::Thrown | SkillCategory::BareHands) => SkillRank::Basic,
            (Self::Barbarian, SkillCategory::Axe) => SkillRank::Skilled,
            (Self::Barbarian, SkillCategory::Sword | SkillCategory::Blunt) => SkillRank::Basic,
            (Self::Ranger, SkillCategory::Thrown) => SkillRank::Skilled,
            (Self::Ranger, SkillCategory::Dagger | SkillCategory::Sword) => SkillRank::Basic,
            (Self::Monk, SkillCategory::BareHands) => SkillRank::Skilled,
            (Self::Monk, SkillCategory::Staff | SkillCategory::Spellcasting) => SkillRank::Basic,
            (
                Self::Archeologist,
                SkillCategory::Blunt | SkillCategory::Dagger | SkillCategory::Thrown,
            ) => SkillRank::Basic,
            _ => SkillRank::Unskilled,
        }
    }

    pub const fn max_skill_rank(self, skill: SkillCategory) -> SkillRank {
        match (self, skill) {
            (Self::Knight, SkillCategory::Sword) => SkillRank::GrandMaster,
            (Self::Knight, SkillCategory::Blunt | SkillCategory::Axe) => SkillRank::Skilled,
            (Self::Knight, _) => SkillRank::Basic,

            (Self::Scout, SkillCategory::Dagger | SkillCategory::Thrown) => SkillRank::Expert,
            (Self::Scout, SkillCategory::Sword | SkillCategory::BareHands) => SkillRank::Skilled,
            (Self::Scout, _) => SkillRank::Basic,

            (Self::Mage, SkillCategory::Spellcasting) => SkillRank::GrandMaster,
            (Self::Mage, SkillCategory::Dagger | SkillCategory::Staff) => SkillRank::Expert,
            (Self::Mage, _) => SkillRank::Basic,

            (Self::Cleric, SkillCategory::Blunt | SkillCategory::Spellcasting) => SkillRank::Expert,
            (Self::Cleric, SkillCategory::Staff) => SkillRank::Skilled,
            (Self::Cleric, _) => SkillRank::Basic,

            (Self::Valkyrie, SkillCategory::Sword) => SkillRank::GrandMaster,
            (Self::Valkyrie, SkillCategory::Axe) => SkillRank::Expert,
            (Self::Valkyrie, SkillCategory::Dagger | SkillCategory::BareHands) => {
                SkillRank::Skilled
            }
            (Self::Valkyrie, _) => SkillRank::Basic,

            (Self::Wizard, SkillCategory::Spellcasting) => SkillRank::GrandMaster,
            (Self::Wizard, SkillCategory::Dagger | SkillCategory::Staff) => SkillRank::Expert,
            (Self::Wizard, _) => SkillRank::Basic,

            (Self::Rogue, SkillCategory::Dagger) => SkillRank::GrandMaster,
            (Self::Rogue, SkillCategory::Sword) => SkillRank::Expert,
            (Self::Rogue, SkillCategory::Thrown | SkillCategory::BareHands) => SkillRank::Skilled,
            (Self::Rogue, _) => SkillRank::Basic,

            (Self::Priest, SkillCategory::Spellcasting | SkillCategory::Blunt) => SkillRank::Expert,
            (Self::Priest, SkillCategory::Staff) => SkillRank::Skilled,
            (Self::Priest, _) => SkillRank::Basic,

            (Self::Samurai, SkillCategory::Sword) => SkillRank::GrandMaster,
            (Self::Samurai, SkillCategory::BareHands) => SkillRank::Master,
            (Self::Samurai, SkillCategory::Thrown | SkillCategory::Dagger) => SkillRank::Expert,
            (Self::Samurai, _) => SkillRank::Basic,

            (Self::Barbarian, SkillCategory::Axe) => SkillRank::GrandMaster,
            (Self::Barbarian, SkillCategory::Sword) => SkillRank::Master,
            (Self::Barbarian, SkillCategory::Blunt | SkillCategory::BareHands) => {
                SkillRank::Skilled
            }
            (Self::Barbarian, SkillCategory::Spellcasting) => SkillRank::Unskilled,
            (Self::Barbarian, _) => SkillRank::Basic,

            (Self::Ranger, SkillCategory::Thrown) => SkillRank::GrandMaster,
            (Self::Ranger, SkillCategory::Dagger | SkillCategory::Sword) => SkillRank::Expert,
            (Self::Ranger, _) => SkillRank::Basic,

            (Self::Monk, SkillCategory::BareHands) => SkillRank::GrandMaster,
            (Self::Monk, SkillCategory::Staff) => SkillRank::Master,
            (Self::Monk, SkillCategory::Spellcasting) => SkillRank::Expert,
            (Self::Monk, _) => SkillRank::Basic,

            (Self::Archeologist, SkillCategory::Blunt | SkillCategory::Dagger) => SkillRank::Expert,
            (Self::Archeologist, SkillCategory::Thrown | SkillCategory::Sword) => {
                SkillRank::Skilled
            }
            (Self::Archeologist, _) => SkillRank::Basic,
        }
    }

    pub const fn unarmed_bonus(self, level: u8, bare_hands: SkillRank) -> (i16, i16) {
        match self {
            Self::Monk => {
                let tier = (level as i16) / 2;
                let hit = 1 + tier + bare_hands.hit_bonus();
                let dmg = 2 + tier + bare_hands.damage_bonus();
                (hit, dmg)
            }
            _ => (bare_hands.hit_bonus(), bare_hands.damage_bonus()),
        }
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
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub artifact_invoke_cooldown: u16,
}

impl CampaignState {
    pub fn new(role: Role, amulet: EntityId) -> Self {
        Self {
            role,
            xp: 0,
            amulet,
            attributes: role.base_attributes().with_race_deltas(role.default_race()),
            alignment: role.default_alignment(),
            race: role.default_race(),
            god_anger: 0,
            prayer_timeout: 0,
            artifact_invoke_cooldown: 0,
        }
    }

    pub fn new_custom(
        role: Role,
        race: Race,
        alignment: Alignment,
        amulet: EntityId,
    ) -> Result<Self, String> {
        if !role.is_valid_combo(race, alignment) {
            return Err(format!(
                "invalid build: {} cannot be {} {}",
                role.display_name(),
                alignment.as_str(),
                race.as_str()
            ));
        }
        Ok(Self {
            role,
            xp: 0,
            amulet,
            attributes: role.base_attributes().with_race_deltas(race),
            alignment,
            race,
            god_anger: 0,
            prayer_timeout: 0,
            artifact_invoke_cooldown: 0,
        })
    }

    pub fn level(self) -> u8 {
        (1 + self.xp.min(MAX_XP) / 20) as u8
    }
    pub fn max_hp(self) -> i16 {
        (self.role.base_hp()
            + self.race.starting_hp_bonus()
            + (4 + self.attributes.con_hp_per_level_bonus()) * i16::from(self.level() - 1))
        .max(4)
    }
    pub fn max_mp(self) -> i16 {
        (self.role.base_mp()
            + self.race.starting_mp_bonus()
            + self.role.mp_per_level() * i16::from(self.level() - 1))
        .max(0)
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
