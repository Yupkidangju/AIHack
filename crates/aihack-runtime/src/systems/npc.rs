use aihack_core::{
    campaign::MAX_LUCK,
    domain::{
        entity::{ActorKind, EntityLocation, Faction},
        monster::MonsterKind,
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
    rng::GameRng,
};

use crate::{
    appearance,
    systems::{combat, shop},
    world::GameWorld,
};

pub const ORACLE_CONSULTATION_FEE: u32 = 50;
pub const MAX_SHOP_DEBT: u32 = 1_000_000;

const ORACLE_HINTS: [&str; 6] = [
    "Oracle whispers: \"Engrave Elbereth in dust or fire when surrounded by foes.\"",
    "Oracle whispers: \"A blindfold shields your eyes from paralyzing gazes.\"",
    "Oracle whispers: \"Dip a long sword into a fountain when lawful and seasoned.\"",
    "Oracle whispers: \"Never nest one bag of holding inside another.\"",
    "Oracle whispers: \"Your pet avoids cursed gear on the floor; watch its steps.\"",
    "Oracle whispers: \"Unpaid goods left unpaid anger both shopkeepers and the watch.\"",
];

pub fn shop_tile_pos(world: &GameWorld) -> Option<Pos> {
    let map = world.current_map();
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            if map.tile(pos).ok() == Some(TileKind::Shop) {
                return Some(pos);
            }
        }
    }
    None
}

pub fn is_in_shop_zone(world: &GameWorld, pos: Pos) -> bool {
    shop_tile_pos(world).is_some_and(|shop_pos| shop_pos.chebyshev_distance(pos) <= 4)
}

pub fn has_hostile_shopkeeper_on_level(world: &GameWorld) -> bool {
    let level = world.current_level();
    world.entities.entities().iter().any(|e| {
        matches!(
            e.actor(),
            Some((
                ActorKind::Monster(MonsterKind::Shopkeeper),
                Faction::Hostile,
                l,
                _,
                _,
                true
            )) if l == level
        )
    })
}

pub fn has_peaceful_shopkeeper_on_level(world: &GameWorld) -> bool {
    let level = world.current_level();
    world.entities.entities().iter().any(|e| {
        matches!(
            e.actor(),
            Some((
                ActorKind::Monster(MonsterKind::Shopkeeper),
                Faction::Shopkeeper,
                l,
                _,
                _,
                true
            )) if l == level
        )
    })
}

pub fn unpaid_item_price_for_data(
    campaign: Option<aihack_core::campaign::CampaignState>,
    data: &aihack_core::domain::item::ItemData,
) -> u32 {
    let base = data.base_price.max(10);
    campaign
        .map(|c| c.attributes.cha_buy_price(base))
        .unwrap_or(base)
}

pub fn unpaid_item_price(world: &GameWorld, item: EntityId) -> u32 {
    let Some(data) = world.entities.item_data(item) else {
        return 10;
    };
    unpaid_item_price_for_data(world.campaign, data)
}

pub fn priest_donation_fee(world: &GameWorld) -> u32 {
    let level = world.campaign.map(|c| c.level()).unwrap_or(1);
    (u32::from(level).saturating_mul(20)).clamp(20, 200)
}

pub fn anger_shop_and_watch(world: &mut GameWorld, message: &str) -> Vec<GameEvent> {
    let level = world.current_level();
    let targets: Vec<EntityId> = world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(mk), faction, l, _, _, true) = e.actor()? else {
                return None;
            };
            if l != level {
                return None;
            }
            if (mk == MonsterKind::Shopkeeper && faction == Faction::Shopkeeper)
                || (mk == MonsterKind::Watchman && faction == Faction::Peaceful)
            {
                Some(e.id)
            } else {
                None
            }
        })
        .collect();
    if targets.is_empty() {
        return Vec::new();
    }
    let entities = &mut world.state_mut().entities;
    for id in targets {
        entities.set_actor_faction(id, Faction::Hostile);
    }
    vec![GameEvent::Message {
        priority: MessagePriority::Warning,
        text: message.to_string(),
    }]
}

