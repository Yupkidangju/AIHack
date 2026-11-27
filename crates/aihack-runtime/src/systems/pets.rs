use aihack_core::{
    campaign::{Race, Role},
    domain::{
        entity::{
            ActorKind, EntityLocation, Faction, DEFAULT_PET_HUNGER, MAX_PET_HUNGER,
            MAX_PET_TAMENESS,
        },
        item::{BucStatus, ItemClass, ItemKind},
        monster::MonsterKind,
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
    rng::GameRng,
};

use crate::{
    domain::monster::try_monster_template_from_registry,
    systems::{combat, death, movement, npc, vision},
    world::GameWorld,
};

pub fn starting_pet_kind(role: Role, race: Race) -> MonsterKind {
    match role {
        Role::Knight | Role::Valkyrie | Role::Samurai => MonsterKind::Pony,
        Role::Mage | Role::Wizard | Role::Rogue | Role::Archeologist => MonsterKind::Kitten,
        _ if race == Race::Elf => MonsterKind::Kitten,
        _ => MonsterKind::LittleDog,
    }
}

pub fn init_pet_state(
    entities: &mut crate::domain::entity::EntityStore,
    pet_id: EntityId,
    tameness: u8,
) {
    entities.set_actor_faction(pet_id, Faction::Pet);
    entities.set_actor_tameness(pet_id, tameness);
    entities.set_actor_pet_hunger(pet_id, DEFAULT_PET_HUNGER);
    entities.set_actor_pet_xp(pet_id, 0);
}

pub fn try_feed_or_tame_by_throw(
    world: &mut GameWorld,
    target: EntityId,
    thrown_item: EntityId,
) -> Option<Vec<GameEvent>> {
    world.campaign?;
    let item_entity = world.entities.get(thrown_item)?;
    let (_, data, _, _, _) = item_entity.item()?;
    if !matches!(data.class, ItemClass::Food | ItemClass::Corpse) {
        return None;
    }
    let target_entity = world.entities.get(target)?;
    let (ActorKind::Monster(mk), faction, _, _, _, alive) = target_entity.actor()? else {
        return None;
    };
    if !alive || !mk.is_pet_capable() {
        return None;
    }
    if mk.is_herbivore_pet() && data.class == ItemClass::Corpse {
        return None;
    }
    let monster_name = aihack_content::monster_display_name(mk, world.content_registry());
    let cur_tameness = target_entity.actor_tameness().unwrap_or(0);
    let cur_hunger = target_entity.actor_pet_hunger().unwrap_or(0);
    let next_tameness = cur_tameness.saturating_add(3).clamp(3, MAX_PET_TAMENESS);
    let next_hunger = cur_hunger.saturating_add(300).clamp(300, MAX_PET_HUNGER);
    let entities = &mut world.state_mut().entities;
    entities.set_item_location(thrown_item, EntityLocation::Consumed);
    entities.set_item_unpaid(thrown_item, false);
    entities.set_actor_faction(target, Faction::Pet);
    entities.set_actor_tameness(target, next_tameness);
    entities.set_actor_pet_hunger(target, next_hunger);
    let text = if faction == Faction::Pet {
        format!("Your {monster_name} eagerly eats the food!")
    } else {
        format!("The {monster_name} eagerly eats the food and becomes tame!")
    };
    Some(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text,
    }])
}

