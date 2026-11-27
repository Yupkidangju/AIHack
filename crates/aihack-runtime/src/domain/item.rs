use aihack_content::ContentRegistry;
use aihack_core::{
    domain::combat::{AttackProfile, DamageRoll},
    error::ContentError,
};

pub use aihack_core::domain::item::{
    BucStatus, ConsumableEffect, EquipmentSlot, ItemClass, ItemData, ItemKind, RingEffect,
    ToolAction, WandEffect, MAX_ITEM_ENCHANTMENT, MAX_ITEM_STACK, MIN_ITEM_ENCHANTMENT,
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
    aihack_content::item_kind_from_id(id)
}

pub const UNARMED_ATTACK: AttackProfile = AttackProfile {
    name: "unarmed",
    hit_bonus: 0,
    damage: DamageRoll { dice: 1, sides: 2 },
};

pub fn shop_base_price(kind: ItemKind) -> u32 {
    item_data(kind).base_price
}
