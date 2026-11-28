use aihack_core::{
    action::DipTarget,
    domain::{
        combat::DeathCause,
        entity::EntityLocation,
        item::{BucStatus, ItemClass, ItemKind},
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    appearance,
    systems::{altar, artifacts, combat::roll_die, death, items},
    world::GameWorld,
};

pub fn dip(
    seed: u64,
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    target: DipTarget,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("dipping is only available in campaign mode".to_string());
    }
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    match target {
        DipTarget::Tile => dip_into_tile(seed, world, rng, item),
        DipTarget::Item(potion_id) => dip_into_potion(seed, world, rng, item, potion_id),
    }
}

fn dip_into_tile(
    seed: u64,
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    let pos = world.player_pos();
    let tile = world
        .current_map()
        .tile(pos)
        .map_err(|_| "current tile is invalid".to_string())?;
    let entity = world
        .entities
        .get(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let (kind, data, _, _, _) = entity
        .item()
        .ok_or_else(|| "entity is not an item".to_string())?;
    let data = *data;
    let buc = entity.item_buc().unwrap_or(BucStatus::Uncursed);
    let qty = entity.item_quantity().unwrap_or(1);

    match tile {
        TileKind::Altar => {
            let cross_aligned = world.campaign.is_some_and(|c| {
                altar::altar_alignment(world).is_some_and(|align| align != c.alignment)
            });
            if cross_aligned {
                let enchant = entity.item_enchantment().unwrap_or(0);
                if !can_place_result_single(
                    world,
                    item,
                    qty,
                    kind,
                    BucStatus::Cursed,
                    enchant,
                    false,
                ) {
                    return Err("inventory letter capacity exceeded".to_string());
                }
                let target = if qty > 1 && kind.is_stackable() {
                    spawn_or_replace_single(
                        world,
                        item,
                        qty,
                        kind,
                        BucStatus::Cursed,
                        enchant,
                        true,
                    )?
                } else {
                    let entities = &mut world.state_mut().entities;
                    entities.set_item_buc(item, BucStatus::Cursed);
                    entities.set_item_buc_known(item, true);
                    item
                };
                let _ = target;
                let state = world.state_mut();
                state.luck = (state.luck - 1).clamp(
                    aihack_core::campaign::MIN_LUCK,
                    aihack_core::campaign::MAX_LUCK,
                );
                if let Some(mut c) = state.campaign {
                    c.god_anger =
                        (c.god_anger.saturating_add(1)).min(aihack_core::campaign::MAX_GOD_ANGER);
                    state.campaign = Some(c);
                }
                return Ok(vec![GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "An unholy chill curses the item you dipped on the alien altar!"
                        .to_string(),
                }]);
            }
            if kind != ItemKind::PotionWater || buc != BucStatus::Uncursed {
                if data.class == ItemClass::Potion
                    && (!world.is_item_identified(kind)
                        || !entity.item_buc_known().unwrap_or(false))
                {
                    return Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "Nothing happens.".to_string(),
                    }]);
                }
                return Err("only uncursed water can be blessed on an altar".to_string());
            }
            if world.mp < 3 {
                return Err("not enough MP (requires 3) to bless water".to_string());
            }
            if !can_place_result_single(
                world,
                item,
                qty,
                ItemKind::PotionWater,
                BucStatus::Blessed,
                0,
                false,
            ) {
                return Err("inventory letter capacity exceeded".to_string());
            }
            world.state_mut().mp -= 3;
            world.identify_item_kind(ItemKind::PotionWater);
            let holy = spawn_or_replace_single(
                world,
                item,
                qty,
                ItemKind::PotionWater,
                BucStatus::Blessed,
                0,
                true,
            )?;
            let _ = holy;
            Ok(vec![GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Your potion glows silver as it turns into holy water.".to_string(),
            }])
        }
        TileKind::Fountain | TileKind::Sink | TileKind::Water => {
            if tile == TileKind::Fountain {
                if let Some(excal_events) = artifacts::try_fountain_excalibur(world, item)? {
                    return Ok(excal_events);
                }
            }
            let mut events = apply_plain_water_effect(seed, world, item, kind, data.class, qty)?;
            if tile == TileKind::Fountain && rng.next_u64().is_multiple_of(5) {
                let _ = world.current_map_mut().set_tile(pos, TileKind::DryFountain);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The fountain dries up!".to_string(),
                });
            }
            Ok(events)
        }
        _ => Err("there is no fountain, sink, water, or altar here to dip into".to_string()),
    }
}

