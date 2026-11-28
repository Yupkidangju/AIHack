use aihack_core::{
    domain::{
        entity::EntityLocation,
        item::{
            AmuletEffect, BucStatus, ConsumableEffect, EquipmentSlot, ItemClass, StatusEffectKind,
            MAX_ITEM_ENCHANTMENT, MAX_ITEM_STACK, MIN_ITEM_ENCHANTMENT,
        },
        level::{
            PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID,
            PHASE5_LEVEL2_STAIRS_UP_POS,
        },
        player::adventurer_template,
        status::Resistances,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    systems::{ascension, combat::roll_die, traps},
    world::GameWorld,
};

pub fn pickup(world: &mut GameWorld) -> Result<GameEvent, String> {
    let pos = world.player_pos();
    let item = world
        .entities
        .item_at(world.current_level(), pos)
        .ok_or_else(|| "no item at player position".to_string())?;
    let player_id = world.player_id;
    if world.campaign.is_some() {
        let Some(floor_entity) = world.entities.get(item) else {
            return Err("entity is not an item".to_string());
        };
        let is_unpaid = floor_entity.item_unpaid() == Some(true);
        if !is_unpaid {
            if let Some((kind, _, _, _, _)) = floor_entity.item() {
                if kind.is_stackable() {
                    let floor_buc = floor_entity.item_buc().unwrap_or(BucStatus::Uncursed);
                    let floor_enchant = floor_entity.item_enchantment().unwrap_or(0);
                    let floor_qty = floor_entity.item_quantity().unwrap_or(1);
                    let target = world.inventory.entries.iter().find_map(|entry| {
                        let existing = world.entities.get(entry.item)?;
                        let (existing_kind, _, _, _, _) = existing.item()?;
                        let existing_qty = existing.item_quantity().unwrap_or(1);
                        if existing_kind == kind
                            && existing.item_unpaid() != Some(true)
                            && existing.item_buc().unwrap_or(BucStatus::Uncursed) == floor_buc
                            && existing.item_enchantment().unwrap_or(0) == floor_enchant
                            && existing_qty.saturating_add(floor_qty) <= MAX_ITEM_STACK
                        {
                            Some((entry.item, entry.letter, existing_qty + floor_qty))
                        } else {
                            None
                        }
                    });
                    if let Some((existing_id, existing_letter, next_qty)) = target {
                        let entities = &mut world.state_mut().entities;
                        entities.set_item_quantity(existing_id, next_qty);
                        entities.set_item_location(item, EntityLocation::Consumed);
                        entities.set_item_quantity(item, 1);
                        return Ok(GameEvent::ItemPickedUp {
                            entity: player_id,
                            item: existing_id,
                            letter: existing_letter,
                        });
                    }
                }
            }
        }
    }
    let letter = world
        .state_mut()
        .inventory
        .add_existing_with_next_letter(item)
        .ok_or_else(|| "inventory letter capacity exceeded".to_string())?;
    world
        .state_mut()
        .entities
        .set_item_location(item, EntityLocation::Inventory { owner: player_id });
    world.state_mut().entities.set_item_letter(item, letter);
    if world.campaign.is_some()
        && world.entities.get(item).and_then(|e| e.item_unpaid()) == Some(true)
        && crate::systems::npc::has_peaceful_shopkeeper_on_level(world)
    {
        let price = crate::systems::npc::unpaid_item_price(world, item);
        let qty = u32::from(
            world
                .entities
                .get(item)
                .and_then(|e| e.item_quantity())
                .unwrap_or(1),
        );
        let charge = price.saturating_mul(qty);
        let next = world
            .shop_debt
            .saturating_add(charge)
            .min(crate::systems::npc::MAX_SHOP_DEBT);
        world.state_mut().shop_debt = next;
    }

    Ok(GameEvent::ItemPickedUp {
        entity: world.player_id,
        item,
        letter,
    })
}

pub(crate) fn consume_one_from_inventory(
    world: &mut GameWorld,
    item: EntityId,
) -> Result<bool, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let qty = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_quantity())
        .unwrap_or(1);
    if qty > 1 {
        world.state_mut().entities.set_item_quantity(item, qty - 1);
        Ok(false)
    } else {
        remove_inventory_item(world, item)?;
        world
            .state_mut()
            .entities
            .set_item_location(item, EntityLocation::Consumed);
        Ok(true)
    }
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
    if world.entities.get(item).and_then(|e| e.item_buc()) == Some(BucStatus::Cursed) {
        world.state_mut().entities.set_item_buc_known(item, true);
    }
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot: EquipmentSlot::Melee,
    }))
}

