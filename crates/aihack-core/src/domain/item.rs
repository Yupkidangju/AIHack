use serde::{Deserialize, Serialize};

use crate::domain::combat::AttackProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Dagger,
    FoodRation,
    PotionHealing,
    WandMagicMissile,
    ScrollReveal,
    ScrollIdentify,
    ScrollLevelTeleport,
    Rock,
    ArmorLeather,
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
    Corpse,
    Quest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
    RevealLevel,
    IdentifySingle,
    LevelTeleport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WandEffect {
    MagicMissile,
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
    pub max_charges: Option<u8>,
    pub nutrition: Option<i16>,
}
