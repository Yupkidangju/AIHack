use aihack_core::{
    domain::{
        entity::{EntityLocation, Faction},
        item::{BucStatus, ItemClass, ItemKind, ToolAction},
        tile::{DoorState, TileKind},
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    position::{Direction, Pos},
    rng::GameRng,
};

use crate::{systems::movement, world::GameWorld};

pub fn has_active_light_source(world: &GameWorld) -> bool {
    let level = world.current_level();
    let pos = world.player_pos();
    world
        .inventory
        .entries
        .iter()
        .any(|entry| is_lit_lamp(world, entry.item))
        || world
            .entities
            .items_at(level, pos)
            .into_iter()
            .any(|id| is_lit_lamp(world, id))
}

fn is_lit_lamp(world: &GameWorld, id: EntityId) -> bool {
    let Some(entity) = world.entities.get(id) else {
        return false;
    };
    let Some((kind, _, _, _, charges)) = entity.item() else {
        return false;
    };
    kind == ItemKind::OilLamp && entity.item_tool_active() == Some(true) && charges.unwrap_or(0) > 0
}

pub fn is_player_blindfolded(world: &GameWorld) -> bool {
    world.inventory.entries.iter().any(|entry| {
        world.entities.get(entry.item).is_some_and(|entity| {
            entity
                .item()
                .is_some_and(|(kind, _, _, _, _)| kind == ItemKind::Blindfold)
                && entity.item_tool_active() == Some(true)
        })
    })
}

pub fn tick_active_lamps(world: &mut GameWorld) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let player_id = world.player_id;
    let current_level = world.current_level();
    let player_pos = world.player_pos();
    let active_lamps: Vec<(EntityId, u8, EntityLocation)> = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            let (kind, _, loc, _, charges) = entity.item()?;
            if kind == ItemKind::OilLamp
                && loc != EntityLocation::Consumed
                && entity.item_tool_active() == Some(true)
            {
                Some((entity.id, charges.unwrap_or(0), loc))
            } else {
                None
            }
        })
        .collect();

    let mut events = Vec::new();
    let mut burned_out_any = false;
    for (id, charges, loc) in active_lamps {
        let next = charges.saturating_sub(1);
        world.state_mut().entities.set_item_charges(id, Some(next));
        if next == 0 {
            world.state_mut().entities.set_item_tool_active(id, false);
            burned_out_any = true;
            let visible = loc == (EntityLocation::Inventory { owner: player_id })
                || loc
                    == (EntityLocation::OnMap {
                        level: current_level,
                        pos: player_pos,
                    });
            if visible {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "Your oil lamp flickers out.".to_string(),
                });
            }
        }
    }
    if burned_out_any {
        world.mark_current_level_explored();
    }
    events
}

pub fn apply(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    direction: Option<Direction>,
) -> Result<Vec<GameEvent>, String> {
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let data = *world
        .entities
        .item_data(item)
        .ok_or_else(|| "entity is not an item".to_string())?;
    if data.class != ItemClass::Tool {
        return Err("item is not a tool".to_string());
    }
    let Some(action) = data.tool_action else {
        return Err("tool has no apply action".to_string());
    };
    let buc = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_buc())
        .unwrap_or(BucStatus::Uncursed);
    let charges = world.entities.item_charges(item);
    let active = world
        .entities
        .get(item)
        .and_then(|entity| entity.item_tool_active())
        .unwrap_or(false);

    match action {
        ToolAction::Container => {
            Err("use container actions (#loot) for bags and chests".to_string())
        }
        ToolAction::Unlock => apply_unlock(world, data.kind, buc, direction),
        ToolAction::Dig => apply_dig(world, rng, data.kind, item, buc, direction),
        ToolAction::Music => apply_whistle(world),
        ToolAction::Light => apply_lamp(world, item, active, charges),
        ToolAction::Blindfold => apply_blindfold(world, item, active, buc),
        ToolAction::Magic => apply_unicorn_horn(world, item, buc),
    }
}