pub fn wear(world: &mut GameWorld, item: EntityId) -> Result<Option<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let slot = data
        .effective_armor_slot()
        .ok_or_else(|| "item is not armor".to_string())?;
    if world.inventory.equipped_in(slot) == Some(item) {
        return Ok(None);
    }
    let mut next_inv = world.inventory.clone();
    next_inv.set_slot(slot, Some(item));
    let _ = compute_ac_for_inventory(world, &next_inv)?;
    if let Some(previous) = world.inventory.equipped_in(slot) {
        unequip_slot(world, slot, previous)?;
    }
    world.state_mut().inventory.set_slot(slot, Some(item));
    if world.entities.get(item).and_then(|e| e.item_buc()) == Some(BucStatus::Cursed) {
        world.state_mut().entities.set_item_buc_known(item, true);
    }
    world
        .state_mut()
        .entities
        .set_item_enchant_known(item, true);
    recompute_player_ac(world)?;
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot,
    }))
}

pub fn drop(world: &mut GameWorld, item: EntityId) -> Result<GameEvent, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    if let Some(entity) = world.entities.get(item) {
        if entity
            .item()
            .is_some_and(|(k, _, _, _, _)| k == aihack_core::domain::item::ItemKind::Blindfold)
            && entity.item_tool_active() == Some(true)
        {
            if entity.item_buc() == Some(BucStatus::Cursed) {
                return Err("The blindfold is cursed and stuck to your face!".to_string());
            }
            world.state_mut().entities.set_item_tool_active(item, false);
            world.mark_current_level_explored();
        }
    }
    remove_inventory_item(world, item)?;
    let level = world.current_level();
    let pos = world.player_pos();
    let was_unpaid = world.entities.get(item).and_then(|e| e.item_unpaid()) == Some(true);
    if world.campaign.is_some()
        && was_unpaid
        && crate::systems::npc::has_peaceful_shopkeeper_on_level(world)
        && crate::systems::npc::is_in_shop_zone(world, pos)
    {
        let price = crate::systems::npc::unpaid_item_price(world, item);
        let qty = u32::from(
            world
                .entities
                .get(item)
                .and_then(|e| e.item_quantity())
                .unwrap_or(1),
        );
        let credit = price.saturating_mul(qty);
        world.state_mut().shop_debt = world.shop_debt.saturating_sub(credit);
    }
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
            state.status_timers.blind = 0;
            state.status_timers.confused = 0;
            state.status_timers.stunned = 0;
            state.status_timers.hallucinating = 0;
            match buc {
                BucStatus::Cursed => 1,
                BucStatus::Uncursed => i32::from(roll_die(rng, 6)) + 4,
                BucStatus::Blessed => i32::from(roll_die(rng, 6)) + 8,
            }
        }
        Some(ConsumableEffect::ApplyStatus { effect, turns }) => {
            let dur = match (effect, buc) {
                (
                    StatusEffectKind::Fast
                    | StatusEffectKind::Invisible
                    | StatusEffectKind::Levitation,
                    BucStatus::Blessed,
                ) => turns.saturating_add(10),
                (
                    StatusEffectKind::Fast
                    | StatusEffectKind::Invisible
                    | StatusEffectKind::Levitation,
                    BucStatus::Cursed,
                ) => (turns / 2).max(1),
                (
                    StatusEffectKind::Blind
                    | StatusEffectKind::Confused
                    | StatusEffectKind::Hallucinating,
                    BucStatus::Blessed,
                ) => (turns / 2).max(1),
                (
                    StatusEffectKind::Blind
                    | StatusEffectKind::Confused
                    | StatusEffectKind::Hallucinating,
                    BucStatus::Cursed,
                ) => turns.saturating_add(8),
                _ => turns,
            };
            let text = {
                let state = world.state_mut();
                match effect {
                    StatusEffectKind::Blind => {
                        state.status_timers.blind = state.status_timers.blind.max(dur);
                        "A cloud of darkness falls upon you."
                    }
                    StatusEffectKind::Confused => {
                        state.status_timers.confused = state.status_timers.confused.max(dur);
                        "Wait, what? The world spins around you!"
                    }
                    StatusEffectKind::Stunned => {
                        state.status_timers.stunned = state.status_timers.stunned.max(dur);
                        "You reel from the shock!"
                    }
                    StatusEffectKind::Hallucinating => {
                        state.status_timers.hallucinating =
                            state.status_timers.hallucinating.max(dur);
                        state.hallucinating = true;
                        "Oh wow! Everything looks so cosmic!"
                    }
                    StatusEffectKind::Fast => {
                        state.status_timers.fast = state.status_timers.fast.max(dur);
                        "You feel yourself moving much faster."
                    }
                    StatusEffectKind::Levitation => {
                        state.status_timers.levitation = state.status_timers.levitation.max(dur);
                        "You begin floating above the floor!"
                    }
                    StatusEffectKind::Invisible => {
                        state.status_timers.invisible = state.status_timers.invisible.max(dur);
                        "You vanish from sight!"
                    }
                }
            };
            consume_one_from_inventory(world, item)?;
            if world.campaign.is_some() {
                world.identify_item_kind(data.kind);
            }
            return Ok(vec![
                GameEvent::ItemConsumed {
                    entity: world.player_id,
                    item,
                },
                GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: text.to_string(),
                },
            ]);
        }
        Some(ConsumableEffect::Water) => {
            consume_one_from_inventory(world, item)?;
            if world.campaign.is_some() {
                world.identify_item_kind(data.kind);
            }
            let player_id = world.player_id;
            match buc {
                BucStatus::Blessed => {
                    let heal = i32::from(roll_die(rng, 4));
                    let state = world.state_mut();
                    state.status_timers.blind = 0;
                    state.status_timers.confused = 0;
                    state.status_timers.stunned = 0;
                    state.status_timers.hallucinating = 0;
                    state.hallucinating = false;
                    let mut events = vec![GameEvent::ItemConsumed {
                        entity: player_id,
                        item,
                    }];
                    if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                        let before = stats.hp;
                        stats.hp = (i32::from(stats.hp) + heal).min(i32::from(stats.max_hp)) as i16;
                        events.push(GameEvent::EntityHealed {
                            entity: player_id,
                            amount: stats.hp - before,
                            hp_after: stats.hp,
                        });
                    }
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "This tastes like holy water! You feel blessed.".to_string(),
                    });
                    return Ok(events);
                }
                BucStatus::Uncursed => {
                    return Ok(vec![
                        GameEvent::ItemConsumed {
                            entity: player_id,
                            item,
                        },
                        GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: "This tastes like cool, plain water.".to_string(),
                        },
                    ]);
                }
                BucStatus::Cursed => {
                    let dmg = roll_die(rng, 4);
                    let state = world.state_mut();
                    state.status_timers.confused = state.status_timers.confused.max(3);
                    if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                        stats.hp = stats.hp.saturating_sub(dmg);
                    }
                    let mut events = vec![
                        GameEvent::ItemConsumed {
                            entity: player_id,
                            item,
                        },
                        GameEvent::Message {
                            priority: MessagePriority::Warning,
                            text: "Ugh! That foul unholy water burns your throat!".to_string(),
                        },
                    ];
                    events.extend(crate::systems::death::collect_death_events_if_hp_depleted(
                        world,
                        player_id,
                        aihack_core::domain::combat::DeathCause::Poison,
                    )?);
                    return Ok(events);
                }
            }
        }
        Some(ConsumableEffect::GainLevel) => {
            consume_one_from_inventory(world, item)?;
            if world.campaign.is_some() {
                world.identify_item_kind(data.kind);
            }
            let player_id = world.player_id;
            let mut msg = "You feel more experienced!";
            let (before_hp, hp_after) = if let Some(mut campaign) = world.campaign {
                let before_level = campaign.level();
                let before_max_hp = campaign.max_hp();
                if campaign.xp < aihack_core::campaign::MAX_XP {
                    campaign.xp = campaign
                        .xp
                        .saturating_add(20)
                        .min(aihack_core::campaign::MAX_XP);
                    let new_max_hp = campaign.max_hp();
                    let new_max_mp = campaign.max_mp();
                    let mp_delta =
                        campaign.role.mp_per_level() * i16::from(campaign.level() - before_level);
                    let hp_gain = (new_max_hp - before_max_hp).max(4);
                    let state = world.state_mut();
                    state.campaign = Some(campaign);
                    state.max_mp = new_max_mp;
                    state.mp = state.mp.saturating_add(mp_delta).min(state.max_mp);
                    let stats = state
                        .entities
                        .actor_stats_mut(player_id)
                        .ok_or_else(|| "player actor stats are missing".to_string())?;
                    let old_hp = stats.hp;
                    stats.max_hp = new_max_hp;
                    stats.hp = stats.hp.saturating_add(hp_gain).min(stats.max_hp);
                    stats.hit_bonus = campaign.hit_bonus();
                    stats.damage_bonus = campaign.damage_bonus();
                    (old_hp, stats.hp)
                } else {
                    msg = "You feel a surge of transcendent vitality!";
                    let state = world.state_mut();
                    let cap_mp = campaign.max_mp().saturating_add(40);
                    state.max_mp = state.max_mp.saturating_add(2).min(cap_mp);
                    state.mp = state.mp.saturating_add(2).min(state.max_mp);
                    let cap_hp = campaign.max_hp().saturating_add(60);
                    let stats = state
                        .entities
                        .actor_stats_mut(player_id)
                        .ok_or_else(|| "player actor stats are missing".to_string())?;
                    let old_hp = stats.hp;
                    stats.max_hp = stats.max_hp.saturating_add(3).min(cap_hp);
                    stats.hp = stats.hp.saturating_add(3).min(stats.max_hp);
                    (old_hp, stats.hp)
                }
            } else {
                let stats = world
                    .state_mut()
                    .entities
                    .actor_stats_mut(player_id)
                    .ok_or_else(|| "player actor stats are missing".to_string())?;
                let old_hp = stats.hp;
                stats.max_hp = stats.max_hp.saturating_add(3);
                stats.hp = stats.hp.saturating_add(3).min(stats.max_hp);
                (old_hp, stats.hp)
            };
            return Ok(vec![
                GameEvent::ItemConsumed {
                    entity: player_id,
                    item,
                },
                GameEvent::EntityHealed {
                    entity: player_id,
                    amount: hp_after - before_hp,
                    hp_after,
                },
                GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: msg.to_string(),
                },
            ]);
        }
        Some(ConsumableEffect::PolymorphSelf) => {
            consume_one_from_inventory(world, item)?;
            if world.campaign.is_some() {
                world.identify_item_kind(data.kind);
            }
            let player_id = world.player_id;
            if world.has_amulet_effect(AmuletEffect::Unchanging) {
                return Ok(vec![
                    GameEvent::ItemConsumed {
                        entity: player_id,
                        item,
                    },
                    GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "Your amulet keeps your form steadfast and unchanged!".to_string(),
                    },
                ]);
            }
            let heal = roll_die(rng, 6);
            let (amount, hp_after) = {
                let stats = world
                    .state_mut()
                    .entities
                    .actor_stats_mut(player_id)
                    .ok_or_else(|| "player actor stats are missing".to_string())?;
                let before = stats.hp;
                stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
                (stats.hp - before, stats.hp)
            };
            return Ok(vec![
                GameEvent::ItemConsumed {
                    entity: player_id,
                    item,
                },
                GameEvent::EntityHealed {
                    entity: player_id,
                    amount,
                    hp_after,
                },
                GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "You undergo a brief physical transformation!".to_string(),
                },
            ]);
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

    consume_one_from_inventory(world, item)?;
    if world.campaign.is_some() {
        world.identify_item_kind(data.kind);
    }

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
pub fn eat(
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
    if !matches!(data.class, ItemClass::Food | ItemClass::Corpse) {
        return Err("item is not edible".to_string());
    }
    let corpse_monster = world
        .entities
        .get(item)
        .and_then(|e| e.item_corpse_monster());
    let corpse_age = world
        .entities
        .get(item)
        .and_then(|e| e.item_corpse_age())
        .unwrap_or(0);
    let corpse_spec =
        corpse_monster.map(|mk| aihack_content::monster_corpse_spec(mk, world.content_registry()));
    let nutrition = corpse_spec
        .as_ref()
        .map(|spec| spec.nutrition)
        .filter(|nutrition| *nutrition > 0)
        .or_else(|| data.nutrition.filter(|nutrition| *nutrition > 0))
        .ok_or_else(|| "edible item has no positive nutrition".to_string())?;

    world.state_mut().nutrition = world.nutrition.saturating_add(nutrition);
    consume_one_from_inventory(world, item)?;

    let mut events = vec![GameEvent::ItemConsumed {
        entity: world.player_id,
        item,
    }];

    if data.class == ItemClass::Corpse && world.campaign.is_some() {
        if corpse_age > 50 {
            if has_poison_resistance(world) {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You choke down the stale corpse, but resist the poison.".to_string(),
                });
            } else {
                let next_poison = world.poison_turns.max(4);
                world.state_mut().poison_turns = next_poison;
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "Ulch! That corpse was tainted!".to_string(),
                });
            }
        } else if let Some(spec) = corpse_spec {
            for effect in &spec.effects {
                match effect.as_str() {
                    "poisonous" => {
                        if has_poison_resistance(world) {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You resist the poison in the corpse.".to_string(),
                            });
                        } else {
                            let next_poison = world.poison_turns.max(3);
                            world.state_mut().poison_turns = next_poison;
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: "That corpse was poisonous!".to_string(),
                            });
                        }
                    }
                    "grant_fire_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::FIRE);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel warm inside!".to_string(),
                            });
                        }
                    }
                    "grant_poison_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::POISON);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel healthy!".to_string(),
                            });
                        }
                    }
                    "grant_telepathy" => {
                        if rng.next_u64() % 100 < 75 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::TELEPATHY);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel a strange mental clarity!".to_string(),
                            });
                        }
                    }
                    "grant_cold_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::COLD);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel cool resistance!".to_string(),
                            });
                        }
                    }
                    "grant_shock_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::SHOCK);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel insulated!".to_string(),
                            });
                        }
                    }
                    "grant_sleep_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::SLEEP);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel wide awake!".to_string(),
                            });
                        }
                    }
                    "grant_magic_resistance" => {
                        if rng.next_u64() % 100 < 50 {
                            world
                                .state_mut()
                                .intrinsic_resistances
                                .insert(Resistances::MAGIC_RESISTANCE);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You feel protected from magic!".to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(events)
}

pub fn read_at(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    turn: u64,
) -> Result<Vec<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.class == ItemClass::Spellbook {
        return crate::systems::spells::study_spellbook(world, rng, item, turn);
    }
    if data.class != ItemClass::Scroll {
        return Err("item is not a scroll".to_string());
    }
    let Some(effect) = data.consumable_effect else {
        return Err("item is not a scroll".to_string());
    };
    let scroll_buc = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        .unwrap_or(BucStatus::Uncursed);

    consume_one_from_inventory(world, item)?;
    if world.campaign.is_some() {
        world.identify_item_kind(data.kind);
    }

    let mut events = vec![GameEvent::ScrollRead {
        entity: world.player_id,
        item,
    }];
    match effect {
        ConsumableEffect::RevealLevel => {
            events.extend(traps::reveal_all_hidden_tiles(world));
        }
        ConsumableEffect::IdentifySingle => {
            let unidentified_kind = if world.campaign.is_some() {
                world
                    .inventory
                    .entries
                    .iter()
                    .map(|entry| entry.item)
                    .find(|candidate| {
                        *candidate != item
                            && world
                                .entities
                                .get(*candidate)
                                .and_then(|entity| entity.item())
                                .is_some_and(|(kind, _, _, _, _)| !world.is_item_identified(kind))
                    })
            } else {
                None
            };
            let unappraised = if world.campaign.is_some() {
                world
                    .inventory
                    .entries
                    .iter()
                    .map(|entry| entry.item)
                    .find(|candidate| {
                        *candidate != item
                            && world.entities.get(*candidate).is_some_and(|entity| {
                                !entity.item_buc_known().unwrap_or(false)
                                    || !entity.item_enchant_known().unwrap_or(false)
                            })
                    })
            } else {
                None
            };
            if let Some(target) = unidentified_kind.or(unappraised).or_else(|| {
                world
                    .inventory
                    .entries
                    .iter()
                    .map(|entry| entry.item)
                    .find(|candidate| *candidate != item)
            }) {
                events.extend(identify_inventory_target(world, target));
            }
        }
        ConsumableEffect::LevelTeleport => {
            if world.campaign.is_some() {
                if crate::campaign::has_amulet(world) {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "A mysterious force prevents you from teleporting with the Amulet!"
                            .to_string(),
                    });
                } else {
                    let from = world.current_level();
                    let mut candidates: Vec<_> = world
                        .levels
                        .levels
                        .iter()
                        .map(|level| level.id)
                        .filter(|id| id.branch == from.branch && *id != from)
                        .collect();
                    candidates.sort_by_key(|id| id.depth);
                    if !candidates.is_empty() {
                        let controlled = world.has_teleport_control();
                        let to_level = if controlled {
                            if crate::campaign::has_amulet(world) {
                                candidates
                                    .iter()
                                    .copied()
                                    .rev()
                                    .find(|id| id.depth < from.depth)
                                    .unwrap_or(candidates[0])
                            } else {
                                candidates
                                    .iter()
                                    .copied()
                                    .find(|id| id.depth > from.depth)
                                    .unwrap_or(candidates[candidates.len() - 1])
                            }
                        } else {
                            let pick = (rng.next_u64() as usize) % candidates.len();
                            candidates[pick]
                        };
                        let to_pos = world
                            .levels
                            .stairs_up_pos(to_level)
                            .or_else(|| world.levels.stairs_down_pos(to_level))
                            .ok_or("teleport landing is missing")?;
                        world.set_player_location(to_level, to_pos);
                        if controlled {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You control your level teleport!".to_string(),
                            });
                        }
                        events.push(GameEvent::LevelChanged {
                            entity: world.player_id,
                            from,
                            to: to_level,
                        });
                        events.extend(ascension::on_level_entered(world, to_level)?);
                    }
                }
            } else {
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
                        let entities = &mut world.state_mut().entities;
                        entities.set_item_charges(wand_id, Some(0));
                        entities.set_item_buc_known(wand_id, true);
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: "Your wand vibrates weakly and loses its charges.".to_string(),
                        });
                    }
                    BucStatus::Blessed => {
                        let was_cursed = world.entities.get(wand_id).and_then(|e| e.item_buc())
                            == Some(BucStatus::Cursed);
                        let entities = &mut world.state_mut().entities;
                        entities.set_item_charges(wand_id, Some(max_charges));
                        if was_cursed {
                            entities.set_item_buc(wand_id, BucStatus::Uncursed);
                        }
                        entities.set_item_buc_known(wand_id, true);
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!("Recharged wand to {max_charges} charges"),
                        });
                    }
                    BucStatus::Uncursed => {
                        let entities = &mut world.state_mut().entities;
                        entities.set_item_charges(wand_id, Some(max_charges));
                        entities.set_item_buc_known(wand_id, true);
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
                let (next_enchant, next_buc, text) = match scroll_buc {
                    BucStatus::Cursed => (
                        (current_enchant - 1).max(MIN_ITEM_ENCHANTMENT),
                        BucStatus::Cursed,
                        "Your weapon glows black for a moment.",
                    ),
                    BucStatus::Uncursed => (
                        (current_enchant + 1).min(MAX_ITEM_ENCHANTMENT),
                        if current_buc == BucStatus::Cursed {
                            BucStatus::Uncursed
                        } else {
                            current_buc
                        },
                        "Your weapon glows blue for a moment.",
                    ),
                    BucStatus::Blessed => {
                        let bonus = roll_die(rng, 2) as i8;
                        (
                            (current_enchant + bonus).min(MAX_ITEM_ENCHANTMENT),
                            if current_buc == BucStatus::Cursed {
                                BucStatus::Uncursed
                            } else {
                                current_buc
                            },
                            "Your weapon glows brightly for a moment.",
                        )
                    }
                };
                let entities = &mut world.state_mut().entities;
                entities.set_item_buc(weapon_id, next_buc);
                entities.set_item_enchantment(weapon_id, next_enchant);
                entities.set_item_buc_known(weapon_id, true);
                entities.set_item_enchant_known(weapon_id, true);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: text.to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your hands twitch for a moment.".to_string(),
                });
            }
        }
        ConsumableEffect::EnchantArmor => {
            let target_armor = [
                EquipmentSlot::Body,
                EquipmentSlot::Shield,
                EquipmentSlot::Helm,
                EquipmentSlot::Cloak,
                EquipmentSlot::Gloves,
                EquipmentSlot::Boots,
            ]
            .into_iter()
            .filter_map(|slot| world.inventory.equipped_in(slot))
            .find(|id| {
                world
                    .entities
                    .item_data(*id)
                    .is_some_and(|d| d.class == ItemClass::Armor)
            });
            if let Some(armor_id) = target_armor {
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
                let (next_enchant, next_buc, text) = match scroll_buc {
                    BucStatus::Cursed => (
                        (current_enchant - 1).max(MIN_ITEM_ENCHANTMENT),
                        BucStatus::Cursed,
                        "Your armor glows black for a moment.",
                    ),
                    BucStatus::Uncursed => (
                        (current_enchant + 1).min(MAX_ITEM_ENCHANTMENT),
                        if current_buc == BucStatus::Cursed {
                            BucStatus::Uncursed
                        } else {
                            current_buc
                        },
                        "Your armor glows silver for a moment.",
                    ),
                    BucStatus::Blessed => {
                        let bonus = roll_die(rng, 2) as i8;
                        (
                            (current_enchant + bonus).min(MAX_ITEM_ENCHANTMENT),
                            if current_buc == BucStatus::Cursed {
                                BucStatus::Uncursed
                            } else {
                                current_buc
                            },
                            "Your armor shines brightly for a moment.",
                        )
                    }
                };
                {
                    let entities = &mut world.state_mut().entities;
                    entities.set_item_buc(armor_id, next_buc);
                    entities.set_item_enchantment(armor_id, next_enchant);
                    entities.set_item_buc_known(armor_id, true);
                    entities.set_item_enchant_known(armor_id, true);
                }
                if recompute_player_ac(world).is_err() {
                    world
                        .state_mut()
                        .entities
                        .set_item_enchantment(armor_id, current_enchant);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: text.to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your skin feels warm for a moment.".to_string(),
                });
            }
        }
        ConsumableEffect::RemoveCurse => match scroll_buc {
            BucStatus::Blessed => {
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
                    let entities = &mut world.state_mut().entities;
                    entities.set_item_buc(id, BucStatus::Uncursed);
                    entities.set_item_buc_known(id, true);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You feel as if someone is helping you.".to_string(),
                });
            }
            BucStatus::Uncursed => {
                let _ = uncurse_equipped_items(world);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You feel as if someone is helping you.".to_string(),
                });
            }
            BucStatus::Cursed => {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You feel like someone is ignoring you.".to_string(),
                });
            }
        },
        ConsumableEffect::MagicMapping => {
            events.extend(crate::systems::spells::reveal_entire_level_map(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "A map crystallizes in your mind!".to_string(),
            });
        }
        ConsumableEffect::BlankPaper => {
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "This scroll appears to be blank.".to_string(),
            });
        }
        ConsumableEffect::Heal { .. }
        | ConsumableEffect::CureStatus
        | ConsumableEffect::ApplyStatus { .. }
        | ConsumableEffect::GainLevel
        | ConsumableEffect::PolymorphSelf
        | ConsumableEffect::Water => {
            return Err("item is not a scroll".to_string());
        }
    }
    Ok(events)
}