pub fn award_pet_kill_xp(
    world: &mut GameWorld,
    pet_id: EntityId,
    defeated_difficulty: u16,
) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let Some(pet) = world.entities.get(pet_id) else {
        return Vec::new();
    };
    let Some((ActorKind::Monster(mk), Faction::Pet, _, _, stats, true)) = pet.actor() else {
        return Vec::new();
    };
    let xp_gain = u32::from(defeated_difficulty.max(1)).saturating_mul(10);
    let next_xp = pet.actor_pet_xp().unwrap_or(0).saturating_add(xp_gain);
    let next_hunger = pet.actor_pet_hunger().unwrap_or(0).saturating_add(60);
    let old_hp = stats.hp;
    let old_max_hp = stats.max_hp;
    let old_ac = stats.ac;
    let old_hit = stats.hit_bonus;
    let mut events = Vec::new();
    {
        let entities = &mut world.state_mut().entities;
        entities.set_actor_pet_xp(pet_id, next_xp);
        entities.set_actor_pet_hunger(pet_id, next_hunger);
        if let Some(s) = entities.actor_stats_mut(pet_id) {
            s.hp = old_hp.saturating_add(2).min(old_max_hp);
        }
    }
    if next_xp >= 20 {
        if let Some(evolved) = mk.evolved_form() {
            if let Ok(tmpl) = try_monster_template_from_registry(evolved, world.content_registry())
            {
                let old_name = aihack_content::monster_display_name(mk, world.content_registry());
                let new_name =
                    aihack_content::monster_display_name(evolved, world.content_registry());
                let entities = &mut world.state_mut().entities;
                entities.set_actor_kind(pet_id, ActorKind::Monster(evolved));
                if let Some(s) = entities.actor_stats_mut(pet_id) {
                    s.max_hp = tmpl.hp.max(old_max_hp.saturating_add(4));
                    s.hp = s.max_hp;
                    s.ac = tmpl.ac.min(old_ac);
                    s.hit_bonus = tmpl.hit_bonus.max(old_hit);
                    s.damage = tmpl.attack_profile.damage;
                    s.speed = tmpl.speed;
                    s.difficulty = tmpl.difficulty;
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: format!("Your {old_name} grows into a {new_name}!"),
                });
            }
        }
    }
    events
}

pub fn tick_pets_hunger(world: &mut GameWorld, turn: u64) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let level = world.current_level();
    let pets: Vec<(EntityId, MonsterKind, u8, i16)> = world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(mk), Faction::Pet, l, _, _, true) = e.actor()? else {
                return None;
            };
            (l == level).then_some((
                e.id,
                mk,
                e.actor_tameness().unwrap_or(1),
                e.actor_pet_hunger().unwrap_or(0),
            ))
        })
        .collect();
    let mut events = Vec::new();
    for (pet_id, mk, tameness, hunger) in pets {
        let next_hunger = hunger.saturating_sub(1).max(0);
        world
            .state_mut()
            .entities
            .set_actor_pet_hunger(pet_id, next_hunger);
        if next_hunger == 0 && turn > 0 && turn.is_multiple_of(20) {
            let pet_name = aihack_content::monster_display_name(mk, world.content_registry());
            let next_tame = tameness.saturating_sub(1);
            let entities = &mut world.state_mut().entities;
            entities.set_actor_tameness(pet_id, next_tame);
            if next_tame == 0 {
                entities.set_actor_faction(pet_id, Faction::Hostile);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("Your {pet_name} goes wild from hunger!"),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("Your {pet_name} looks very hungry!"),
                });
            }
        }
    }
    events
}

pub fn follow_player_on_level_change(
    world: &mut GameWorld,
    from: LevelId,
    from_player_pos: Pos,
    to: LevelId,
    to_player_pos: Pos,
) -> Vec<GameEvent> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let followers: Vec<EntityId> = world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(_), Faction::Pet, level, pos, _, true) = e.actor()? else {
                return None;
            };
            (level == from && pos.chebyshev_distance(from_player_pos) <= 2).then_some(e.id)
        })
        .collect();
    let mut events = Vec::new();
    for pet_id in followers {
        if let Some(landing) = find_pet_landing(world, to, to_player_pos) {
            world
                .state_mut()
                .entities
                .set_actor_location(pet_id, to, landing);
            events.push(GameEvent::LevelChanged {
                entity: pet_id,
                from,
                to,
            });
        }
    }
    events
}

