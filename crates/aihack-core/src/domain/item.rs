use serde::{Deserialize, Serialize};

use crate::domain::combat::AttackProfile;

pub const MIN_ITEM_ENCHANTMENT: i8 = -3;
pub const MAX_ITEM_ENCHANTMENT: i8 = 5;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Body,
    Ring,
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
    pub max_charges: Option<u8>,
    pub nutrition: Option<i16>,
}