pub fn identify(
    world: &mut GameWorld,
    scroll: EntityId,
    target: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if scroll == target {
        return Err("scroll cannot identify itself".to_string());
    }
    if !world.inventory.contains(scroll) {
        return Err("scroll is not in player inventory".to_string());
    }
    if !world.inventory.contains(target) {
        return Err("target is not in player inventory".to_string());
    }
    let scroll_data = *world
        .entities
        .item_data(scroll)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if scroll_data.consumable_effect != Some(ConsumableEffect::IdentifySingle) {
        return Err("item is not an identify scroll".to_string());
    }

    consume_one_from_inventory(world, scroll)?;
    if world.campaign.is_some() {
        world.identify_item_kind(scroll_data.kind);
    }

    let mut events = vec![GameEvent::ScrollRead {
        entity: world.player_id,
        item: scroll,
    }];
    events.extend(identify_inventory_target(world, target));
    Ok(events)
}

pub(crate) fn uncurse_equipped_items(world: &mut GameWorld) -> usize {
    let equipped_cursed: Vec<EntityId> = world
        .inventory
        .equipped_slots()
        .into_iter()
        .filter_map(|(_, item)| item)
        .filter(|id| {
            world.entities.get(*id).and_then(|entity| entity.item_buc()) == Some(BucStatus::Cursed)
        })
        .collect();
    let count = equipped_cursed.len();
    for id in equipped_cursed {
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc(id, BucStatus::Uncursed);
        entities.set_item_buc_known(id, true);
    }
    count
}