pub fn provoke_actor(world: &mut GameWorld, target: EntityId) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let Some(entity) = world.entities.get(target) else {
        return Vec::new();
    };
    let Some((ActorKind::Monster(mk), faction, level, _, _, true)) = entity.actor() else {
        return Vec::new();
    };
    let mut events = Vec::new();
    match faction {
        Faction::Peaceful | Faction::Shopkeeper => {
            let name = aihack_content::monster_display_name(mk, world.content_registry());
            world
                .state_mut()
                .entities
                .set_actor_faction(target, Faction::Hostile);
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: format!("The {name} turns hostile!"),
            });
            let watch_ids: Vec<EntityId> = world
                .entities
                .entities()
                .iter()
                .filter_map(|e| {
                    let (
                        ActorKind::Monster(MonsterKind::Watchman),
                        Faction::Peaceful,
                        l,
                        _,
                        _,
                        true,
                    ) = e.actor()?
                    else {
                        return None;
                    };
                    (l == level && e.id != target).then_some(e.id)
                })
                .collect();
            if !watch_ids.is_empty() {
                for id in watch_ids {
                    world
                        .state_mut()
                        .entities
                        .set_actor_faction(id, Faction::Hostile);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "The watchman shouts: Halt, criminal!".to_string(),
                });
            }
        }
        Faction::Pet => {
            let name = aihack_content::monster_display_name(mk, world.content_registry());
            let next_tame = entity.actor_tameness().unwrap_or(0).saturating_sub(3);
            let entities = &mut world.state_mut().entities;
            entities.set_actor_tameness(target, next_tame);
            if next_tame == 0 {
                entities.set_actor_faction(target, Faction::Hostile);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("Your {name} turns hostile after your attack!"),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("Your {name} yelps in betrayal!"),
                });
            }
        }
        _ => {}
    }
    events
}

pub fn shop_debt_on(world: &GameWorld, level: LevelId) -> u32 {
    world
        .shop_debts
        .iter()
        .find(|&&(l, _)| l == level)
        .map(|&(_, amt)| amt)
        .unwrap_or(0)
}

fn sync_total_shop_debt(
    state: &mut aihack_core::world::WorldState<crate::domain::entity::EntityStore>,
) {
    state.shop_debts.retain(|&(_, amt)| amt > 0);
    state.shop_debts.sort_by_key(|&(lvl, _)| lvl);
    state.shop_debt = state
        .shop_debts
        .iter()
        .fold(0_u32, |acc, &(_, amt)| acc.saturating_add(amt))
        .min(MAX_SHOP_DEBT);
}

pub fn add_shop_debt(world: &mut GameWorld, level: LevelId, amount: u32) {
    if amount == 0 {
        return;
    }
    let headroom = MAX_SHOP_DEBT.saturating_sub(world.shop_debt);
    let actual = amount.min(headroom);
    if actual == 0 {
        return;
    }
    let state = world.state_mut();
    if let Some((_, existing)) = state.shop_debts.iter_mut().find(|(l, _)| *l == level) {
        *existing = existing.saturating_add(actual);
    } else {
        state.shop_debts.push((level, actual));
    }
    sync_total_shop_debt(state);
}

pub fn sub_shop_debt(world: &mut GameWorld, level: LevelId, amount: u32) {
    if amount == 0 {
        return;
    }
    let state = world.state_mut();
    if let Some((_, existing)) = state.shop_debts.iter_mut().find(|(l, _)| *l == level) {
        *existing = existing.saturating_sub(amount);
    }
    sync_total_shop_debt(state);
}

pub fn clear_shop_debt_on(world: &mut GameWorld, level: LevelId) {
    let state = world.state_mut();
    state.shop_debts.retain(|&(l, _)| l != level);
    sync_total_shop_debt(state);
}

pub fn check_shop_theft_on_move_or_stairs(
    world: &mut GameWorld,
    leaving_level: bool,
) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let debt = shop_debt_on(world, world.current_level());
    if debt == 0 {
        return Vec::new();
    }
    if (leaving_level || !is_in_shop_zone(world, world.player_pos()))
        && has_peaceful_shopkeeper_on_level(world)
    {
        return anger_shop_and_watch(
            world,
            &format!("Stop, thief! You owe {debt} gold to the shopkeeper!"),
        );
    }
    Vec::new()
}

pub fn on_shopkeeper_died(world: &mut GameWorld, level: LevelId) {
    let any_other_alive = world.entities.entities().iter().any(|e| {
        matches!(
            e.actor(),
            Some((ActorKind::Monster(MonsterKind::Shopkeeper), _, l, _, _, true)) if l == level
        )
    });
    if any_other_alive {
        return;
    }
    let mut to_clear = Vec::new();
    for e in world.entities.entities() {
        if e.item_unpaid() != Some(true) {
            continue;
        }
        let Some((_, _, loc, _, _)) = e.item() else {
            continue;
        };
        match loc {
            EntityLocation::Inventory { owner } if owner == world.player_id => {
                if e.item_unpaid_level() == Some(level) {
                    to_clear.push(e.id);
                }
            }
            EntityLocation::OnMap { level: l, .. } if l == level => {
                to_clear.push(e.id);
            }
            _ => {}
        }
    }
    clear_shop_debt_on(world, level);
    let entities = &mut world.state_mut().entities;
    for id in to_clear {
        entities.set_item_unpaid(id, false);
        entities.set_item_unpaid_level(id, None);
    }
}

