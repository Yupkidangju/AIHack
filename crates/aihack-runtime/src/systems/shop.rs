use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        entity::EntityLocation,
        item::{BucStatus, ItemClass, ItemData, ItemKind, MAX_ITEM_STACK},
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

pub fn buy_price_for_world(world: &GameWorld, kind: ItemKind) -> Result<u32, String> {
    let base = buy_price(kind, world.content_registry())?;
    Ok(world
        .campaign
        .map(|c| c.attributes.cha_buy_price(base))
        .unwrap_or(base))
}

pub fn sell_price(data: &ItemData) -> u32 {
    (data.base_price / 2).max(1)
}

pub fn sell_price_for_world(world: &GameWorld, data: &ItemData) -> u32 {
    world
        .campaign
        .map(|c| c.attributes.cha_sell_price(data.base_price))
        .unwrap_or_else(|| sell_price(data))
}

pub fn can_stack_buy(world: &GameWorld, kind: ItemKind) -> bool {
    find_buy_stack(world, kind).is_some()
}

pub fn can_buy_slot(world: &GameWorld, kind: ItemKind) -> bool {
    world.inventory.has_free_slot() || can_stack_buy(world, kind)
}

fn find_buy_stack(
    world: &GameWorld,
    kind: ItemKind,
) -> Option<(
    EntityId,
    aihack_core::domain::inventory::InventoryLetter,
    u16,
)> {
    if world.campaign.is_none() || !kind.is_stackable() {
        return None;
    }
    world.inventory.entries.iter().find_map(|entry| {
        let entity = world.entities.get(entry.item)?;
        let (existing_kind, _, _, _, _) = entity.item()?;
        let qty = entity.item_quantity().unwrap_or(1);
        if existing_kind == kind
            && entity.item_buc().unwrap_or(BucStatus::Uncursed) == BucStatus::Uncursed
            && entity.item_enchantment().unwrap_or(0) == 0
            && qty < MAX_ITEM_STACK
        {
            Some((entry.item, entry.letter, qty + 1))
        } else {
            None
        }
    })
}

fn shop_item_log_name(world: &GameWorld, kind: ItemKind, class: ItemClass) -> String {
    if world.campaign.is_some()
        && !world.is_item_identified(kind)
        && kind.has_randomized_appearance()
    {
        match class {
            ItemClass::Potion => "unidentified potion".to_string(),
            ItemClass::Scroll => "unidentified scroll".to_string(),
            ItemClass::Ring => "unidentified ring".to_string(),
            _ => "unidentified item".to_string(),
        }
    } else {
        format!("{kind:?}")
    }
}

pub fn buy(world: &mut GameWorld, kind: ItemKind) -> Result<Vec<GameEvent>, String> {
    if !is_on_shop(world) {
        return Err("must be standing in a shop to buy".to_string());
    }
    let price = buy_price_for_world(world, kind)?;
    if world.gold < price {
        return Err("not enough gold".to_string());
    }
    let data = crate::domain::item::try_item_data_from_registry(kind, world.content_registry())
        .map_err(|error| error.to_string())?;
    let log_name = shop_item_log_name(world, kind, data.class);
    if let Some((item_id, letter, next_qty)) = find_buy_stack(world, kind) {
        let player_id = world.player_id;
        let entities = &mut world.state_mut().entities;
        entities.set_item_quantity(item_id, next_qty);
        entities.set_item_buc_known(item_id, true);
        entities.set_item_enchant_known(item_id, true);
        world.state_mut().gold -= price;
        return Ok(vec![
            GameEvent::ItemPickedUp {
                entity: player_id,
                item: item_id,
                letter,
            },
            GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("Bought {log_name} for {price} gold."),
            },
        ]);
    }
    if !world.inventory.has_free_slot() {
        return Err("inventory letter capacity exceeded".to_string());
    }
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
    world.state_mut().entities.set_item_buc_known(item_id, true);
    world
        .state_mut()
        .entities
        .set_item_enchant_known(item_id, true);
    world.state_mut().gold -= price;
    Ok(vec![
        GameEvent::ItemPickedUp {
            entity: player_id,
            item: item_id,
            letter,
        },
        GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("Bought {log_name} for {price} gold."),
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
    if let Some((non_empty, active, buc)) = world.entities.get(item).map(|entity| {
        (
            !entity.item_contained_items().unwrap_or_default().is_empty(),
            entity.item_tool_active() == Some(true),
            entity.item_buc(),
        )
    }) {
        if non_empty {
            return Err("cannot sell a non-empty container".to_string());
        }
        if data.kind == ItemKind::Blindfold && active {
            if buc == Some(BucStatus::Cursed) {
                world.state_mut().entities.set_item_buc_known(item, true);
                return Err("The blindfold is cursed and stuck to your face!".to_string());
            }
            world.state_mut().entities.set_item_tool_active(item, false);
        }
        if data.kind == ItemKind::OilLamp && active {
            world.state_mut().entities.set_item_tool_active(item, false);
        }
    }
    let price = sell_price_for_world(world, &data);
    let log_name = shop_item_log_name(world, data.kind, data.class);
    items::consume_one_from_inventory(world, item)?;
    world.state_mut().gold = world.gold.saturating_add(price);
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("Sold {log_name} for {price} gold."),
    }])
}