fn dip_into_potion(
    seed: u64,
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    potion_id: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if item == potion_id {
        return Err("cannot dip an item into itself".to_string());
    }
    if !world.inventory.contains(potion_id) {
        return Err("target potion is not in player inventory".to_string());
    }
    let target_entity = world
        .entities
        .get(potion_id)
        .ok_or_else(|| "target potion is missing".to_string())?;
    let (target_kind, target_data, _, _, _) = target_entity
        .item()
        .ok_or_else(|| "target is not an item".to_string())?;
    if target_data.class != ItemClass::Potion {
        return Err("can only dip into a potion".to_string());
    }
    let target_buc = target_entity.item_buc().unwrap_or(BucStatus::Uncursed);
    let target_qty = target_entity.item_quantity().unwrap_or(1);

    let item_entity = world
        .entities
        .get(item)
        .ok_or_else(|| "item is missing".to_string())?;
    let (item_kind, item_data, _, _, _) = item_entity
        .item()
        .ok_or_else(|| "entity is not an item".to_string())?;
    let item_class = item_data.class;
    let item_buc = item_entity.item_buc().unwrap_or(BucStatus::Uncursed);
    let item_qty = item_entity.item_quantity().unwrap_or(1);
    let item_enchant = item_entity.item_enchantment().unwrap_or(0);
    let freed_target_slot = target_qty <= 1;

    if target_kind == ItemKind::PotionWater {
        if item_kind == ItemKind::PotionWater && target_buc == BucStatus::Uncursed {
            if !world.is_item_identified(target_kind)
                || !target_entity.item_buc_known().unwrap_or(false)
                || !world.is_item_identified(item_kind)
            {
                return Ok(vec![GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Nothing happens.".to_string(),
                }]);
            }
            return Err("dipping plain water into plain water does nothing".to_string());
        }
        match target_buc {
            BucStatus::Blessed => {
                let next_buc = match item_buc {
                    BucStatus::Cursed => BucStatus::Uncursed,
                    BucStatus::Uncursed | BucStatus::Blessed => BucStatus::Blessed,
                };
                if item_qty > 1
                    && item_kind.is_stackable()
                    && !can_place_result_single(
                        world,
                        item,
                        item_qty,
                        item_kind,
                        next_buc,
                        item_enchant,
                        freed_target_slot,
                    )
                {
                    return Err("inventory letter capacity exceeded".to_string());
                }
                items::consume_one_from_inventory(world, potion_id)?;
                world.identify_item_kind(ItemKind::PotionWater);
                if world.inventory.contains(potion_id) {
                    world
                        .state_mut()
                        .entities
                        .set_item_buc_known(potion_id, true);
                }
                if item_qty > 1 && item_kind.is_stackable() {
                    let _ = spawn_or_replace_single(
                        world,
                        item,
                        item_qty,
                        item_kind,
                        next_buc,
                        item_enchant,
                        true,
                    )?;
                } else {
                    let entities = &mut world.state_mut().entities;
                    entities.set_item_buc(item, next_buc);
                    entities.set_item_buc_known(item, true);
                    items::recompute_player_ac(world)?;
                }
                return Ok(vec![GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A silver glow surrounds the dipped item.".to_string(),
                }]);
            }
            BucStatus::Cursed => {
                let next_buc = match item_buc {
                    BucStatus::Blessed => BucStatus::Uncursed,
                    BucStatus::Uncursed | BucStatus::Cursed => BucStatus::Cursed,
                };
                if item_qty > 1
                    && item_kind.is_stackable()
                    && !can_place_result_single(
                        world,
                        item,
                        item_qty,
                        item_kind,
                        next_buc,
                        item_enchant,
                        freed_target_slot,
                    )
                {
                    return Err("inventory letter capacity exceeded".to_string());
                }
                items::consume_one_from_inventory(world, potion_id)?;
                world.identify_item_kind(ItemKind::PotionWater);
                if world.inventory.contains(potion_id) {
                    world
                        .state_mut()
                        .entities
                        .set_item_buc_known(potion_id, true);
                }
                if item_qty > 1 && item_kind.is_stackable() {
                    let _ = spawn_or_replace_single(
                        world,
                        item,
                        item_qty,
                        item_kind,
                        next_buc,
                        item_enchant,
                        true,
                    )?;
                } else {
                    let entities = &mut world.state_mut().entities;
                    entities.set_item_buc(item, next_buc);
                    entities.set_item_buc_known(item, true);
                    items::recompute_player_ac(world)?;
                }
                return Ok(vec![GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "A black aura clings to the dipped item.".to_string(),
                }]);
            }
            BucStatus::Uncursed => {
                if !can_plain_water_place(
                    world,
                    item,
                    item_kind,
                    item_class,
                    item_qty,
                    freed_target_slot,
                ) {
                    return Err("inventory letter capacity exceeded".to_string());
                }
                items::consume_one_from_inventory(world, potion_id)?;
                return apply_plain_water_effect(
                    seed, world, item, item_kind, item_class, item_qty,
                );
            }
        }
    }

    // 연금술 혼합: 두 물약 조합
    if item_class == ItemClass::Potion && item_kind != ItemKind::PotionWater {
        let result_kind = match (item_kind, target_kind) {
            (ItemKind::PotionHealing, ItemKind::PotionHealing)
            | (ItemKind::PotionHealing, ItemKind::PotionCure)
            | (ItemKind::PotionCure, ItemKind::PotionHealing) => ItemKind::PotionExtraHealing,
            (ItemKind::PotionSpeed, ItemKind::PotionConfusion)
            | (ItemKind::PotionConfusion, ItemKind::PotionSpeed) => ItemKind::PotionHallucination,
            (ItemKind::PotionInvisibility, ItemKind::PotionLevitation)
            | (ItemKind::PotionLevitation, ItemKind::PotionInvisibility) => ItemKind::PotionSpeed,
            _ => ItemKind::PotionWater,
        };
        let result_buc = if item_buc == BucStatus::Blessed
            && target_buc == BucStatus::Blessed
            && result_kind != ItemKind::PotionWater
        {
            BucStatus::Blessed
        } else {
            BucStatus::Uncursed
        };
        if !can_place_result_single(
            world,
            item,
            item_qty,
            result_kind,
            result_buc,
            0,
            freed_target_slot,
        ) {
            return Err("inventory letter capacity exceeded".to_string());
        }
        items::consume_one_from_inventory(world, potion_id)?;
        let _ = spawn_or_replace_single(world, item, item_qty, result_kind, result_buc, 0, true)?;
        let mut events = Vec::new();
        if (item_buc == BucStatus::Cursed || target_buc == BucStatus::Cursed)
            && result_kind == ItemKind::PotionWater
        {
            let dmg = roll_die(rng, 4);
            let player_id = world.player_id;
            if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                stats.hp = stats.hp.saturating_sub(dmg);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: format!("The volatile mixture splashes you for {dmg} damage!"),
            });
            events.extend(death::collect_death_events_if_hp_depleted(
                world,
                player_id,
                DeathCause::Explosion,
            )?);
        } else {
            let name = appearance::display_name_for_kind(seed, world, result_kind);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("The potions bubble and mix into {name}!"),
            });
        }
        return Ok(events);
    }

    if !world.is_item_identified(target_kind) || !world.is_item_identified(item_kind) {
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "Nothing happens.".to_string(),
        }]);
    }
    Err("dipping into that potion has no special effect".to_string())
}