fn find_pet_landing(world: &GameWorld, level: LevelId, center: Pos) -> Option<Pos> {
    let map = world.levels.map(level)?;
    for radius in 1..=3_i16 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let candidate = Pos {
                    x: center.x.saturating_add(dx),
                    y: center.y.saturating_add(dy),
                };
                if !map.contains(candidate) {
                    continue;
                }
                if matches!(
                    map.tile(candidate),
                    Ok(TileKind::Floor
                        | TileKind::StairsUp
                        | TileKind::StairsDown
                        | TileKind::Shop
                        | TileKind::Altar)
                ) && world.entities.alive_actor_at(level, candidate).is_none()
                {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

pub fn is_edible_for_pet(world: &GameWorld, pet_kind: MonsterKind, item_id: EntityId) -> bool {
    let Some(entity) = world.entities.get(item_id) else {
        return false;
    };
    let Some((kind, data, _, _, _)) = entity.item() else {
        return false;
    };
    if entity.item_unpaid() == Some(true) || entity.item_buc() == Some(BucStatus::Cursed) {
        return false;
    }
    if kind == ItemKind::FoodRation {
        return true;
    }
    if data.class == ItemClass::Corpse && !pet_kind.is_herbivore_pet() {
        let fresh = entity.item_corpse_age().unwrap_or(0) <= 40;
        let poisonous = entity.item_corpse_monster().is_some_and(|cm| {
            aihack_content::monster_corpse_spec(cm, world.content_registry())
                .effects
                .iter()
                .any(|e| e == "poisonous")
        });
        return fresh && !poisonous;
    }
    false
}

pub fn step_pet_ai(
    world: &mut GameWorld,
    rng: &mut GameRng,
    pet_id: EntityId,
) -> Result<Vec<GameEvent>, String> {
    let mut events = Vec::new();
    let Some(pet_entity) = world.entities.get(pet_id) else {
        return Ok(events);
    };
    let Some((ActorKind::Monster(pet_kind), Faction::Pet, level, pet_pos, _, true)) =
        pet_entity.actor()
    else {
        return Ok(events);
    };
    let pet_hunger = pet_entity.actor_pet_hunger().unwrap_or(0);
    let pet_tameness = pet_entity.actor_tameness().unwrap_or(1);
    let carried: Vec<EntityId> = pet_entity
        .actor_monster_inventory()
        .unwrap_or_default()
        .to_vec();
    let pet_name = aihack_content::monster_display_name(pet_kind, world.content_registry());

    // 1. Engage adjacent hostile monster
    let adjacent_hostile = world.entities.entities().iter().find_map(|e| {
        let (ActorKind::Monster(_), Faction::Hostile, l, pos, _, true) = e.actor()? else {
            return None;
        };
        (l == level && pos.chebyshev_distance(pet_pos) <= 1).then_some(e.id)
    });
    if let Some(target_id) = adjacent_hostile {
        let difficulty = world
            .entities
            .get(target_id)
            .and_then(|e| e.monster_difficulty())
            .unwrap_or(1);
        if let Some(resolution) = combat::resolve_attack(world, rng, pet_id, target_id) {
            events.push(combat::attack_event(&resolution));
            let deaths = death::collect_death_events_after_attack(world, pet_id, target_id)?;
            let target_died = deaths
                .iter()
                .any(|e| matches!(e, GameEvent::EntityDied { entity, .. } if *entity == target_id));
            events.extend(deaths);
            if target_died {
                events.extend(award_pet_kill_xp(world, pet_id, difficulty));
            }
        }
        return Ok(events);
    }

    // 2. Eat edible floor food if hungry
    if pet_hunger < 500 {
        if let Some(food_id) = world
            .entities
            .items_at(level, pet_pos)
            .into_iter()
            .find(|&id| is_edible_for_pet(world, pet_kind, id))
        {
            let entities = &mut world.state_mut().entities;
            entities.set_item_location(food_id, EntityLocation::Consumed);
            entities.set_actor_pet_hunger(pet_id, pet_hunger.saturating_add(300));
            entities.set_actor_tameness(pet_id, pet_tameness.saturating_add(2));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("Your {pet_name} eats the food on the floor."),
            });
            return Ok(events);
        }
    }

    // 3. Pet shop stealing: carry unpaid shop item outside shop and drop for player
    let player_pos = world.player_pos();
    if let Some(&stolen_id) = carried.first() {
        if !npc::is_in_shop_zone(world, pet_pos) && pet_pos.chebyshev_distance(player_pos) <= 2 {
            let item_label = world
                .entities
                .item_data(stolen_id)
                .map(|d| aihack_content::item_display_name(d.kind, world.content_registry()))
                .unwrap_or_else(|| "item".to_string());
            let entities = &mut world.state_mut().entities;
            entities.remove_monster_inventory(pet_id, stolen_id);
            entities.set_item_unpaid(stolen_id, false);
            entities.set_item_location(
                stolen_id,
                EntityLocation::OnMap {
                    level,
                    pos: pet_pos,
                },
            );
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("Your {pet_name} drops a {item_label} at your feet!"),
            });
            return Ok(events);
        }
    } else if npc::is_in_shop_zone(world, pet_pos) {
        if let Some(unpaid_id) = world
            .entities
            .items_at(level, pet_pos)
            .into_iter()
            .find(|&id| {
                world.entities.get(id).is_some_and(|e| {
                    e.item_unpaid() == Some(true)
                        && e.item_buc() != Some(BucStatus::Cursed)
                        && e.item().is_some_and(|(k, d, _, _, _)| {
                            k != ItemKind::AmuletAscension && d.container_capacity.is_none()
                        })
                })
            })
        {
            let item_label = world
                .entities
                .item_data(unpaid_id)
                .map(|d| aihack_content::item_display_name(d.kind, world.content_registry()))
                .unwrap_or_else(|| "item".to_string());
            let entities = &mut world.state_mut().entities;
            entities.set_item_location(unpaid_id, EntityLocation::Inventory { owner: pet_id });
            entities.push_monster_inventory(pet_id, unpaid_id);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("Your {pet_name} picks up a {item_label} from the shop!"),
            });
            return Ok(events);
        }
    }

    // 4. Choose target: nearby food if hungry, nearby visible hostile, or player
    let target_pos = if !carried.is_empty() {
        player_pos
    } else if pet_hunger < 500 {
        find_nearby_pet_food(world, level, pet_pos, pet_kind)
            .or_else(|| find_nearby_hostile(world, level, pet_pos))
            .unwrap_or(player_pos)
    } else {
        find_nearby_hostile(world, level, pet_pos).unwrap_or(player_pos)
    };

    if target_pos == player_pos && pet_pos.chebyshev_distance(player_pos) <= 1 && carried.is_empty()
    {
        return Ok(events);
    }

    // Step toward target while avoiding cursed floor items
    let cur_dist = (
        pet_pos.chebyshev_distance(target_pos),
        manhattan(pet_pos, target_pos),
    );
    let mut best: Option<(Direction, (i16, i16))> = None;
    let mut warned_cursed = false;
    for dir in Direction::ALL {
        if !movement::is_passable_for_actor(world, pet_id, dir) {
            continue;
        }
        let next = pet_pos.offset(dir.delta());
        let next_dist = (
            next.chebyshev_distance(target_pos),
            manhattan(next, target_pos),
        );
        if next_dist >= cur_dist && carried.is_empty() {
            continue;
        }
        if let Some(cursed_item) = world
            .entities
            .items_at(level, next)
            .into_iter()
            .find(|&id| {
                world
                    .entities
                    .get(id)
                    .is_some_and(|e| e.item_buc() == Some(BucStatus::Cursed))
            })
        {
            world
                .state_mut()
                .entities
                .set_item_buc_known(cursed_item, true);
            if !warned_cursed {
                warned_cursed = true;
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: format!("Your {pet_name} moves reluctantly away from the cursed item."),
                });
            }
            continue;
        }
        if best.is_none_or(|(_, d)| next_dist < d) {
            best = Some((dir, next_dist));
        }
    }
    if let Some((dir, _)) = best {
        if movement::move_actor(world, pet_id, dir).is_ok() {
            let to = world
                .entities
                .actor_location(pet_id)
                .map(|(_, p)| p)
                .unwrap_or(pet_pos);
            events.push(GameEvent::EntityMoved {
                entity: pet_id,
                from: pet_pos,
                to,
            });
        }
    }
    Ok(events)
}

fn find_nearby_pet_food(
    world: &GameWorld,
    level: LevelId,
    origin: Pos,
    pet_kind: MonsterKind,
) -> Option<Pos> {
    world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (_, _, EntityLocation::OnMap { level: l, pos }, _, _) = e.item()? else {
                return None;
            };
            if l != level || origin.chebyshev_distance(pos) > 5 {
                return None;
            }
            is_edible_for_pet(world, pet_kind, e.id)
                .then_some((pos, origin.chebyshev_distance(pos)))
        })
        .min_by_key(|&(_, d)| d)
        .map(|(p, _)| p)
}

fn find_nearby_hostile(world: &GameWorld, level: LevelId, origin: Pos) -> Option<Pos> {
    world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(_), Faction::Hostile, l, pos, _, true) = e.actor()? else {
                return None;
            };
            if l != level || origin.chebyshev_distance(pos) > 5 {
                return None;
            }
            vision::has_line_of_sight(world, origin, pos)
                .then_some((pos, origin.chebyshev_distance(pos)))
        })
        .min_by_key(|&(_, d)| d)
        .map(|(p, _)| p)
}

fn manhattan(a: Pos, b: Pos) -> i16 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