pub fn adjacent_peaceful_npc(world: &GameWorld, kind: MonsterKind) -> Option<EntityId> {
    let level = world.current_level();
    let player_pos = world.player_pos();
    world.entities.entities().iter().find_map(|e| {
        let (ActorKind::Monster(mk), faction, l, pos, _, true) = e.actor()? else {
            return None;
        };
        (mk == kind
            && matches!(faction, Faction::Peaceful | Faction::Shopkeeper)
            && l == level
            && pos.chebyshev_distance(player_pos) <= 1)
            .then_some(e.id)
    })
}

pub fn can_pay(world: &GameWorld) -> bool {
    if world.campaign.is_none() {
        return false;
    }
    let cur_debt = shop_debt_on(world, world.current_level());
    if cur_debt > 0 && has_peaceful_shopkeeper_on_level(world) && world.gold >= cur_debt {
        return true;
    }
    if adjacent_peaceful_npc(world, MonsterKind::TemplePriest).is_some()
        && world.gold >= priest_donation_fee(world)
    {
        return true;
    }
    if adjacent_peaceful_npc(world, MonsterKind::Oracle).is_some()
        && world.gold >= ORACLE_CONSULTATION_FEE
    {
        return true;
    }
    false
}

pub fn pay(seed: u64, world: &mut GameWorld, rng: &mut GameRng) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("paying NPCs is only available in campaign mode".to_string());
    }
    let current_level = world.current_level();
    let debt = shop_debt_on(world, current_level);
    if debt > 0 && has_peaceful_shopkeeper_on_level(world) {
        if world.gold < debt {
            return Err(format!("not enough gold to pay {debt} gold debt"));
        }
        world.state_mut().gold -= debt;
        clear_shop_debt_on(world, current_level);
        let carried: Vec<EntityId> = world.inventory.entries.iter().map(|e| e.item).collect();
        for id in carried {
            let belongs_here = world.entities.get(id).is_some_and(|e| {
                e.item_unpaid() == Some(true)
                    && e.item_unpaid_level().unwrap_or(current_level) == current_level
            });
            if belongs_here {
                world.state_mut().entities.set_item_unpaid(id, false);
                world.state_mut().entities.set_item_unpaid_level(id, None);
                world.state_mut().entities.set_item_buc_known(id, true);
            }
        }
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("You pay {debt} gold to the shopkeeper. Thank you for your business!"),
        }]);
    }

    if adjacent_peaceful_npc(world, MonsterKind::TemplePriest).is_some() {
        let fee = priest_donation_fee(world);
        if world.gold < fee {
            return Err(format!(
                "the temple priest requests a donation of {fee} gold"
            ));
        }
        world.state_mut().gold -= fee;
        if let Some(mut c) = world.campaign {
            c.god_anger = c.god_anger.saturating_sub(3);
            world.state_mut().campaign = Some(c);
        }
        let state = world.state_mut();
        state.luck = state.luck.saturating_add(1).min(MAX_LUCK);
        state.poison_turns = 0;
        state.status_timers.blind = 0;
        state.status_timers.confused = 0;
        state.status_timers.stunned = 0;
        let mut events = vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!(
                "You donate {fee} gold. The temple priest blesses you with divine favor!"
            ),
        }];
        if let Some((item_id, kind)) = find_first_unidentified_inventory_item(world) {
            world.identify_item_kind(kind);
            world.state_mut().entities.set_item_buc_known(item_id, true);
            world
                .state_mut()
                .entities
                .set_item_enchant_known(item_id, true);
            let name = appearance::display_name_for_world_item(seed, world, item_id, kind);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("The priest identifies your {name}."),
            });
        } else {
            let heal = combat::roll_die(rng, 8) + 4;
            let player_id = world.player_id;
            if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
                let next = stats.hp.saturating_add(heal).min(stats.max_hp);
                let gained = next - stats.hp;
                stats.hp = next;
                if gained > 0 {
                    events.push(GameEvent::EntityHealed {
                        entity: player_id,
                        amount: gained,
                        hp_after: next,
                    });
                }
            }
        }
        return Ok(events);
    }

    if adjacent_peaceful_npc(world, MonsterKind::Oracle).is_some() {
        if world.gold < ORACLE_CONSULTATION_FEE {
            return Err(format!(
                "a major consultation costs {ORACLE_CONSULTATION_FEE} gold"
            ));
        }
        world.state_mut().gold -= ORACLE_CONSULTATION_FEE;
        world.state_mut().luck = world.luck.saturating_add(1).min(MAX_LUCK);
        let reveal_events = crate::systems::traps::reveal_all_hidden_tiles(world);
        let revealed = reveal_events.len();
        let mut events = vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!(
                "You pay {ORACLE_CONSULTATION_FEE} gold. The Oracle reveals {revealed} hidden features on this level!"
            ),
        }];
        events.extend(reveal_events);
        if let Some((item_id, kind)) = find_first_unidentified_inventory_item(world) {
            world.identify_item_kind(kind);
            world.state_mut().entities.set_item_buc_known(item_id, true);
            world
                .state_mut()
                .entities
                .set_item_enchant_known(item_id, true);
            let name = appearance::display_name_for_world_item(seed, world, item_id, kind);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("The Oracle's vision identifies your {name}."),
            });
        }
        return Ok(events);
    }

    Err("there is no shop debt or adjacent NPC to pay".to_string())
}

