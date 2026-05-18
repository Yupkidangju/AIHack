use crate::{
    core::{event::GameEvent, ids::EntityId, rng::GameRng, world::GameWorld},
    domain::{
        entity::EntityLocation,
        inventory::InventoryLetter,
        item::{ConsumableEffect, EquipmentSlot, ItemClass},
        level::{
            PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID,
            PHASE5_LEVEL2_STAIRS_UP_POS,
        },
    },
    systems::{combat::roll_die, traps},
};

pub fn pickup(world: &mut GameWorld) -> Result<GameEvent, String> {
    let pos = world.player_pos();
    let item = world
        .entities
        .item_at(world.current_level(), pos)
        .ok_or_else(|| "no item at player position".to_string())?;
    let letter = world
        .inventory
        .add_existing_with_next_letter(item)
        .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
    world.entities.set_item_location(
        item,
        EntityLocation::Inventory {
            owner: world.player_id,
        },
    );
    world.entities.set_item_letter(item, letter);

    Ok(GameEvent::ItemPickedUp {
        entity: world.player_id,
        item,
        letter,
    })
}

pub fn wield(world: &mut GameWorld, item: EntityId) -> Result<Option<GameEvent>, String> {
    if world.inventory.equipped_melee == Some(item) {
        return Ok(None);
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.class != ItemClass::Weapon {
        return Err("item is not a weapon".to_string());
    }
    world.inventory.equip_melee(item);
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot: EquipmentSlot::Melee,
    }))
}

pub fn wear(world: &mut GameWorld, item: EntityId) -> Result<Option<GameEvent>, String> {
    if world.inventory.equipped_body == Some(item) {
        return Ok(None);
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.class != ItemClass::Armor {
        return Err("item is not armor".to_string());
    }
    world.inventory.equip_body(item);
    if let Some(stats) = world.entities.actor_stats_mut(world.player_id) {
        stats.ac -= 1;
    }
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot: EquipmentSlot::Body,
    }))
}

pub fn drop(world: &mut GameWorld, item: EntityId) -> Result<GameEvent, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    world.inventory.remove(item);
    let pos = world.player_pos();
    world.entities.set_item_location(
        item,
        EntityLocation::OnMap {
            level: world.current_level(),
            pos,
        },
    );
    Ok(GameEvent::ItemDropped {
        entity: world.player_id,
        item,
        pos,
    })
}

pub fn quaff(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let Some(ConsumableEffect::Heal { dice, sides, bonus }) = data.consumable_effect else {
        return Err("item is not a potion".to_string());
    };

    let raw_heal = (0..dice).map(|_| roll_die(rng, sides)).sum::<i16>() + bonus;
    let player_id = world.player_id;
    let stats = world
        .entities
        .actor_stats_mut(player_id)
        .ok_or_else(|| "player actor stats are missing".to_string())?;
    let before = stats.hp;
    stats.hp = stats.max_hp.min(stats.hp + raw_heal);
    let effective = stats.hp - before;
    let hp_after = stats.hp;

    world.inventory.remove(item);
    world
        .entities
        .set_item_location(item, EntityLocation::Consumed);

    Ok(vec![
        GameEvent::ItemConsumed {
            entity: player_id,
            item,
        },
        GameEvent::EntityHealed {
            entity: player_id,
            amount: effective,
            hp_after,
        },
    ])
}

pub fn read(world: &mut GameWorld, item: EntityId) -> Result<Vec<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let Some(effect) = data.consumable_effect else {
        return Err("item is not a scroll".to_string());
    };

    world.inventory.remove(item);
    world
        .entities
        .set_item_location(item, EntityLocation::Consumed);

    let mut events = vec![GameEvent::ScrollRead {
        entity: world.player_id,
        item,
    }];
    match effect {
        ConsumableEffect::RevealLevel => {
            events.extend(traps::reveal_all_hidden_tiles(world));
        }
        ConsumableEffect::IdentifySingle => {
            if let Some(target) = world
                .inventory
                .entries
                .iter()
                .map(|entry| entry.item)
                .find(|candidate| *candidate != item)
            {
                if let Some(kind) = world
                    .entities
                    .get(target)
                    .and_then(|entity| entity.item())
                    .map(|(kind, _, _, _, _)| kind)
                {
                    world.identify_item_kind(kind);
                    events.push(GameEvent::ItemIdentified {
                        entity: world.player_id,
                        item: target,
                    });
                }
            }
        }
        ConsumableEffect::LevelTeleport => {
            let from = world.current_level();
            let (to_level, to_pos) = if from == PHASE5_LEVEL1_ID {
                (PHASE5_LEVEL2_ID, PHASE5_LEVEL2_STAIRS_UP_POS)
            } else {
                (PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN)
            };
            world.set_player_location(to_level, to_pos);
            events.push(GameEvent::LevelChanged {
                entity: world.player_id,
                from,
                to: to_level,
            });
        }
        ConsumableEffect::Heal { .. } => return Err("item is not a scroll".to_string()),
    }
    Ok(events)
}

pub fn inventory_letter(world: &GameWorld, item: EntityId) -> Option<InventoryLetter> {
    world.inventory.letter_for(item)
}