fn can_plain_water_place(
    world: &GameWorld,
    item: EntityId,
    kind: ItemKind,
    class: ItemClass,
    qty: u16,
    extra_freed_slot: bool,
) -> bool {
    if class == ItemClass::Potion && kind != ItemKind::PotionWater {
        return can_place_result_single(
            world,
            item,
            qty,
            ItemKind::PotionWater,
            BucStatus::Uncursed,
            0,
            extra_freed_slot,
        );
    }
    if class == ItemClass::Scroll && kind != ItemKind::ScrollBlank {
        return can_place_result_single(
            world,
            item,
            qty,
            ItemKind::ScrollBlank,
            BucStatus::Uncursed,
            0,
            extra_freed_slot,
        );
    }
    true
}

fn can_place_result_single(
    world: &GameWorld,
    source_item: EntityId,
    source_qty: u16,
    new_kind: ItemKind,
    new_buc: BucStatus,
    new_enchant: i8,
    extra_freed_slot: bool,
) -> bool {
    if source_qty <= 1 {
        return true;
    }
    if world.inventory.has_free_slot() || extra_freed_slot {
        return true;
    }
    new_kind.is_stackable()
        && world.inventory.entries.iter().any(|entry| {
            let Some(e) = world.entities.get(entry.item) else {
                return false;
            };
            let Some((k, _, _, _, _)) = e.item() else {
                return false;
            };
            let q = e.item_quantity().unwrap_or(1);
            entry.item != source_item
                && k == new_kind
                && e.item_unpaid() != Some(true)
                && e.item_buc().unwrap_or(BucStatus::Uncursed) == new_buc
                && e.item_enchantment().unwrap_or(0) == new_enchant
                && e.item_erosion().unwrap_or(0) == 0
                && q < aihack_core::domain::item::MAX_ITEM_STACK
        })
}