fn find_first_unidentified_inventory_item(
    world: &GameWorld,
) -> Option<(EntityId, aihack_core::domain::item::ItemKind)> {
    world.inventory.entries.iter().find_map(|entry| {
        let entity = world.entities.get(entry.item)?;
        let (kind, _, _, _, _) = entity.item()?;
        let needs = !world.is_item_identified(kind)
            || !entity.item_buc_known().unwrap_or(false)
            || !entity.item_enchant_known().unwrap_or(false);
        needs.then_some((entry.item, kind))
    })
}

pub fn can_chat_in_direction(world: &GameWorld, direction: Direction) -> bool {
    if world.campaign.is_none() {
        return false;
    }
    let pos = world.player_pos().offset(direction.delta());
    world
        .entities
        .alive_friendly_at(world.current_level(), pos)
        .is_some()
}

pub fn chat(
    seed: u64,
    turn: u64,
    world: &GameWorld,
    direction: Direction,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("chatting is only available in campaign mode".to_string());
    }
    let pos = world.player_pos().offset(direction.delta());
    let Some(target_id) = world.entities.alive_actor_at(world.current_level(), pos) else {
        return Err("there is no one there to chat with".to_string());
    };
    let Some(entity) = world.entities.get(target_id) else {
        return Err("there is no one there to chat with".to_string());
    };
    let Some((ActorKind::Monster(mk), faction, _, _, _, true)) = entity.actor() else {
        return Err("there is no one there to chat with".to_string());
    };
    let name = aihack_content::monster_display_name(mk, world.content_registry());
    let text = match faction {
        Faction::Hostile => format!("The {name} snarls aggressively at you!"),
        Faction::Pet => {
            let tameness = entity.actor_tameness().unwrap_or(0);
            let hunger = entity.actor_pet_hunger().unwrap_or(0);
            let xp = entity.actor_pet_xp().unwrap_or(0);
            let mood = if hunger < 150 {
                "hungry"
            } else if tameness >= 8 {
                "devoted"
            } else {
                "friendly"
            };
            format!("Your {name} responds affectionately ({mood}, tameness {tameness}, xp {xp}).")
        }
        Faction::Shopkeeper => {
            let debt = shop_debt_on(world, world.current_level());
            if debt > 0 {
                format!("Shopkeeper says: \"You currently owe {debt} gold. Pay before leaving!\"")
            } else {
                "Shopkeeper says: \"Welcome! Pick up wares to buy on tab, or press $ to trade.\""
                    .to_string()
            }
        }
        Faction::Peaceful | Faction::Neutral => match mk {
            MonsterKind::TemplePriest => {
                let fee = priest_donation_fee(world);
                format!(
                    "Temple priest says: \"Peace be with you. Offer {fee} gold via Pay for a divine blessing.\""
                )
            }
            MonsterKind::Oracle => {
                let idx = ((seed.wrapping_add(turn)) as usize) % ORACLE_HINTS.len();
                format!(
                    "{} (Pay {ORACLE_CONSULTATION_FEE} gold for a major consultation.)",
                    ORACLE_HINTS[idx]
                )
            }
            MonsterKind::Watchman => {
                let debt = shop_debt_on(world, world.current_level());
                if debt > 0 {
                    format!(
                        "Watchman warns: \"Make sure you pay your {debt} gold shop bill before leaving!\""
                    )
                } else {
                    "Watchman says: \"Keep the peace in Minetown, adventurer.\"".to_string()
                }
            }
            _ => format!("The {name} nods quietly."),
        },
        Faction::Player => return Err("cannot chat with yourself".to_string()),
    };
    let _ = shop::is_on_shop;
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text,
    }])
}
