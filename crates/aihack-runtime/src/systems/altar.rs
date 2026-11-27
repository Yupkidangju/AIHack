use aihack_core::{
    domain::{
        item::{BucStatus, ItemClass, ItemKind},
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
};

use crate::{systems::items, world::GameWorld};

pub fn is_on_altar(world: &GameWorld) -> bool {
    world.campaign.is_some()
        && world.current_map().tile(world.player_pos()).ok() == Some(TileKind::Altar)
}

pub fn offer(world: &mut GameWorld, item: EntityId) -> Result<Vec<GameEvent>, String> {
    if !is_on_altar(world) {
        return Err("must be standing on an altar to offer".to_string());
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.kind == ItemKind::AmuletAscension {
        return Err("cannot offer the Amulet of Ascension".to_string());
    }
    if let Some((non_empty, active, buc)) = world.entities.get(item).map(|entity| {
        (
            !entity.item_contained_items().unwrap_or_default().is_empty(),
            entity.item_tool_active() == Some(true),
            entity.item_buc(),
        )
    }) {
        if non_empty {
            return Err("cannot offer a non-empty container".to_string());
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

    items::consume_one_from_inventory(world, item)?;
    let player_id = world.player_id;
    let mut events = vec![GameEvent::ItemConsumed {
        entity: player_id,
        item,
    }];

    if matches!(data.class, ItemClass::Corpse | ItemClass::Food) {
        let state = world.state_mut();
        state.luck = state.luck.saturating_add(1).min(3);
        state.prayer_cooldown = state.prayer_cooldown.saturating_sub(10);
        state.mp = state.mp.saturating_add(4).min(state.max_mp);
        if let Some(weapon_id) = world.inventory.equipped_melee {
            if world
                .entities
                .get(weapon_id)
                .and_then(|entity| entity.item_buc())
                == Some(BucStatus::Uncursed)
            {
                let entities = &mut world.state_mut().entities;
                entities.set_item_buc(weapon_id, BucStatus::Blessed);
                entities.set_item_buc_known(weapon_id, true);
            }
        }
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: "Your offering pleases the gods.".to_string(),
        });
    } else {
        world.identify_item_kind(data.kind);
        if world.inventory.contains(item) {
            let entities = &mut world.state_mut().entities;
            entities.set_item_buc_known(item, true);
            entities.set_item_enchant_known(item, true);
        }
        let state = world.state_mut();
        state.mp = state.mp.saturating_add(2).min(state.max_mp);
        state.prayer_cooldown = state.prayer_cooldown.saturating_sub(5);
        events.push(GameEvent::ItemIdentified {
            entity: player_id,
            item,
        });
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: "The altar reveals the nature of your offering.".to_string(),
        });
    }

    Ok(events)
}

pub fn pray_at_altar(world: &mut GameWorld) -> Vec<GameEvent> {
    let cursed_items: Vec<EntityId> = world
        .inventory
        .entries
        .iter()
        .map(|entry| entry.item)
        .filter(|id| {
            world.entities.get(*id).and_then(|entity| entity.item_buc()) == Some(BucStatus::Cursed)
        })
        .collect();
    for id in cursed_items {
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc(id, BucStatus::Uncursed);
        entities.set_item_buc_known(id, true);
    }
    let plain_water: Vec<EntityId> = world
        .inventory
        .entries
        .iter()
        .map(|entry| entry.item)
        .filter(|id| {
            world.entities.get(*id).is_some_and(|entity| {
                entity
                    .item()
                    .is_some_and(|(k, _, _, _, _)| k == ItemKind::PotionWater)
                    && entity.item_buc() == Some(BucStatus::Uncursed)
            })
        })
        .collect();
    for id in plain_water {
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc(id, BucStatus::Blessed);
        entities.set_item_buc_known(id, true);
    }
    let state = world.state_mut();
    state.mp = state.max_mp;

    let mut events = Vec::new();
    let wis_bonus = world
        .campaign
        .map(|c| c.attributes.wis_heal_bonus())
        .unwrap_or(0);
    let player_id = world.player_id;
    if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
        let before = stats.hp;
        stats.hp = stats
            .hp
            .saturating_add(6_i16.saturating_add(wis_bonus))
            .min(stats.max_hp);
        let amount = stats.hp - before;
        if amount > 0 {
            events.push(GameEvent::EntityHealed {
                entity: player_id,
                amount,
                hp_after: stats.hp,
            });
        }
    }
    events.push(GameEvent::Message {
        priority: MessagePriority::Info,
        text: "The altar radiates holy warmth!".to_string(),
    });
    events
}