fn apply_unlock(
    world: &mut GameWorld,
    kind: ItemKind,
    buc: BucStatus,
    direction: Option<Direction>,
) -> Result<Vec<GameEvent>, String> {
    if let Some(dir) = direction {
        let target_pos = world.player_pos().offset(dir.delta());
        let tile = world
            .current_map()
            .tile(target_pos)
            .map_err(|_| "nothing to unlock there".to_string())?;
        return match tile {
            TileKind::Door(DoorState::Locked) => {
                world
                    .current_map_mut()
                    .set_tile(target_pos, TileKind::Door(DoorState::Closed))
                    .map_err(|e| e.to_string())?;
                Ok(vec![
                    GameEvent::DoorChanged {
                        pos: target_pos,
                        from: DoorState::Locked,
                        to: DoorState::Closed,
                    },
                    GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You unlock the door.".to_string(),
                    },
                ])
            }
            TileKind::Door(DoorState::Closed) if kind == ItemKind::SkeletonKey => {
                world
                    .current_map_mut()
                    .set_tile(target_pos, TileKind::Door(DoorState::Locked))
                    .map_err(|e| e.to_string())?;
                Ok(vec![
                    GameEvent::DoorChanged {
                        pos: target_pos,
                        from: DoorState::Closed,
                        to: DoorState::Locked,
                    },
                    GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "You lock the door.".to_string(),
                    },
                ])
            }
            _ => Err("there is no locked door in that direction".to_string()),
        };
    }

    let level = world.current_level();
    let pos = world.player_pos();
    let chest = world
        .entities
        .items_at(level, pos)
        .into_iter()
        .chain(world.inventory.entries.iter().map(|entry| entry.item))
        .find(|id| {
            world
                .entities
                .item_data(*id)
                .is_some_and(|d| d.kind == ItemKind::Chest)
        })
        .ok_or_else(|| "there is no chest or lock here".to_string())?;
    let locked = world
        .entities
        .get(chest)
        .and_then(|e| e.item_container_locked())
        .unwrap_or(false);
    let trapped = world
        .entities
        .get(chest)
        .and_then(|e| e.item_container_trapped())
        .unwrap_or(false);

    if locked {
        let entities = &mut world.state_mut().entities;
        entities.set_item_container_locked(chest, false);
        if kind == ItemKind::LockPick && buc != BucStatus::Cursed && trapped {
            entities.set_item_container_trapped(chest, false);
        }
        Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You unlock the chest.".to_string(),
        }])
    } else if trapped && kind == ItemKind::LockPick && buc != BucStatus::Cursed {
        world
            .state_mut()
            .entities
            .set_item_container_trapped(chest, false);
        Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You disarm the trap on the chest.".to_string(),
        }])
    } else if kind == ItemKind::SkeletonKey {
        world
            .state_mut()
            .entities
            .set_item_container_locked(chest, true);
        Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You lock the chest.".to_string(),
        }])
    } else {
        Err("there is nothing locked here to unlock".to_string())
    }
}

fn apply_dig(
    world: &mut GameWorld,
    rng: &mut GameRng,
    kind: ItemKind,
    item: EntityId,
    buc: BucStatus,
    direction: Option<Direction>,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("digging is only available in campaign mode".to_string());
    }
    let dir = direction.ok_or_else(|| "choose a direction to dig".to_string())?;
    let pos = world.player_pos().offset(dir.delta());
    let (width, height, tile) = {
        let map = world.current_map();
        (map.width, map.height, map.tile(pos))
    };
    if pos.x <= 0 || pos.x >= width - 1 || pos.y <= 0 || pos.y >= height - 1 {
        return Err("the outer dungeon wall is too hard to dig".to_string());
    }
    if tile.ok() != Some(TileKind::Wall) {
        return Err("there is no wall in that direction".to_string());
    }
    if kind == ItemKind::PickAxe && buc == BucStatus::Cursed && rng.next_u64().is_multiple_of(2) {
        world.state_mut().entities.set_item_buc_known(item, true);
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Warning,
            text: "The cursed pick-axe slips in your hands!".to_string(),
        }]);
    }
    world
        .current_map_mut()
        .set_tile(pos, TileKind::Floor)
        .map_err(|e| e.to_string())?;
    world.mark_current_level_explored();
    Ok(vec![
        GameEvent::TileRevealed {
            pos,
            tile: TileKind::Floor,
        },
        GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You dig through the rock.".to_string(),
        },
    ])
}

