use aihack_core::{
    domain::{
        combat::DeathCause,
        entity::ActorKind,
        item::{BucStatus, ItemKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{TileKind, TrapKind},
    },
    event::{GameEvent, MessagePriority},
    position::Direction,
    rng::GameRng,
};

use crate::{
    appearance,
    systems::{combat::roll_die, death, items},
    world::GameWorld,
};

pub fn drink_from_tile(world: &mut GameWorld, rng: &mut GameRng) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("fountains and sinks are only available in campaign mode".to_string());
    }
    let pos = world.player_pos();
    let tile = world
        .current_map()
        .tile(pos)
        .map_err(|_| "current tile is invalid".to_string())?;
    let player_id = world.player_id;

    match tile {
        TileKind::Fountain => {
            let roll = rng.next_u64() % 6;
            match roll {
                0 => {
                    let heal = roll_die(rng, 4);
                    let state = world.state_mut();
                    state.nutrition = state.nutrition.saturating_add(15);
                    if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                        stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The cool water refreshes you.".to_string(),
                    }])
                }
                1 => {
                    let state = world.state_mut();
                    state.luck = state
                        .luck
                        .saturating_add(1)
                        .min(aihack_core::campaign::MAX_LUCK);
                    state.mp = state.mp.saturating_add(3).min(state.max_mp);
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The sparkling water fills you with vigor!".to_string(),
                    }])
                }
                2 => {
                    let bonus = 25 + (rng.next_u64() % 26) as u32;
                    world.state_mut().gold = world.gold.saturating_add(bonus);
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: format!("You find {bonus} gold coins at the bottom of the fountain!"),
                    }])
                }
                3 => {
                    let unidentified = world.inventory.entries.iter().find_map(|entry| {
                        let kind = world.entities.get(entry.item)?.item()?.0;
                        (!world.is_item_identified(kind)).then_some((entry.item, kind))
                    });
                    if let Some((item_id, kind)) = unidentified {
                        world.identify_item_kind(kind);
                        let entities = &mut world.state_mut().entities;
                        entities.set_item_buc_known(item_id, true);
                        entities.set_item_enchant_known(item_id, true);
                    } else if let Some(weapon_id) = world.inventory.equipped_melee {
                        let entities = &mut world.state_mut().entities;
                        if entities.get(weapon_id).and_then(|e| e.item_buc())
                            == Some(BucStatus::Uncursed)
                        {
                            entities.set_item_buc(weapon_id, BucStatus::Blessed);
                            entities.set_item_buc_known(weapon_id, true);
                        }
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "A water nymph blesses your gear before vanishing!".to_string(),
                    }])
                }
                4 => {
                    if !items::has_poison_resistance(world) {
                        let next_pois = world.poison_turns.max(3);
                        world.state_mut().poison_turns = next_pois;
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "The water tastes foul and stagnant!".to_string(),
                    }])
                }
                _ => {
                    let _ = world.current_map_mut().set_tile(pos, TileKind::DryFountain);
                    let spawn_pos = Direction::ALL.into_iter().find_map(|dir| {
                        let candidate = pos.offset(dir.delta());
                        let passable = world.current_map().tile(candidate).ok().is_some_and(|t| {
                            t.is_movement_passable()
                                && !matches!(t, TileKind::Water | TileKind::Lava)
                        });
                        (passable
                            && world
                                .entities
                                .alive_actor_at(world.current_level(), candidate)
                                .is_none())
                        .then_some(candidate)
                    });
                    if let Some(bat_pos) = spawn_pos {
                        let template = crate::domain::monster::try_monster_template_from_registry(
                            MonsterKind::Bat,
                            world.content_registry(),
                        )
                        .map_err(|err| err.to_string())?;
                        let level = world.current_level();
                        let bat_id = world
                            .state_mut()
                            .entities
                            .spawn_monster_with_template(MonsterKind::Bat, template, bat_pos)
                            .map_err(|err| err.to_string())?;
                        world
                            .state_mut()
                            .entities
                            .set_actor_location(bat_id, level, bat_pos);
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "The fountain dries up with a sudden hiss!".to_string(),
                    }])
                }
            }
        }
        TileKind::Sink => {
            let roll = rng.next_u64() % 5;
            let res = world.effective_player_resistances();
            match roll {
                0 => {
                    world.state_mut().nutrition = world.nutrition.saturating_add(10);
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The lukewarm tap water tastes dull but wet.".to_string(),
                    }])
                }
                1 => {
                    if !res.contains(Resistances::FIRE) {
                        let dmg = roll_die(rng, 4);
                        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                            stats.hp = stats.hp.saturating_sub(dmg);
                        }
                        let mut events = vec![GameEvent::Message {
                            priority: MessagePriority::Warning,
                            text: format!("The water is scalding hot! You take {dmg} damage!"),
                        }];
                        events.extend(death::collect_death_events_if_hp_depleted(
                            world,
                            player_id,
                            DeathCause::Trap {
                                trap: TrapKind::Fire,
                            },
                        )?);
                        return Ok(events);
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The hot tap water feels soothing.".to_string(),
                    }])
                }
                2 => {
                    if !res.contains(Resistances::COLD) {
                        let dmg = roll_die(rng, 4);
                        if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                            stats.hp = stats.hp.saturating_sub(dmg);
                        }
                        let mut events = vec![GameEvent::Message {
                            priority: MessagePriority::Warning,
                            text: format!("The water is freezing cold! You take {dmg} damage!"),
                        }];
                        events.extend(death::collect_death_events_if_hp_depleted(
                            world,
                            player_id,
                            DeathCause::Drowning,
                        )?);
                        return Ok(events);
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The cold water feels crisp.".to_string(),
                    }])
                }
                3 => {
                    world.state_mut().gold = world.gold.saturating_add(20);
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You fish 20 gold pieces out of the sink drain!".to_string(),
                    }])
                }
                _ => {
                    if !items::has_poison_resistance(world) {
                        let next_pois = world.poison_turns.max(2);
                        world.state_mut().poison_turns = next_pois;
                    }
                    if let Some(mut campaign) = world.campaign {
                        if campaign.attributes.dex > 3 {
                            campaign.attributes.dex -= 1;
                            let hit = campaign.hit_bonus();
                            world.state_mut().campaign = Some(campaign);
                            if let Some(stats) =
                                world.state_mut().entities.actor_stats_mut(player_id)
                            {
                                stats.hit_bonus = hit;
                            }
                        }
                    }
                    Ok(vec![GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "You swallow awful black sludge from the sink!".to_string(),
                    }])
                }
            }
        }
        _ => Err("there is no fountain or sink here to drink from".to_string()),
    }
}

