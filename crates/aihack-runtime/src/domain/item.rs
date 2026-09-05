use aihack_content::ContentRegistry;
use aihack_core::{
    domain::combat::{AttackProfile, DamageRoll},
    error::ContentError,
};

pub use aihack_core::domain::item::{
    ConsumableEffect, EquipmentSlot, ItemClass, ItemData, ItemKind, WandEffect,
};

pub fn item_data(kind: ItemKind) -> ItemData {
    try_item_data(kind).expect("embedded content registry is validated before item creation")
}

pub fn try_item_data(kind: ItemKind) -> Result<ItemData, ContentError> {
    try_item_data_from_registry(kind, aihack_content::registry()?)
}

pub fn try_item_data_from_registry(
    kind: ItemKind,
    registry: &ContentRegistry,
) -> Result<ItemData, ContentError> {
    aihack_content::item_data_from_registry(kind, registry)
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
        "item.quest.ascension" => Ok(ItemKind::AmuletAscension),
        _ => Err(ContentError::UnknownReference {
            owner: "item kind".to_owned(),
            target: id.to_owned(),
        }),
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