fn apply_whistle(world: &mut GameWorld) -> Result<Vec<GameEvent>, String> {
    let level = world.current_level();
    let player_pos = world.player_pos();
    let pets: Vec<(EntityId, Pos)> = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            let (_, faction, actor_level, actor_pos, _, alive) = entity.actor()?;
            (alive && faction == Faction::Pet && actor_level == level)
                .then_some((entity.id, actor_pos))
        })
        .collect();
    let mut events = vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You blow a shrill note on the tin whistle.".to_string(),
    }];
    for (pet_id, pet_pos) in pets {
        let current_dist = pet_pos.chebyshev_distance(player_pos);
        if current_dist <= 1 {
            continue;
        }
        let best_dir = Direction::ALL.into_iter().find(|dir| {
            let next = pet_pos.offset(dir.delta());
            next.chebyshev_distance(player_pos) < current_dist
                && movement::is_passable_for_actor(world, pet_id, *dir)
        });
        if let Some(dir) = best_dir {
            if movement::move_actor(world, pet_id, dir).is_ok() {
                if let Some((_, to)) = world.entities.actor_location(pet_id) {
                    events.push(GameEvent::EntityMoved {
                        entity: pet_id,
                        from: pet_pos,
                        to,
                    });
                }
            }
        }
    }
    Ok(events)
}

fn apply_lamp(
    world: &mut GameWorld,
    item: EntityId,
    active: bool,
    charges: Option<u8>,
) -> Result<Vec<GameEvent>, String> {
    if active {
        world.state_mut().entities.set_item_tool_active(item, false);
        world.mark_current_level_explored();
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You extinguish your oil lamp.".to_string(),
        }]);
    }
    if charges.unwrap_or(0) == 0 {
        return Err("The oil lamp is out of fuel.".to_string());
    }
    world.state_mut().entities.set_item_tool_active(item, true);
    world.mark_current_level_explored();
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You light your oil lamp.".to_string(),
    }])
}

fn apply_blindfold(
    world: &mut GameWorld,
    item: EntityId,
    active: bool,
    buc: BucStatus,
) -> Result<Vec<GameEvent>, String> {
    if active {
        if buc == BucStatus::Cursed {
            world.state_mut().entities.set_item_buc_known(item, true);
            return Err("The blindfold is cursed and stuck to your face!".to_string());
        }
        world.state_mut().entities.set_item_tool_active(item, false);
        world.mark_current_level_explored();
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "You remove your blindfold.".to_string(),
        }]);
    }
    if buc == BucStatus::Cursed {
        world.state_mut().entities.set_item_buc_known(item, true);
    }
    world.state_mut().entities.set_item_tool_active(item, true);
    world.mark_current_level_explored();
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You tie the blindfold over your eyes.".to_string(),
    }])
}

fn apply_unicorn_horn(
    world: &mut GameWorld,
    item: EntityId,
    buc: BucStatus,
) -> Result<Vec<GameEvent>, String> {
    if buc == BucStatus::Cursed {
        let state = world.state_mut();
        state.entities.set_item_buc_known(item, true);
        state.status_timers.confused = state.status_timers.confused.max(5);
        state.status_timers.stunned = state.status_timers.stunned.max(3);
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Warning,
            text: "Nothing good happens as the cursed horn twists in your hand!".to_string(),
        }]);
    }
    let state = world.state_mut();
    state.poison_turns = 0;
    state.paralysis_turns = 0;
    state.hallucinating = false;
    state.status_timers.blind = 0;
    state.status_timers.confused = 0;
    state.status_timers.stunned = 0;
    state.status_timers.hallucinating = 0;
    if let Some(mut campaign) = state.campaign {
        let base = campaign.role.base_attributes();
        campaign.attributes.str_ = campaign.attributes.str_.max(base.str_);
        campaign.attributes.dex = campaign.attributes.dex.max(base.dex);
        campaign.attributes.con = campaign.attributes.con.max(base.con);
        campaign.attributes.int = campaign.attributes.int.max(base.int);
        campaign.attributes.wis = campaign.attributes.wis.max(base.wis);
        campaign.attributes.cha = campaign.attributes.cha.max(base.cha);
        let new_max_hp = campaign.max_hp();
        let new_hit = campaign.hit_bonus();
        let new_dmg = campaign.damage_bonus();
        let new_max_mp = campaign.max_mp();
        state.campaign = Some(campaign);
        state.max_mp = new_max_mp;
        state.mp = state.mp.min(new_max_mp);
        if let Some(stats) = state.entities.actor_stats_mut(state.player_id) {
            stats.max_hp = new_max_hp;
            stats.hp = stats.hp.min(new_max_hp);
            stats.hit_bonus = new_hit;
            stats.damage_bonus = new_dmg;
        }
    }
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "You apply the unicorn horn and feel restored!".to_string(),
    }])
}