pub(crate) fn identify_inventory_target(world: &mut GameWorld, target: EntityId) -> Vec<GameEvent> {
    let mut events = Vec::new();
    if let Some(kind) = world
        .entities
        .get(target)
        .and_then(|entity| entity.item())
        .map(|(kind, _, _, _, _)| kind)
    {
        world.identify_item_kind(kind);
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc_known(target, true);
        entities.set_item_enchant_known(target, true);
        events.push(GameEvent::ItemIdentified {
            entity: world.player_id,
            item: target,
        });
    }
    events
}

pub(crate) fn remove_inventory_item(world: &mut GameWorld, item: EntityId) -> Result<(), String> {
    if let Some(slot) = world.inventory.slot_of(item) {
        unequip_slot(world, slot, item)?;
    }
    world
        .state_mut()
        .inventory
        .remove(item)
        .ok_or_else(|| "item is not in player inventory".to_string())?;
    Ok(())
}

pub fn put_on(world: &mut GameWorld, item: EntityId) -> Result<Option<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let target_slot = match data.class {
        ItemClass::Ring => {
            if world.inventory.slot_of(item).is_some() {
                return Ok(None);
            }
            if world.campaign.is_none() || world.inventory.equipped_ring.is_none() {
                EquipmentSlot::LeftRing
            } else if world.inventory.equipped_right_ring.is_none() {
                EquipmentSlot::RightRing
            } else {
                EquipmentSlot::LeftRing
            }
        }
        ItemClass::Amulet => {
            if world.inventory.equipped_amulet == Some(item) {
                return Ok(None);
            }
            EquipmentSlot::Amulet
        }
        _ => {
            return Err(if world.campaign.is_some() {
                "item is not a ring or amulet".to_string()
            } else {
                "item is not a ring".to_string()
            });
        }
    };
    let item_kind = data.kind;
    let mut next_inv = world.inventory.clone();
    next_inv.set_slot(target_slot, Some(item));
    let _ = compute_ac_for_inventory(world, &next_inv)?;
    if let Some(previous) = world.inventory.equipped_in(target_slot) {
        unequip_slot(world, target_slot, previous)?;
    }
    world
        .state_mut()
        .inventory
        .set_slot(target_slot, Some(item));
    if world.entities.get(item).and_then(|e| e.item_buc()) == Some(BucStatus::Cursed) {
        world.state_mut().entities.set_item_buc_known(item, true);
    }
    if data.ac_bonus != 0 {
        world
            .state_mut()
            .entities
            .set_item_enchant_known(item, true);
    }
    recompute_player_ac(world)?;
    if world.campaign.is_some() {
        world.identify_item_kind(item_kind);
    }
    Ok(Some(GameEvent::ItemEquipped {
        entity: world.player_id,
        item,
        slot: target_slot,
    }))
}

