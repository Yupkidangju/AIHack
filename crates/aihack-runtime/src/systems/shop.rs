use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        entity::EntityLocation,
        item::{ItemData, ItemKind},
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
};

use crate::{systems::items, world::GameWorld};

pub const SHOP_CATALOG: [ItemKind; 9] = [
    ItemKind::FoodRation,
    ItemKind::PotionHealing,
    ItemKind::PotionCure,
    ItemKind::ScrollEnchantWeapon,
    ItemKind::ScrollEnchantArmor,
    ItemKind::ScrollRemoveCurse,
    ItemKind::RingProtection,
    ItemKind::RingPoisonResistance,
    ItemKind::RingFireResistance,
];

pub fn is_on_shop(world: &GameWorld) -> bool {
    world.campaign.is_some()
        && world.current_map().tile(world.player_pos()).ok() == Some(TileKind::Shop)
}

pub fn buy_price(kind: ItemKind, registry: &ContentRegistry) -> Result<u32, String> {
    if !SHOP_CATALOG.contains(&kind) {
        return Err("item is not sold in this shop".to_string());
    }
    let data = crate::domain::item::try_item_data_from_registry(kind, registry)
        .map_err(|error| error.to_string())?;
    Ok(data.base_price.max(10))
}

pub fn sell_price(data: &ItemData) -> u32 {
    (data.base_price / 2).max(1)
}

pub fn buy(world: &mut GameWorld, kind: ItemKind) -> Result<Vec<GameEvent>, String> {
    if !is_on_shop(world) {
        return Err("must be standing in a shop to buy".to_string());
    }
    if world.inventory.next_letter_index >= 26 {
        return Err("inventory letter capacity exceeded".to_string());
    }
    let price = buy_price(kind, world.content_registry())?;
    if world.gold < price {
        return Err("not enough gold".to_string());
    }
    let data = crate::domain::item::try_item_data_from_registry(kind, world.content_registry())
        .map_err(|error| error.to_string())?;
    let player_id = world.player_id;
    let item_id = world
        .state_mut()
        .entities
        .spawn_item_with_data(kind, data, EntityLocation::Inventory { owner: player_id })
        .map_err(|error| error.to_string())?;
    let letter = world
        .state_mut()
        .inventory
        .add_existing_with_next_letter(item_id)
        .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
    world.state_mut().entities.set_item_letter(item_id, letter);
    world.state_mut().gold -= price;
    Ok(vec![
        GameEvent::ItemPickedUp {
            entity: player_id,
            item: item_id,
            letter,
        },
        GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("Bought {:?} for {price} gold.", kind),
        },
    ])
}

pub fn sell(world: &mut GameWorld, item: EntityId) -> Result<Vec<GameEvent>, String> {
    if !is_on_shop(world) {
        return Err("must be standing in a shop to sell".to_string());
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.kind == ItemKind::AmuletAscension {
        return Err("cannot sell the Amulet of Ascension".to_string());
    }
    let price = sell_price(&data);
    items::remove_inventory_item(world, item)?;
    world
        .state_mut()
        .entities
        .set_item_location(item, EntityLocation::Consumed);
    world.state_mut().gold = world.gold.saturating_add(price);
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("Sold {:?} for {price} gold.", data.kind),
    }])
}
