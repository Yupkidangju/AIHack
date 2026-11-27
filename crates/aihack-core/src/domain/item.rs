use serde::{Deserialize, Serialize};

use crate::domain::combat::AttackProfile;

pub const MIN_ITEM_ENCHANTMENT: i8 = -3;
pub const MAX_ITEM_ENCHANTMENT: i8 = 5;
pub const MAX_ITEM_STACK: u16 = 99;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BucStatus {
    Blessed,
    #[default]
    Uncursed,
    Cursed,
}

impl BucStatus {
    pub fn is_uncursed(&self) -> bool {
        matches!(self, Self::Uncursed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Dagger,
    ShortSword,
    Mace,
    FoodRation,
    PotionHealing,
    PotionExtraHealing,
    PotionCure,
    WandMagicMissile,
    WandFire,
    ScrollReveal,
    ScrollIdentify,
    ScrollLevelTeleport,
    ScrollCharging,
    ScrollEnchantWeapon,
    ScrollEnchantArmor,
    ScrollRemoveCurse,
    Rock,
    ArmorLeather,
    ArmorChainMail,
    RingPoisonResistance,
    RingFireResistance,
    RingProtection,
    CorpseJackal,
    AmuletAscension,
    ArmorShield,
    ArmorHelm,
    ArmorCloak,
    ArmorGloves,
    ArmorBoots,
    AmuletEsp,
    PotionSpeed,
    PotionBlindness,
    PotionConfusion,
    PotionHallucination,
    PotionInvisibility,
    PotionLevitation,
    Corpse,
    SkeletonKey,
    LockPick,
    PickAxe,
    DwarvishMattock,
    TinWhistle,
    OilLamp,
    Blindfold,
    UnicornHorn,
    Sack,
    BagOfHolding,
    Chest,
    PotionWater,
    ScrollBlank,
    LongSword,
    Katana,
    BattleAxe,
    Quarterstaff,
    BroadSword,
}

impl ItemKind {
    pub const fn is_stackable(self) -> bool {
        matches!(
            self,
            Self::FoodRation
                | Self::PotionHealing
                | Self::PotionExtraHealing
                | Self::PotionCure
                | Self::PotionSpeed
                | Self::PotionBlindness
                | Self::PotionConfusion
                | Self::PotionHallucination
                | Self::PotionInvisibility
                | Self::PotionLevitation
                | Self::PotionWater
                | Self::ScrollReveal
                | Self::ScrollIdentify
                | Self::ScrollLevelTeleport
                | Self::ScrollCharging
                | Self::ScrollEnchantWeapon
                | Self::ScrollEnchantArmor
                | Self::ScrollRemoveCurse
                | Self::ScrollBlank
                | Self::Rock
        )
    }

    pub const fn has_randomized_appearance(self) -> bool {
        matches!(
            self,
            Self::PotionHealing
                | Self::PotionExtraHealing
                | Self::PotionCure
                | Self::PotionSpeed
                | Self::PotionBlindness
                | Self::PotionConfusion
                | Self::PotionHallucination
                | Self::PotionInvisibility
                | Self::PotionLevitation
                | Self::ScrollReveal
                | Self::ScrollIdentify
                | Self::ScrollLevelTeleport
                | Self::ScrollCharging
                | Self::ScrollEnchantWeapon
                | Self::ScrollEnchantArmor
                | Self::ScrollRemoveCurse
                | Self::RingPoisonResistance
                | Self::RingFireResistance
                | Self::RingProtection
                | Self::AmuletEsp
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemClass {
    Weapon,
    Food,
    Potion,
    Wand,
    Scroll,
    Rock,
    Armor,
    Ring,
    Corpse,
    Quest,
    Amulet,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Helm,
    Body,
    Cloak,
    Gloves,
    Boots,
    #[serde(alias = "Ring")]
    LeftRing,
    RightRing,
    Amulet,
}

impl EquipmentSlot {
    pub const ALL: [Self; 10] = [
        Self::Melee,
        Self::Shield,
        Self::Helm,
        Self::Body,
        Self::Cloak,
        Self::Gloves,
        Self::Boots,
        Self::LeftRing,
        Self::RightRing,
        Self::Amulet,
    ];

    pub const ALL_DEFENSE: [Self; 9] = [
        Self::Shield,
        Self::Helm,
        Self::Body,
        Self::Cloak,
        Self::Gloves,
        Self::Boots,
        Self::LeftRing,
        Self::RightRing,
        Self::Amulet,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Melee => "weapon",
            Self::Shield => "shield",
            Self::Helm => "helm",
            Self::Body => "body armor",
            Self::Cloak => "cloak",
            Self::Gloves => "gloves",
            Self::Boots => "boots",
            Self::LeftRing => "left ring",
            Self::RightRing => "right ring",
            Self::Amulet => "amulet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemMaterial {
    Iron,
    Steel,
    Leather,
    Cloth,
    Wood,
    Glass,
    Paper,
    Mineral,
    Mithril,
    Dragonhide,
    Gold,
    Silver,
    Organic,
    Gemstone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolAction {
    Light,
    Unlock,
    Dig,
    Container,
    Music,
    Magic,
    Blindfold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Blind,
    Confused,
    Stunned,
    Hallucinating,
    Fast,
    Levitation,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingEffect {
    PoisonResistance,
    FireResistance,
    Protection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
    RevealLevel,
    IdentifySingle,
    LevelTeleport,
    RechargeWand,
    EnchantWeapon,
    EnchantArmor,
    RemoveCurse,
    CureStatus,
    ApplyStatus { effect: StatusEffectKind, turns: u8 },
    Water,
    BlankPaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WandEffect {
    MagicMissile,
    Fire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    pub kind: ItemKind,
    pub class: ItemClass,
    pub glyph: char,
    pub weight: i16,
    pub base_price: u32,
    #[serde(default)]
    pub ac_bonus: i16,
    pub attack_profile: Option<AttackProfile>,
    pub consumable_effect: Option<ConsumableEffect>,
    pub wand_effect: Option<WandEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_effect: Option<RingEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_slot: Option<EquipmentSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_action: Option<ToolAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_capacity: Option<i16>,
    pub max_charges: Option<u8>,
    pub nutrition: Option<i16>,
}

impl ItemData {
    pub const fn effective_armor_slot(&self) -> Option<EquipmentSlot> {
        if let Some(slot) = self.armor_slot {
            return Some(slot);
        }
        if matches!(self.class, ItemClass::Armor) {
            return Some(EquipmentSlot::Body);
        }
        None
    }
}