fn apply_plain_water_effect(
    seed: u64,
    world: &mut GameWorld,
    item: EntityId,
    kind: ItemKind,
    class: ItemClass,
    qty: u16,
) -> Result<Vec<GameEvent>, String> {
    if class == ItemClass::Potion && kind != ItemKind::PotionWater {
        let _ = spawn_or_replace_single(
            world,
            item,
            qty,
            ItemKind::PotionWater,
            BucStatus::Uncursed,
            0,
            true,
        )?;
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "The potion dilutes into plain water.".to_string(),
        }]);
    }
    if class == ItemClass::Scroll && kind != ItemKind::ScrollBlank {
        let _ = spawn_or_replace_single(
            world,
            item,
            qty,
            ItemKind::ScrollBlank,
            BucStatus::Uncursed,
            0,
            true,
        )?;
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "The writing washes off the scroll.".to_string(),
        }]);
    }
    if kind == ItemKind::OilLamp {
        world.state_mut().entities.set_item_tool_active(item, false);
        world.mark_current_level_explored();
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "Your lamp sputters out in the water.".to_string(),
        }]);
    }
    let material = world
        .content_registry()
        .item(aihack_content::item_id(kind))
        .and_then(|d| d.material.as_deref());
    let is_artifact = world
        .entities
        .get(item)
        .and_then(|e| e.item_artifact_id())
        .is_some();
    if !is_artifact && matches!(material, Some("iron" | "steel")) {
        let erosion = world
            .entities
            .get(item)
            .and_then(|e| e.item_erosion())
            .unwrap_or(0);
        let next = (erosion + 1).min(3);
        world.state_mut().entities.set_item_erosion(item, next);
        items::recompute_player_ac(world)?;
        let item_name = appearance::display_name_for_world_item(seed, world, item, kind);
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Warning,
            text: format!("Your {item_name} looks rusty!"),
        }]);
    }
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "The dipped item gets wet.".to_string(),
    }])
}

fn spawn_or_replace_single(
    world: &mut GameWorld,
    source_item: EntityId,
    source_qty: u16,
    new_kind: ItemKind,
    new_buc: BucStatus,
    new_enchant: i8,
    buc_known: bool,
) -> Result<EntityId, String> {
    let new_data =
        crate::domain::item::try_item_data_from_registry(new_kind, world.content_registry())
            .map_err(|e| e.to_string())?;
    let player_id = world.player_id;

    // 이미 인벤토리에 합칠 동일 스택이 있는 경우 우선 소비 후 병합
    if new_kind.is_stackable() {
        let merge_target = world.inventory.entries.iter().find_map(|entry| {
            let e = world.entities.get(entry.item)?;
            let (k, _, _, _, _) = e.item()?;
            let q = e.item_quantity().unwrap_or(1);
            if entry.item != source_item
                && k == new_kind
                && e.item_unpaid() != Some(true)
                && e.item_buc().unwrap_or(BucStatus::Uncursed) == new_buc
                && e.item_enchantment().unwrap_or(0) == new_enchant
                && e.item_erosion().unwrap_or(0) == 0
                && q < aihack_core::domain::item::MAX_ITEM_STACK
            {
                Some((entry.item, q + 1))
            } else {
                None
            }
        });
        if let Some((target_id, next_qty)) = merge_target {
            items::consume_one_from_inventory(world, source_item)?;
            let entities = &mut world.state_mut().entities;
            entities.set_item_quantity(target_id, next_qty);
            if buc_known {
                entities.set_item_buc_known(target_id, true);
            }
            return Ok(target_id);
        }
    }

    if source_qty > 1 {
        if !world.inventory.has_free_slot() {
            return Err("inventory letter capacity exceeded".to_string());
        }
        items::consume_one_from_inventory(world, source_item)?;
        let spawned = world
            .state_mut()
            .entities
            .spawn_item_with_data(
                new_kind,
                new_data,
                EntityLocation::Inventory { owner: player_id },
            )
            .map_err(|e| e.to_string())?;
        let letter = world
            .state_mut()
            .inventory
            .add_existing_with_next_letter(spawned)
            .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
        let entities = &mut world.state_mut().entities;
        entities.set_item_letter(spawned, letter);
        entities.set_item_buc(spawned, new_buc);
        entities.set_item_enchantment(spawned, new_enchant);
        if buc_known {
            entities.set_item_buc_known(spawned, true);
        }
        Ok(spawned)
    } else {
        let entities = &mut world.state_mut().entities;
        entities.set_item_kind_and_data(source_item, new_kind, new_data);
        entities.set_item_buc(source_item, new_buc);
        entities.set_item_enchantment(source_item, new_enchant);
        entities.set_item_erosion(source_item, 0);
        if buc_known {
            entities.set_item_buc_known(source_item, true);
        }
        items::recompute_player_ac(world)?;
        Ok(source_item)
    }
}
