use serde::{Deserialize, Serialize};

use crate::{
    core::error::ContentError,
    data,
    domain::combat::{AttackProfile, DamageRoll},
};

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
    try_item_data(kind).expect("embedded content registry is validated before item creation")
}

pub fn try_item_data(kind: ItemKind) -> Result<ItemData, ContentError> {
    let id = item_id(kind);
    let definition = data::registry()?
        .item(id)
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "item factory".to_owned(),
            target: id.to_owned(),
        })?;
    let class = match definition.kind.as_str() {
        "weapon" => ItemClass::Weapon,
        "food" => ItemClass::Food,
        "potion" => ItemClass::Potion,
        "wand" => ItemClass::Wand,
        "scroll" => ItemClass::Scroll,
        "armor" => ItemClass::Armor,
        "corpse" => ItemClass::Corpse,
        _ => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: definition.kind.clone(),
            })
        }
    };
    let class = if kind == ItemKind::Rock {
        ItemClass::Rock
    } else {
        class
    };
    let glyph = definition
        .glyph
        .chars()
        .next()
        .ok_or_else(|| ContentError::Parse {
            file: id.to_owned(),
            message: "glyph must contain one character".to_owned(),
        })?;
    let attack_profile = definition
        .damage
        .as_deref()
        .map(|value| {
            let damage = parse_damage(value)?;
            let name = match kind {
                ItemKind::Dagger => "dagger",
                ItemKind::Rock => "rock",
                _ => "weapon",
            };
            Ok(AttackProfile {
                name,
                hit_bonus: definition.hit_bonus.unwrap_or_default(),
                damage,
            })
        })
        .transpose()?;
    let consumable_effect = match definition.effect.as_deref() {
        Some("heal_1d8_plus_4") => Some(ConsumableEffect::Heal {
            dice: 1,
            sides: 8,
            bonus: 4,
        }),
        Some("reveal") => Some(ConsumableEffect::RevealLevel),
        Some("identify") => Some(ConsumableEffect::IdentifySingle),
        Some("teleport") => Some(ConsumableEffect::LevelTeleport),
        Some("magic_missile") | None => None,
        Some(effect) => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: effect.to_owned(),
            })
        }
    };
    Ok(ItemData {
        kind,
        class,
        glyph,
        weight: definition.weight,
        base_price: definition.base_price.unwrap_or_default() as u32,
        attack_profile,
        consumable_effect,
        wand_effect: (definition.effect.as_deref() == Some("magic_missile"))
            .then_some(WandEffect::MagicMissile),
        max_charges: definition.charges,
        nutrition: definition.nutrition,
    })
}

pub fn item_kind_from_id(id: &str) -> Result<ItemKind, ContentError> {
    match id {
        "item.weapon.dagger" => Ok(ItemKind::Dagger),
        "item.food.ration" => Ok(ItemKind::FoodRation),
        "item.potion.healing" => Ok(ItemKind::PotionHealing),
        "item.wand.magic_missile" => Ok(ItemKind::WandMagicMissile),
        "item.scroll.reveal" => Ok(ItemKind::ScrollReveal),
        "item.scroll.identify" => Ok(ItemKind::ScrollIdentify),
        "item.scroll.teleport" => Ok(ItemKind::ScrollLevelTeleport),
        "item.weapon.rock" => Ok(ItemKind::Rock),
        "item.armor.leather" => Ok(ItemKind::ArmorLeather),
        "item.corpse.jackal" => Ok(ItemKind::CorpseJackal),
        _ => Err(ContentError::UnknownReference {
            owner: "item kind".to_owned(),
            target: id.to_owned(),
        }),
    }
}

fn item_id(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Dagger => "item.weapon.dagger",
        ItemKind::FoodRation => "item.food.ration",
        ItemKind::PotionHealing => "item.potion.healing",
        ItemKind::WandMagicMissile => "item.wand.magic_missile",
        ItemKind::ScrollReveal => "item.scroll.reveal",
        ItemKind::ScrollIdentify => "item.scroll.identify",
        ItemKind::ScrollLevelTeleport => "item.scroll.teleport",
        ItemKind::Rock => "item.weapon.rock",
        ItemKind::ArmorLeather => "item.armor.leather",
        ItemKind::CorpseJackal => "item.corpse.jackal",
    }
}

fn parse_damage(value: &str) -> Result<DamageRoll, ContentError> {
    if value == "0" {
        return Ok(DamageRoll::none());
    }
    let Some((dice, sides)) = value.split_once('d') else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    let (Ok(dice), Ok(sides)) = (dice.parse(), sides.parse()) else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    Ok(DamageRoll::new(dice, sides))
}

pub const UNARMED_ATTACK: AttackProfile = AttackProfile {
    name: "unarmed",
    hit_bonus: 0,
    damage: DamageRoll { dice: 1, sides: 2 },
};

pub fn shop_base_price(kind: ItemKind) -> u32 {
    item_data(kind).base_price
}
