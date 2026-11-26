use aihack_core::{
    domain::{
        entity::EntityLocation,
        item::{BucStatus, ConsumableEffect, EquipmentSlot, ItemClass, MAX_ITEM_ENCHANTMENT},
        level::{
            PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID,
            PHASE5_LEVEL2_STAIRS_UP_POS,
        },
        player::adventurer_template,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    systems::{combat::roll_die, traps},
    world::GameWorld,
};

pub fn pickup(world: &mut GameWorld) -> Result<GameEvent, String> {
    let pos = world.player_pos();
    let item = world
        .entities
        .item_at(world.current_level(), pos)
        .ok_or_else(|| "no item at player position".to_string())?;
    let letter = world
        .state_mut()
        .inventory
        .add_existing_with_next_letter(item)
        .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
    let player_id = world.player_id;
    world
        .state_mut()
        .entities
        .set_item_location(item, EntityLocation::Inventory { owner: player_id });
    world.state_mut().entities.set_item_letter(item, letter);

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
    if let Some(previous) = world.inventory.equipped_melee {
        if world.entities.get(previous).and_then(|e| e.item_buc()) == Some(BucStatus::Cursed) {
            return Err("equipped weapon is cursed and cannot be unwielded".to_string());
        }
    }
    world.state_mut().inventory.equip_melee(item);
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
    let _ = compute_ac_with(world, Some(item), world.inventory.equipped_ring)?;
    if let Some(previous) = world.inventory.equipped_body {
        unequip_body(world, previous)?;
    }
    world.state_mut().inventory.equip_body(item);
    recompute_player_ac(world)?;
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
    remove_inventory_item(world, item)?;
    let level = world.current_level();
    let pos = world.player_pos();
    world
        .state_mut()
        .entities
        .set_item_location(item, EntityLocation::OnMap { level, pos });
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
    let buc = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        .unwrap_or(BucStatus::Uncursed);
    let raw_heal = match data.consumable_effect {
        Some(ConsumableEffect::Heal { dice, sides, bonus }) => {
            let base = (0..dice)
                .map(|_| i32::from(roll_die(rng, sides)))
                .sum::<i32>()
                + i32::from(bonus);
            match buc {
                BucStatus::Blessed => base + 4,
                BucStatus::Uncursed => base,
                BucStatus::Cursed => (base - 4).max(1),
            }
        }
        Some(ConsumableEffect::CureStatus) => {
            let state = world.state_mut();
            state.poison_turns = 0;
            state.paralysis_turns = 0;
            state.hallucinating = false;
            match buc {
                BucStatus::Cursed => 1,
                BucStatus::Uncursed => i32::from(roll_die(rng, 6)) + 4,
                BucStatus::Blessed => i32::from(roll_die(rng, 6)) + 8,
            }
        }
        _ => return Err("item is not a potion".to_string()),
    };

    let player_id = world.player_id;
    let stats = world
        .state_mut()
        .entities
        .actor_stats_mut(player_id)
        .ok_or_else(|| "player actor stats are missing".to_string())?;
    let before = stats.hp;
    let healed = i32::from(stats.hp)
        .saturating_add(raw_heal)
        .min(i32::from(stats.max_hp));
    stats.hp = healed.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let effective = stats.hp - before;
    let hp_after = stats.hp;

    remove_inventory_item(world, item)?;
    world
        .state_mut()
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

/// 음식과 시체의 콘텐츠 영양값을 월드 허기 상태로 전달한다.
pub fn eat(world: &mut GameWorld, item: EntityId) -> Result<Vec<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if !matches!(data.class, ItemClass::Food | ItemClass::Corpse) {
        return Err("item is not edible".to_string());
    }
    let nutrition = data
        .nutrition
        .filter(|nutrition| *nutrition > 0)
        .ok_or_else(|| "edible item has no positive nutrition".to_string())?;

    world.state_mut().nutrition = world.nutrition.saturating_add(nutrition);
    remove_inventory_item(world, item)?;
    world
        .state_mut()
        .entities
        .set_item_location(item, EntityLocation::Consumed);

    Ok(vec![GameEvent::ItemConsumed {
        entity: world.player_id,
        item,
    }])
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
    if matches!(
        effect,
        ConsumableEffect::Heal { .. } | ConsumableEffect::CureStatus
    ) {
        return Err("item is not a scroll".to_string());
    }
    let scroll_buc = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        .unwrap_or(BucStatus::Uncursed);

    remove_inventory_item(world, item)?;
    world
        .state_mut()
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
            let to_pos = if world.campaign.is_some() {
                world
                    .levels
                    .stairs_up_pos(to_level)
                    .ok_or("teleport landing is missing")?
            } else {
                to_pos
            };
            world.set_player_location(to_level, to_pos);
            events.push(GameEvent::LevelChanged {
                entity: world.player_id,
                from,
                to: to_level,
            });
        }
        ConsumableEffect::RechargeWand => {
            let mut candidate: Option<(EntityId, u8)> = None;
            let mut fallback: Option<(EntityId, u8)> = None;
            for entry in &world.inventory.entries {
                if let Some((_, data, _, _, charges)) = world
                    .entities
                    .get(entry.item)
                    .and_then(|entity| entity.item())
                {
                    if data.wand_effect.is_some() {
                        if let Some(max_charges) = data.max_charges {
                            if fallback.is_none() {
                                fallback = Some((entry.item, max_charges));
                            }
                            if charges.unwrap_or(0) < max_charges {
                                candidate = Some((entry.item, max_charges));
                                break;
                            }
                        }
                    }
                }
            }
            if let Some((wand_id, max_charges)) = candidate.or(fallback) {
                match scroll_buc {
                    BucStatus::Cursed => {
                        world
                            .state_mut()
                            .entities
                            .set_item_charges(wand_id, Some(0));
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: "Your wand vibrates weakly and loses its charges.".to_string(),
                        });
                    }
                    BucStatus::Blessed => {
                        world
                            .state_mut()
                            .entities
                            .set_item_charges(wand_id, Some(max_charges));
                        if world.entities.get(wand_id).and_then(|e| e.item_buc())
                            == Some(BucStatus::Cursed)
                        {
                            world
                                .state_mut()
                                .entities
                                .set_item_buc(wand_id, BucStatus::Uncursed);
                        }
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!("Recharged wand to {max_charges} charges"),
                        });
                    }
                    BucStatus::Uncursed => {
                        world
                            .state_mut()
                            .entities
                            .set_item_charges(wand_id, Some(max_charges));
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!("Recharged wand to {max_charges} charges"),
                        });
                    }
                }
            }
        }
        ConsumableEffect::EnchantWeapon => {
            if let Some(weapon_id) = world.inventory.equipped_melee {
                let (current_enchant, current_buc) = world
                    .entities
                    .get(weapon_id)
                    .map(|entity| {
                        (
                            entity.item_enchantment().unwrap_or(0),
                            entity.item_buc().unwrap_or(BucStatus::Uncursed),
                        )
                    })
                    .unwrap_or((0, BucStatus::Uncursed));
                if current_buc == BucStatus::Cursed {
                    world
                        .state_mut()
                        .entities
                        .set_item_buc(weapon_id, BucStatus::Uncursed);
                }
                let next_enchant = (current_enchant + 1).min(MAX_ITEM_ENCHANTMENT);
                world
                    .state_mut()
                    .entities
                    .set_item_enchantment(weapon_id, next_enchant);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your weapon glows blue for a moment.".to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your hands twitch for a moment.".to_string(),
                });
            }
        }
        ConsumableEffect::EnchantArmor => {
            if let Some(armor_id) = world.inventory.equipped_body {
                let (current_enchant, current_buc) = world
                    .entities
                    .get(armor_id)
                    .map(|entity| {
                        (
                            entity.item_enchantment().unwrap_or(0),
                            entity.item_buc().unwrap_or(BucStatus::Uncursed),
                        )
                    })
                    .unwrap_or((0, BucStatus::Uncursed));
                if current_buc == BucStatus::Cursed {
                    world
                        .state_mut()
                        .entities
                        .set_item_buc(armor_id, BucStatus::Uncursed);
                }
                let next_enchant = (current_enchant + 1).min(MAX_ITEM_ENCHANTMENT);
                world
                    .state_mut()
                    .entities
                    .set_item_enchantment(armor_id, next_enchant);
                if recompute_player_ac(world).is_err() {
                    world
                        .state_mut()
                        .entities
                        .set_item_enchantment(armor_id, current_enchant);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your armor glows silver for a moment.".to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your skin feels warm for a moment.".to_string(),
                });
            }
        }
        ConsumableEffect::RemoveCurse => {
            let cursed_items: Vec<EntityId> = world
                .inventory
                .entries
                .iter()
                .map(|entry| entry.item)
                .filter(|id| {
                    world.entities.get(*id).and_then(|entity| entity.item_buc())
                        == Some(BucStatus::Cursed)
                })
                .collect();
            for id in cursed_items {
                world
                    .state_mut()
                    .entities
                    .set_item_buc(id, BucStatus::Uncursed);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "You feel as if someone is helping you.".to_string(),
            });
        }
        ConsumableEffect::Heal { .. } | ConsumableEffect::CureStatus => {
            return Err("item is not a scroll".to_string());
        }
    }
    Ok(events)
}