pub fn remove_ring(world: &mut GameWorld) -> Result<GameEvent, String> {
    let (slot, item) = if let Some(item) = world.inventory.equipped_right_ring {
        (EquipmentSlot::RightRing, item)
    } else if let Some(item) = world.inventory.equipped_ring {
        (EquipmentSlot::LeftRing, item)
    } else {
        return Err("no ring is equipped".to_string());
    };
    unequip_slot(world, slot, item)?;
    Ok(GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You take off your ring.".to_string(),
    })
}

pub fn take_off(world: &mut GameWorld, slot: EquipmentSlot) -> Result<GameEvent, String> {
    let item = world
        .inventory
        .equipped_in(slot)
        .ok_or_else(|| format!("no {} is equipped", slot.label()))?;
    unequip_slot(world, slot, item)?;
    Ok(GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("You take off your {}.", slot.label()),
    })
}

pub(crate) fn unequip_slot(
    world: &mut GameWorld,
    slot: EquipmentSlot,
    item: EntityId,
) -> Result<(), String> {
    if world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        == Some(BucStatus::Cursed)
    {
        let err = match slot {
            EquipmentSlot::Melee => "equipped weapon is cursed and cannot be unwielded",
            EquipmentSlot::LeftRing | EquipmentSlot::RightRing => {
                "equipped ring is cursed and cannot be removed"
            }
            EquipmentSlot::Amulet => "equipped amulet is cursed and cannot be removed",
            EquipmentSlot::Shield
            | EquipmentSlot::Helm
            | EquipmentSlot::Body
            | EquipmentSlot::Cloak
            | EquipmentSlot::Gloves
            | EquipmentSlot::Boots => "equipped armor is cursed and cannot be removed",
        };
        return Err(err.to_string());
    }
    if world.entities.actor_stats(world.player_id).is_none() {
        return Err("player actor stats are missing".to_string());
    }
    world.state_mut().inventory.set_slot(slot, None);
    if slot != EquipmentSlot::Melee {
        recompute_player_ac(world)?;
    }
    Ok(())
}