pub fn on_player_enter_tile(
    seed: u64,
    world: &mut GameWorld,
    rng: &mut GameRng,
) -> (Vec<GameEvent>, Option<DeathCause>) {
    let pos = world.player_pos();
    let Ok(tile) = world.current_map().tile(pos) else {
        return (Vec::new(), None);
    };
    let player_id = world.player_id;
    match tile {
        TileKind::Water => {
            if world.status_timers.levitation > 0 {
                return (
                    vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You float safely over the water.".to_string(),
                    }],
                    None,
                );
            }
            if i32::from(world.carried_weight()).saturating_mul(2)
                <= i32::from(world.carrying_capacity())
            {
                let mut events = Vec::new();
                world.state_mut().nutrition = world.nutrition.saturating_sub(2);
                // 점등된 오일 램프 소등
                for entry in world.inventory.entries.clone() {
                    if let Some(entity) = world.entities.get(entry.item) {
                        if entity
                            .item()
                            .is_some_and(|(k, _, _, _, _)| k == ItemKind::OilLamp)
                            && entity.item_tool_active() == Some(true)
                        {
                            world
                                .state_mut()
                                .entities
                                .set_item_tool_active(entry.item, false);
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: "Your oil lamp sputters out in the water!".to_string(),
                            });
                        }
                    }
                }
                // 철제 착용 장비 1개 녹
                let iron_equipped = world
                    .inventory
                    .equipped_slots()
                    .into_iter()
                    .filter_map(|(_, opt)| opt)
                    .find(|id| {
                        let Some(entity) = world.entities.get(*id) else {
                            return false;
                        };
                        if entity.item_artifact_id().is_some() {
                            return false;
                        }
                        let erosion = entity.item_erosion().unwrap_or(0);
                        if erosion >= 3 {
                            return false;
                        }
                        let Some(kind) = world.entities.item_data(*id).map(|d| d.kind) else {
                            return false;
                        };
                        let mat = world
                            .content_registry()
                            .item(aihack_content::item_id(kind))
                            .and_then(|d| d.material.as_deref());
                        matches!(mat, Some("iron" | "steel"))
                    });
                if let Some(gear_id) = iron_equipped {
                    let erosion = world
                        .entities
                        .get(gear_id)
                        .and_then(|e| e.item_erosion())
                        .unwrap_or(0);
                    let next = (erosion + 1).min(3);
                    world.state_mut().entities.set_item_erosion(gear_id, next);
                    let _ = items::recompute_player_ac(world);
                    if let Some(kind) = world.entities.item_data(gear_id).map(|d| d.kind) {
                        let gear_name =
                            appearance::display_name_for_world_item(seed, world, gear_id, kind);
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Warning,
                            text: format!("Your {gear_name} looks rusty!"),
                        });
                    }
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You swim through the water.".to_string(),
                });
                (events, None)
            } else {
                let dmg = roll_die(rng, 6) + roll_die(rng, 6);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                (
                    vec![GameEvent::Message {
                        priority: MessagePriority::Danger,
                        text: format!("You struggle in the deep water for {dmg} drowning damage!"),
                    }],
                    Some(DeathCause::Drowning),
                )
            }
        }
        TileKind::Lava => {
            if world.status_timers.levitation > 0 {
                return (
                    vec![GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You float safely over the molten lava.".to_string(),
                    }],
                    None,
                );
            }
            if items::has_fire_resistance(world) {
                let dmg = roll_die(rng, 6);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                (
                    vec![GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: format!("The molten lava sears your feet for {dmg} damage!"),
                    }],
                    Some(DeathCause::Lava),
                )
            } else {
                let dmg = roll_die(rng, 10) + roll_die(rng, 10) + roll_die(rng, 10);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                (
                    vec![GameEvent::Message {
                        priority: MessagePriority::Danger,
                        text: format!("You plunge into molten lava for {dmg} fire damage!"),
                    }],
                    Some(DeathCause::Lava),
                )
            }
        }
        _ => (Vec::new(), None),
    }
}

pub fn monster_can_traverse(kind: MonsterKind, tile: TileKind) -> bool {
    match tile {
        TileKind::Water | TileKind::Lava => matches!(kind, MonsterKind::Bat | MonsterKind::Dragon),
        _ => true,
    }
}

pub fn actor_kind_can_traverse(kind: Option<ActorKind>, tile: TileKind) -> bool {
    match kind {
        Some(ActorKind::Monster(mk)) => monster_can_traverse(mk, tile),
        _ => true,
    }
}
