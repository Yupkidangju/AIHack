use serde::{Deserialize, Serialize};

use crate::domain::combat::{AttackProfile, DamageRoll};

/// [v0.1.0] Phase 7 최소 아이템 종류다. wand/scroll/throw 테스트용 rock을 포함한다.
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
}

/// [v0.1.0] Phase 7 명령 허용 범위를 결정하는 아이템 대분류다.
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
}

/// [v0.1.0] Phase 4는 근접 무기 슬롯 하나만 제공한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Body,
}

/// [v0.1.0] 소비 아이템 효과다. 현재는 치유 물약만 허용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
    RevealLevel,
    IdentifySingle,
    LevelTeleport,
}

/// [v0.1.0] wand의 최소 effect 종류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WandEffect {
    MagicMissile,
}

/// [v0.1.0] 아이템 실데이터다. Phase 7에서는 charge/effect까지 닫는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    pub kind: ItemKind,
    pub class: ItemClass,
    pub glyph: char,
    pub weight: i16,
    pub base_price: u32,
    pub attack_profile: Option<AttackProfile>,
    pub consumable_effect: Option<ConsumableEffect>,
    pub wand_effect: Option<WandEffect>,
    pub max_charges: Option<u8>,
    pub nutrition: Option<i16>,
}

pub fn item_data(kind: ItemKind) -> ItemData {
    match kind {
        ItemKind::Dagger => ItemData {
            kind,
            class: ItemClass::Weapon,
            glyph: ')',
            weight: 10,
            base_price: 4,
            attack_profile: Some(AttackProfile::dagger()),
            consumable_effect: None,
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::FoodRation => ItemData {
            kind,
            class: ItemClass::Food,
            glyph: '%',
            weight: 20,
            base_price: 45,
            attack_profile: None,
            consumable_effect: None,
            wand_effect: None,
            max_charges: None,
            nutrition: Some(800),
        },
        ItemKind::PotionHealing => ItemData {
            kind,
            class: ItemClass::Potion,
            glyph: '!',
            weight: 20,
            base_price: 50,
            attack_profile: None,
            consumable_effect: Some(ConsumableEffect::Heal {
                dice: 1,
                sides: 8,
                bonus: 4,
            }),
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::WandMagicMissile => ItemData {
            kind,
            class: ItemClass::Wand,
            glyph: '/',
            weight: 7,
            base_price: 175,
            attack_profile: None,
            consumable_effect: None,
            wand_effect: Some(WandEffect::MagicMissile),
            max_charges: Some(3),
            nutrition: None,
        },
        ItemKind::ScrollReveal => ItemData {
            kind,
            class: ItemClass::Scroll,
            glyph: '?',
            weight: 5,
            base_price: 60,
            attack_profile: None,
            consumable_effect: Some(ConsumableEffect::RevealLevel),
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::ScrollIdentify => ItemData {
            kind,
            class: ItemClass::Scroll,
            glyph: '?',
            weight: 5,
            base_price: 80,
            attack_profile: None,
            consumable_effect: Some(ConsumableEffect::IdentifySingle),
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::ScrollLevelTeleport => ItemData {
            kind,
            class: ItemClass::Scroll,
            glyph: '?',
            weight: 5,
            base_price: 100,
            attack_profile: None,
            consumable_effect: Some(ConsumableEffect::LevelTeleport),
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::Rock => ItemData {
            kind,
            class: ItemClass::Rock,
            glyph: '*',
            weight: 10,
            base_price: 1,
            attack_profile: Some(AttackProfile::natural("rock", DamageRoll::new(1, 3))),
            consumable_effect: None,
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::ArmorLeather => ItemData {
            kind,
            class: ItemClass::Armor,
            glyph: '[',
            weight: 15,
            base_price: 8,
            attack_profile: None,
            consumable_effect: None,
            wand_effect: None,
            max_charges: None,
            nutrition: None,
        },
        ItemKind::CorpseJackal => ItemData {
            kind,
            class: ItemClass::Corpse,
            glyph: '%',
            weight: 12,
            base_price: 0,
            attack_profile: None,
            consumable_effect: None,
            wand_effect: None,
            max_charges: None,
            nutrition: Some(50),
        },
    }
}

pub const UNARMED_ATTACK: AttackProfile = AttackProfile {
    name: "unarmed",
    hit_bonus: 0,
    damage: DamageRoll { dice: 1, sides: 2 },
};

pub fn shop_base_price(kind: ItemKind) -> u32 {
    item_data(kind).base_price
}
