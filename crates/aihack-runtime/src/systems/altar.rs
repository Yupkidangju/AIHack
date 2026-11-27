use aihack_core::{
    campaign::{Alignment, MAX_GOD_ANGER, MAX_LUCK, MAX_PRAYER_TIMEOUT, MIN_LUCK},
    domain::{
        combat::DeathCause,
        item::{BucStatus, ItemClass, ItemKind},
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    systems::{artifacts, combat, death, items},
    world::GameWorld,
};

pub fn altar_alignment(world: &GameWorld) -> Option<Alignment> {
    world.campaign?;
    world
        .levels
        .get(world.current_level())
        .and_then(|level| level.altar_alignment(world.player_pos()))
}

pub fn is_on_altar(world: &GameWorld) -> bool {
    world.campaign.is_some()
        && world.current_map().tile(world.player_pos()).ok() == Some(TileKind::Altar)
}

pub fn offer(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if !is_on_altar(world) {
        return Err("must be standing on an altar to offer".to_string());
    }
    let Some(mut campaign) = world.campaign else {
        return Err("altar offering requires campaign mode".to_string());
    };
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let entity = world
        .entities
        .get(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    let (kind, data, _, _, _) = entity
        .item()
        .ok_or_else(|| "entity is not an item".to_string())?;
    let data = *data;
    if data.kind == ItemKind::AmuletAscension {
        return Err("cannot offer the Amulet of Ascension".to_string());
    }
    if !entity.item_contained_items().unwrap_or_default().is_empty() {
        return Err("cannot offer a non-empty container".to_string());
    }
    let active = entity.item_tool_active() == Some(true);
    let buc = entity.item_buc();
    let corpse_diff = if kind == ItemKind::CorpseJackal {
        1
    } else if let Some(mk) = entity.item_corpse_monster() {
        crate::domain::monster::try_monster_template_from_registry(mk, world.content_registry())
            .map(|t| t.difficulty)
            .unwrap_or(1)
            .max(1)
    } else {
        0
    };
    let cross_aligned = altar_alignment(world).is_some_and(|align| align != campaign.alignment);
    if cross_aligned && data.class != ItemClass::Corpse {
        return Err("only a corpse can convert a cross-aligned altar".to_string());
    }
    if data.kind == ItemKind::Blindfold && active {
        if buc == Some(BucStatus::Cursed) {
            return Err("The blindfold is cursed and stuck to your face!".to_string());
        }
        world.state_mut().entities.set_item_tool_active(item, false);
    }
    if data.kind == ItemKind::OilLamp && active {
        world.state_mut().entities.set_item_tool_active(item, false);
    }

    items::consume_one_from_inventory(world, item)?;
    let player_id = world.player_id;
    let mut events = vec![GameEvent::ItemConsumed {
        entity: player_id,
        item,
    }];

    if cross_aligned {
        let level_id = world.current_level();
        let pos = world.player_pos();
        if let Some(level) = world.state_mut().levels.get_mut(level_id) {
            level.set_altar_alignment(pos, campaign.alignment);
        }
        campaign.god_anger = campaign.god_anger.saturating_sub(1);
        let state = world.state_mut();
        state.campaign = Some(campaign);
        state.luck = (state.luck + 1).clamp(MIN_LUCK, MAX_LUCK);
        state.prayer_cooldown = state.prayer_cooldown.saturating_sub(10);
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("You convert the altar to {}!", campaign.alignment.as_str()),
        });
        return Ok(events);
    }

    if matches!(data.class, ItemClass::Corpse | ItemClass::Food) {
        let luck_before = world.luck;
        let anger_before = campaign.god_anger;
        campaign.prayer_timeout = campaign.prayer_timeout.saturating_sub(25);
        if anger_before > 0 {
            campaign.god_anger = anger_before.saturating_sub(2);
            let state = world.state_mut();
            state.campaign = Some(campaign);
            state.prayer_cooldown = state.prayer_cooldown.saturating_sub(10);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Your offering appeases your deity's anger.".to_string(),
            });
        } else {
            let state = world.state_mut();
            state.campaign = Some(campaign);
            state.luck = state
                .luck
                .saturating_add(1)
                .clamp(MIN_LUCK, 3.max(luck_before + 1).min(MAX_LUCK));
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
            if data.class == ItemClass::Corpse && (luck_before >= 2 || corpse_diff >= 2) {
                if let Some((_gift_id, def)) = artifacts::try_grant_sacrifice_artifact(world, rng)?
                {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: format!("A golden light bestows {} upon the altar!", def.name),
                    });
                }
            }
        }
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