pub(crate) fn remove_inventory_item(world: &mut GameWorld, item: EntityId) -> Result<(), String> {
    if world.inventory.equipped_melee == Some(item)
        && world
            .entities
            .get(item)
            .and_then(|entity| entity.item_buc())
            == Some(BucStatus::Cursed)
    {
        return Err("equipped weapon is cursed and cannot be removed".to_string());
    }
    if world.inventory.equipped_body == Some(item) {
        unequip_body(world, item)?;
    }
    if world.inventory.equipped_ring == Some(item) {
        unequip_ring(world, item)?;
    }
    world
        .state_mut()
        .inventory
        .remove(item)
        .ok_or_else(|| "item is not in player inventory".to_string())?;
    Ok(())
}

pub fn put_on(world: &mut GameWorld, item: EntityId) -> Result<Option<GameEvent>, String> {
    if world.inventory.equipped_ring == Some(item) {
        return Ok(None);
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.class != ItemClass::Ring {
        return Err("item is not a ring".to_string());
    }
    let _ = compute_ac_with(world, world.inventory.equipped_body, Some(item))?;
    if let Some(previous) = world.inventory.equipped_ring {
        unequip_ring(world, previous)?;
    }
    world.state_mut().inventory.equip_ring(item);
    recompute_player_ac(world)?;
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot: EquipmentSlot::Ring,
    }))
}