pub(crate) fn compute_ac_for_inventory(
    world: &GameWorld,
    inventory: &aihack_core::domain::inventory::Inventory,
) -> Result<i16, String> {
    let mut total_bonus: i16 = 0;
    for equipped in inventory.all_defense_items() {
        if let Some(data) = world.entities.item_data(equipped) {
            let enchant = world
                .entities
                .get(equipped)
                .and_then(|entity| entity.item_enchantment())
                .unwrap_or(0);
            let erosion = world
                .entities
                .get(equipped)
                .and_then(|entity| entity.item_erosion())
                .unwrap_or(0);
            total_bonus = total_bonus.saturating_add(
                data.ac_bonus
                    .saturating_add(i16::from(enchant))
                    .saturating_sub(i16::from(erosion)),
            );
        }
    }
    adventurer_template()
        .ac
        .checked_sub(total_bonus)
        .ok_or_else(|| "armor AC exceeds the supported range".to_string())
}

pub(crate) fn recompute_player_ac(world: &mut GameWorld) -> Result<(), String> {
    let derived_ac = compute_ac_for_inventory(world, &world.inventory)?;
    let player_id = world.player_id;
    if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
        stats.ac = derived_ac;
    }
    Ok(())
}

pub(crate) fn has_poison_resistance(world: &GameWorld) -> bool {
    world
        .effective_player_resistances()
        .contains(Resistances::POISON)
}

pub(crate) fn has_fire_resistance(world: &GameWorld) -> bool {
    world
        .effective_player_resistances()
        .contains(Resistances::FIRE)
}
