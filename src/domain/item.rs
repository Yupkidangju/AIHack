use serde::{Deserialize, Serialize};

use crate::domain::combat::{AttackProfile, DamageRoll};

/// [v0.1.0] Phase 4 최소 아이템 종류다. 현재는 무기, 식량, 치유 물약만 포함한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Dagger,
    FoodRation,
    PotionHealing,
}

/// [v0.1.0] Phase 4 명령 허용 범위를 결정하는 아이템 대분류다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemClass {
    Weapon,
    Food,
    Potion,
}

/// [v0.1.0] Phase 4는 근접 무기 슬롯 하나만 제공한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
}

/// [v0.1.0] 소비 아이템 효과다. 현재는 치유 물약만 허용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
}

/// [v0.1.0] 아이템 실데이터다. Phase 4에서는 식량 사용 효과를 구현하지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    pub kind: ItemKind,
    pub class: ItemClass,
    pub glyph: char,
    pub weight: i16,
    pub attack_profile: Option<AttackProfile>,
    pub consumable_effect: Option<ConsumableEffect>,
    pub nutrition: Option<i16>,
}

pub fn item_data(kind: ItemKind) -> ItemData {
    match kind {
        ItemKind::Dagger => ItemData {
            kind,
            class: ItemClass::Weapon,
            glyph: ')',
            weight: 10,
            attack_profile: Some(AttackProfile::dagger()),
            consumable_effect: None,
            nutrition: None,
        },
        ItemKind::FoodRation => ItemData {
            kind,
            class: ItemClass::Food,
            glyph: '%',
            weight: 20,
            attack_profile: None,
            consumable_effect: None,
            nutrition: Some(800),
        },
        ItemKind::PotionHealing => ItemData {
            kind,
            class: ItemClass::Potion,
            glyph: '!',
            weight: 20,
            attack_profile: None,
            consumable_effect: Some(ConsumableEffect::Heal {
                dice: 1,
                sides: 8,
                bonus: 4,
            }),
            nutrition: None,
        },
    }
}

pub const UNARMED_ATTACK: AttackProfile = AttackProfile {
    name: "unarmed",
    hit_bonus: 0,
    damage: DamageRoll { dice: 1, sides: 2 },
};