pub fn resolve_campaign_pray(
    world: &mut GameWorld,
    rng: &mut GameRng,
) -> Result<Vec<GameEvent>, String> {
    if world.prayer_cooldown > 0 {
        return Err("prayer is on cooldown".to_string());
    }
    let Some(mut campaign) = world.campaign else {
        return Err("campaign state required".to_string());
    };
    let cooldown = campaign.attributes.wis_prayer_cooldown();
    let on_altar = is_on_altar(world);
    let cross_aligned =
        on_altar && altar_alignment(world).is_some_and(|align| align != campaign.alignment);
    let player_id = world.player_id;

    if cross_aligned {
        campaign.god_anger = (campaign.god_anger.saturating_add(2)).min(MAX_GOD_ANGER);
        campaign.prayer_timeout = 200;
        let state = world.state_mut();
        state.campaign = Some(campaign);
        state.prayer_cooldown = cooldown;
        state.luck = (state.luck - 2).clamp(MIN_LUCK, MAX_LUCK);
        let dmg = combat::roll_die(rng, 8) + combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
            stats.hp = stats.hp.saturating_sub(dmg);
        }
        let mut events = vec![
            GameEvent::PrayerOffered {
                entity: player_id,
                cooldown_after: world.prayer_cooldown,
            },
            GameEvent::Message {
                priority: MessagePriority::Danger,
                text: format!(
                    "You pray at a hostile altar and suffer {dmg} damage from divine wrath!"
                ),
            },
        ];
        events.extend(death::collect_death_events_if_hp_depleted(
            world,
            player_id,
            DeathCause::DivineWrath,
        )?);
        return Ok(events);
    }

    if campaign.god_anger > 0 || campaign.prayer_timeout > 0 {
        campaign.god_anger = (campaign.god_anger.saturating_add(1)).min(MAX_GOD_ANGER);
        campaign.prayer_timeout =
            (campaign.prayer_timeout.saturating_add(100)).min(MAX_PRAYER_TIMEOUT);
        let state = world.state_mut();
        state.campaign = Some(campaign);
        state.prayer_cooldown = cooldown;
        state.luck = (state.luck - 1).clamp(MIN_LUCK, MAX_LUCK);
        let dmg = combat::roll_die(rng, 6) + combat::roll_die(rng, 6);
        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
            stats.hp = stats.hp.saturating_sub(dmg);
        }
        let mut events = vec![
            GameEvent::PrayerOffered {
                entity: player_id,
                cooldown_after: world.prayer_cooldown,
            },
            GameEvent::Message {
                priority: MessagePriority::Danger,
                text: format!("Your deity is displeased and smites you for {dmg} damage!"),
            },
        ];
        events.extend(death::collect_death_events_if_hp_depleted(
            world,
            player_id,
            DeathCause::DivineWrath,
        )?);
        return Ok(events);
    }

    campaign.prayer_timeout = 100;
    let state = world.state_mut();
    state.campaign = Some(campaign);
    state.prayer_cooldown = cooldown;
    state.luck = state
        .luck
        .saturating_add(1)
        .clamp(MIN_LUCK, 3.max(state.luck + 1).min(MAX_LUCK));
    let mut events = vec![GameEvent::PrayerOffered {
        entity: player_id,
        cooldown_after: world.prayer_cooldown,
    }];
    if on_altar {
        events.extend(pray_at_altar(world));
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