pub fn remove_ring(world: &mut GameWorld) -> Result<GameEvent, String> {
    let item = world
        .inventory
        .equipped_ring
        .ok_or_else(|| "no ring is equipped".to_string())?;
    unequip_ring(world, item)?;
    Ok(GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You take off your ring.".to_string(),
    })
}

pub(crate) fn unequip_ring(world: &mut GameWorld, item: EntityId) -> Result<(), String> {
    if world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        == Some(BucStatus::Cursed)
    {
        return Err("equipped ring is cursed and cannot be removed".to_string());
    }
    world.state_mut().inventory.equipped_ring = None;
    recompute_player_ac(world)
}

pub(crate) fn compute_ac_with(
    world: &GameWorld,
    equipped_body: Option<EntityId>,
    equipped_ring: Option<EntityId>,
) -> Result<i16, String> {
    let mut total_bonus: i16 = 0;
    for equipped in [equipped_body, equipped_ring].into_iter().flatten() {
        if let Some(data) = world.entities.item_data(equipped) {
            let enchant = world
                .entities
                .get(equipped)
                .and_then(|entity| entity.item_enchantment())
                .unwrap_or(0);
            total_bonus =
                total_bonus.saturating_add(data.ac_bonus.saturating_add(i16::from(enchant)));
        }
    }
    adventurer_template()
        .ac
        .checked_sub(total_bonus)
        .ok_or_else(|| "armor AC exceeds the supported range".to_string())
}

pub(crate) fn recompute_player_ac(world: &mut GameWorld) -> Result<(), String> {
    let derived_ac = compute_ac_with(
        world,
        world.inventory.equipped_body,
        world.inventory.equipped_ring,
    )?;
    let player_id = world.player_id;
    if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
        stats.ac = derived_ac;
    }
    Ok(())
}

pub(crate) fn has_poison_resistance(world: &GameWorld) -> bool {
    equipped_ring_effect(world) == Some(aihack_core::domain::item::RingEffect::PoisonResistance)
}

pub(crate) fn has_fire_resistance(world: &GameWorld) -> bool {
    equipped_ring_effect(world) == Some(aihack_core::domain::item::RingEffect::FireResistance)
}

fn equipped_ring_effect(world: &GameWorld) -> Option<aihack_core::domain::item::RingEffect> {
    let ring = world.inventory.equipped_ring?;
    world
        .entities
        .item_data(ring)
        .and_then(|data| data.ring_effect)
}

fn unequip_body(world: &mut GameWorld, item: EntityId) -> Result<(), String> {
    world
        .entities
        .item_data(item)
        .filter(|data| data.class == ItemClass::Armor)
        .ok_or_else(|| "equipped body item is not armor".to_string())?;
    if world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        == Some(BucStatus::Cursed)
    {
        return Err("equipped armor is cursed and cannot be removed".to_string());
    }
    if world.entities.actor_stats(world.player_id).is_none() {
        return Err("player actor stats are missing".to_string());
    }
    world.state_mut().inventory.equipped_body = None;
    recompute_player_ac(world)
}
