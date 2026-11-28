use aihack_core::{
    domain::{
        combat::{AttackProfile, DamageRoll},
        entity::{ActorKind, EntityLocation, Faction},
        item::{BucStatus, ItemClass, ItemKind, WandEffect},
        monster::{MonsterAiKind, MonsterKind, MonsterPassive},
        status::Resistances,
        tile::{DoorState, TileKind},
    },
    event::{GameEvent, MessagePriority},
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
    rng::GameRng,
    run_state::RunState,
};

use crate::{
    domain::monster::try_monster_template_from_registry,
    systems::{combat, death, items, movement, pets, tools, vision},
    world::GameWorld,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterIntent {
    Wait,
    Move {
        entity: EntityId,
        direction: Direction,
    },
    MeleeAttack {
        attacker: EntityId,
        defender: EntityId,
    },
    RangedAttack {
        attacker: EntityId,
        defender: EntityId,
    },
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MonsterTurnPlan {
    pub intents: Vec<MonsterIntent>,
}

pub const ENERGY_THRESHOLD: i16 = 12;
pub const MAX_MONSTER_ACTIONS_PER_TURN: usize = 4;
const MAX_MONSTER_INVENTORY: usize = 4;

pub fn run_monster_turn(
    world: &mut GameWorld,
    rng: &mut GameRng,
    state: &mut RunState,
    turn: u64,
) -> Result<Vec<GameEvent>, String> {
    let mut events = Vec::new();
    let in_campaign = world.campaign.is_some();
    let actors = if in_campaign {
        world
            .entities
            .non_player_actors_on_level(world.current_level())
    } else {
        world.current_level_hostile_monsters()
    };
    for actor in actors {
        if !world.player_alive() {
            break;
        }
        let Some(entity) = world.entities.get(actor) else {
            continue;
        };
        if !entity.is_alive_actor() {
            continue;
        }
        let Some(stats) = world.entities.actor_stats(actor) else {
            continue;
        };
        let speed = stats.speed;
        let effective_gain = if world.is_fast() && speed > 0 {
            (speed / 2).max(1)
        } else {
            speed
        };
        let next_energy = stats.energy.saturating_add(effective_gain).clamp(-36, 72);
        if let Some(stats_mut) = world.state_mut().entities.actor_stats_mut(actor) {
            stats_mut.energy = next_energy;
        }
        for _ in 0..MAX_MONSTER_ACTIONS_PER_TURN {
            if !world.player_alive() {
                break;
            }
            let Some(entity) = world.entities.get(actor) else {
                break;
            };
            if !entity.is_alive_actor() {
                break;
            }
            let current_energy = world
                .entities
                .actor_stats(actor)
                .map(|s| s.energy)
                .unwrap_or(0);
            if current_energy < ENERGY_THRESHOLD {
                break;
            }
            let spent = (current_energy - ENERGY_THRESHOLD).clamp(-36, 72);
            if let Some(stats_mut) = world.state_mut().entities.actor_stats_mut(actor) {
                stats_mut.energy = spent;
            }
            if in_campaign {
                let acted = step_campaign_actor(world, rng, actor, &mut events)?;
                if !acted {
                    let clamped = spent.min(ENERGY_THRESHOLD.saturating_sub(effective_gain).max(0));
                    if let Some(stats_mut) = world.state_mut().entities.actor_stats_mut(actor) {
                        stats_mut.energy = clamped;
                    }
                    break;
                }
            } else {
                let intent = decide_monster_action(world, rng, actor);
                if matches!(intent, MonsterIntent::Wait) {
                    let clamped = spent.min(ENERGY_THRESHOLD.saturating_sub(effective_gain).max(0));
                    if let Some(stats_mut) = world.state_mut().entities.actor_stats_mut(actor) {
                        stats_mut.energy = clamped;
                    }
                    break;
                }
                apply_monster_intent(world, rng, intent, &mut events)?;
            }
        }
    }
    regenerate_monsters(world, &mut events);
    *state = death::state_after_deaths_at(world, turn);
    Ok(events)
}

#[cfg(test)]
pub fn collect_monster_turn_at(world: &GameWorld, rng: &mut GameRng, turn: u64) -> MonsterTurnPlan {
    MonsterTurnPlan {
        intents: world
            .current_level_hostile_monsters()
            .into_iter()
            .map(|actor| decide_monster_intent(world, rng, actor, turn))
            .collect(),
    }
}

pub fn try_split_on_hit(
    world: &mut GameWorld,
    defender: EntityId,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Ok(Vec::new());
    }
    let Some(entity) = world.entities.get(defender) else {
        return Ok(Vec::new());
    };
    let Some((ActorKind::Monster(mk), faction, level, pos, stats, true)) = entity.actor() else {
        return Ok(Vec::new());
    };
    if stats.hp < 6 {
        return Ok(Vec::new());
    }
    let split_pos = Direction::ALL.into_iter().find_map(|dir| {
        let candidate = pos.offset(dir.delta());
        let ok_tile = world
            .map(level)
            .tile(candidate)
            .is_ok_and(|t| matches!(t, TileKind::Floor | TileKind::Shop | TileKind::Altar));
        (ok_tile && world.entities.alive_actor_at(level, candidate).is_none()).then_some(candidate)
    });
    let Some(clone_pos) = split_pos else {
        return Ok(Vec::new());
    };
    let hp_a = stats.hp / 2;
    let hp_b = stats.hp - hp_a;
    let max_hp = stats.max_hp;
    let template = try_monster_template_from_registry(mk, world.content_registry())
        .map_err(|error| error.to_string())?;
    let name = aihack_content::monster_display_name(mk, world.content_registry());
    let clone_id = world
        .state_mut()
        .entities
        .spawn_monster_with_faction(mk, template, faction, clone_pos)
        .map_err(|error| error.to_string())?;
    let entities = &mut world.state_mut().entities;
    entities.set_actor_location(clone_id, level, clone_pos);
    if let Some(orig) = entities.actor_stats_mut(defender) {
        orig.hp = hp_a.max(1);
    }
    if let Some(clone_stats) = entities.actor_stats_mut(clone_id) {
        clone_stats.max_hp = max_hp;
        clone_stats.hp = hp_b.max(1);
    }
    Ok(vec![GameEvent::Message {
        priority: MessagePriority::Warning,
        text: format!("The {name} splits in two!"),
    }])
}

fn step_campaign_actor(
    world: &mut GameWorld,
    rng: &mut GameRng,
    actor: EntityId,
    events: &mut Vec<GameEvent>,
) -> Result<bool, String> {
    let Some(entity) = world.entities.get(actor) else {
        return Ok(false);
    };
    let Some((ActorKind::Monster(mk), faction, level, actor_pos, stats, true)) = entity.actor()
    else {
        return Ok(false);
    };
    let ai_kind = entity
        .monster_ai_kind()
        .unwrap_or(MonsterAiKind::Stationary);
    let passive = entity.monster_passive();
    let hp = stats.hp;
    let max_hp = stats.max_hp;
    let carried: Vec<EntityId> = entity
        .actor_monster_inventory()
        .unwrap_or_default()
        .to_vec();

    match faction {
        Faction::Pet => {
            let before = events.len();
            let pet_events = pets::step_pet_ai(world, rng, actor)?;
            let acted = !pet_events.is_empty();
            events.extend(pet_events);
            return Ok(acted || events.len() > before);
        }
        Faction::Shopkeeper => {
            // Peaceful shopkeeper stays in shop until provoked.
            return Ok(false);
        }
        Faction::Peaceful => {
            if aihack_content::monster_has_ai_flag(mk, "watch", world.content_registry()) {
                return step_watchman_ai(world, rng, actor, mk, level, actor_pos, events);
            }
            return Ok(false);
        }
        Faction::Player => return Ok(false),
        Faction::Hostile | Faction::Neutral => {}
    }

    if ai_kind == MonsterAiKind::Stationary {
        return Ok(false);
    }

    let registry = world.content_registry();
    let uses_items = aihack_content::monster_has_ai_flag(mk, "uses_items", registry);
    let picks_up = aihack_content::monster_has_ai_flag(mk, "picks_up", registry);
    let opens_doors = aihack_content::monster_has_ai_flag(mk, "opens_doors", registry);

    // 1. Drink healing potion when wounded
    if uses_items && hp.saturating_mul(2) <= max_hp {
        if let Some(potion_id) = carried.iter().copied().find(|&id| {
            world.entities.item_data(id).is_some_and(|d| {
                matches!(
                    d.kind,
                    ItemKind::PotionHealing | ItemKind::PotionExtraHealing | ItemKind::PotionCure
                )
            })
        }) {
            let potion_kind = world
                .entities
                .item_data(potion_id)
                .map(|d| d.kind)
                .unwrap_or(ItemKind::PotionHealing);
            let heal = match potion_kind {
                ItemKind::PotionExtraHealing => {
                    combat::roll_die(rng, 8) + combat::roll_die(rng, 8) + 8
                }
                _ => combat::roll_die(rng, 8) + 4,
            };
            let name = aihack_content::monster_display_name(mk, world.content_registry());
            let entities = &mut world.state_mut().entities;
            entities.remove_monster_inventory(actor, potion_id);
            entities.set_item_location(potion_id, EntityLocation::Consumed);
            if let Some(s) = entities.actor_stats_mut(actor) {
                let next = s.hp.saturating_add(heal).min(s.max_hp);
                let gained = next - s.hp;
                s.hp = next;
                events.push(GameEvent::EntityHealed {
                    entity: actor,
                    amount: gained,
                    hp_after: next,
                });
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: format!("The {name} quaffs a healing potion!"),
            });
            return Ok(true);
        }
    }

    // 2. Pick up useful floor gear into monster_inventory
    if picks_up && carried.len() < MAX_MONSTER_INVENTORY {
        if let Some((item_id, item_kind, ac_bonus)) = world
            .entities
            .items_at(level, actor_pos)
            .into_iter()
            .find_map(|id| {
                let item_e = world.entities.get(id)?;
                if item_e.item_unpaid() == Some(true)
                    || item_e.item_buc() == Some(BucStatus::Cursed)
                {
                    return None;
                }
                let (kind, data, _, _, charges) = item_e.item()?;
                if kind == ItemKind::AmuletAscension || data.container_capacity.is_some() {
                    return None;
                }
                let useful = match data.class {
                    ItemClass::Weapon | ItemClass::Armor => true,
                    ItemClass::Potion => matches!(
                        kind,
                        ItemKind::PotionHealing
                            | ItemKind::PotionExtraHealing
                            | ItemKind::PotionCure
                    ),
                    ItemClass::Wand => charges.unwrap_or(0) > 0,
                    _ => false,
                };
                useful.then_some((id, kind, data.ac_bonus))
            })
        {
            let monster_name = aihack_content::monster_display_name(mk, world.content_registry());
            let item_name = aihack_content::item_display_name(item_kind, world.content_registry());
            let entities = &mut world.state_mut().entities;
            entities.set_item_location(item_id, EntityLocation::Inventory { owner: actor });
            entities.push_monster_inventory(actor, item_id);
            if ac_bonus > 0 {
                if let Some(s) = entities.actor_stats_mut(actor) {
                    s.ac = s.ac.saturating_sub(ac_bonus.min(4)).max(-10);
                }
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("The {monster_name} picks up a {item_name}!"),
            });
            return Ok(true);
        }
    }

    // 3. Summon allies (OrcCaptain)
    if passive == Some(MonsterPassive::SummonAllies)
        && vision::monster_has_line_of_sight_to_player(world, actor)
    {
        let alive_orcs = world
            .entities
            .entities()
            .iter()
            .filter(|e| {
                matches!(
                    e.actor(),
                    Some((ActorKind::Monster(MonsterKind::Orc), Faction::Hostile, l, _, _, true))
                        if l == level
                )
            })
            .count();
        if alive_orcs < 4 && rng.next_u64().is_multiple_of(3) {
            if let Some(spawn_pos) = Direction::ALL.into_iter().find_map(|dir| {
                let candidate = actor_pos.offset(dir.delta());
                let ok_tile = world
                    .map(level)
                    .tile(candidate)
                    .is_ok_and(|t| matches!(t, TileKind::Floor | TileKind::Shop | TileKind::Altar));
                (ok_tile && world.entities.alive_actor_at(level, candidate).is_none())
                    .then_some(candidate)
            }) {
                if let Ok(tmpl) =
                    try_monster_template_from_registry(MonsterKind::Orc, world.content_registry())
                {
                    let orc_id = world
                        .state_mut()
                        .entities
                        .spawn_monster_with_faction(
                            MonsterKind::Orc,
                            tmpl,
                            Faction::Hostile,
                            spawn_pos,
                        )
                        .map_err(|error| error.to_string())?;
                    world
                        .state_mut()
                        .entities
                        .set_actor_location(orc_id, level, spawn_pos);
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Warning,
                        text: "The orc captain blows a war horn and summons an orc!".to_string(),
                    });
                    return Ok(true);
                }
            }
        }
    }

    // 4. Target selection: conflict, faction rival, adjacent pet/watchman, or player
    let player_id = world.player_id;
    let player_pos = world.player_pos();
    let conflict_target = if world.has_conflict() {
        find_conflict_target(world, actor, level, actor_pos)
    } else {
        None
    };
    let rival_target = find_faction_rival(world, actor, mk, level, actor_pos);
    let adjacent_ally_target = find_adjacent_pet_or_watch(world, level, actor_pos);
    let player_dist = actor_pos.chebyshev_distance(player_pos);
    let player_visible =
        player_dist <= 1 || vision::monster_has_line_of_sight_to_player(world, actor);

    let (target_id, target_pos, targeting_player) = if let Some((cid, cpos)) = conflict_target {
        (cid, cpos, false)
    } else if let Some((rid, rpos)) = rival_target {
        if !player_visible || actor_pos.chebyshev_distance(rpos) <= player_dist {
            (rid, rpos, false)
        } else {
            (player_id, player_pos, true)
        }
    } else if player_dist <= 1 {
        (player_id, player_pos, true)
    } else if let Some((aid, apos)) = adjacent_ally_target {
        (aid, apos, false)
    } else {
        (player_id, player_pos, true)
    };

    // 5. Elbereth ward check when targeting player
    if targeting_player
        && crate::systems::engrave::has_active_elbereth(world, player_pos)
        && mk != MonsterKind::Minotaur
        && player_visible
    {
        let flee = choose_flee_intent(world, actor, actor_pos, player_pos);
        if matches!(flee, MonsterIntent::Wait) {
            return Ok(false);
        }
        apply_monster_intent(world, rng, flee, events)?;
        return Ok(true);
    }

    // 6. Adjacent melee attack
    if actor_pos.chebyshev_distance(target_pos) <= 1 {
        apply_monster_intent(
            world,
            rng,
            MonsterIntent::MeleeAttack {
                attacker: actor,
                defender: target_id,
            },
            events,
        )?;
        return Ok(true);
    }

    // 7. Medusa gaze attack (blocked by Reflection, Blind, or Blindfold; avoids infinite stunlock)
    if passive == Some(MonsterPassive::GazeParalyze)
        && targeting_player
        && player_visible
        && player_dist <= 5
    {
        if world.has_reflection() {
            if let Some(mstats) = world.state_mut().entities.actor_stats_mut(actor) {
                mstats.hp = 0;
            }
            events.push(GameEvent::PassiveAttackTriggered {
                source: actor,
                target: player_id,
            });
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Medusa sees her reflection and turns to stone!".to_string(),
            });
            events.extend(death::collect_death_events_after_attack(
                world, player_id, actor,
            )?);
            return Ok(true);
        }
        if world.status_timers.blind > 0 || tools::is_player_blindfolded(world) {
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Your blindness shields you from Medusa's petrifying gaze!".to_string(),
            });
            return Ok(true);
        }
        let gaze_dmg = combat::roll_die(rng, 6);
        {
            let state = world.state_mut();
            if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                stats.hp = stats.hp.saturating_sub(gaze_dmg);
            }
            if state.paralysis_turns == 0 {
                state.paralysis_turns = 2;
            }
            state.status_timers.stunned = state.status_timers.stunned.max(2);
        }
        events.push(GameEvent::PassiveAttackTriggered {
            source: actor,
            target: player_id,
        });
        events.push(GameEvent::Message {
            priority: MessagePriority::Warning,
            text: format!("Medusa's petrifying gaze strikes you for {gaze_dmg} damage!"),
        });
        events.extend(death::collect_death_events_after_attack(
            world, actor, player_id,
        )?);
        return Ok(true);
    }

    // 8. Monster zapping carried attack wand in straight clear line
    if uses_items && ranged_shot_has_clear_line(world, actor_pos, target_pos) {
        if let Some((wand_id, wand_effect, charges)) = carried.iter().copied().find_map(|id| {
            let e = world.entities.get(id)?;
            let (_, data, _, _, ch) = e.item()?;
            let eff = data.wand_effect?;
            if !matches!(
                eff,
                WandEffect::MagicMissile
                    | WandEffect::Fire
                    | WandEffect::Cold
                    | WandEffect::Lightning
                    | WandEffect::Death
                    | WandEffect::Striking
            ) {
                return None;
            }
            let count = ch?;
            (count > 0).then_some((id, eff, count))
        }) {
            let monster_name = aihack_content::monster_display_name(mk, world.content_registry());
            world
                .state_mut()
                .entities
                .set_item_charges(wand_id, Some(charges - 1));
            let reflected = target_id == player_id && world.has_reflection();
            let actual_target = if reflected { actor } else { target_id };
            if reflected {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: format!(
                        "The {monster_name}'s wand ray reflects off you back at the {monster_name}!"
                    ),
                });
            }
            let target_res = if actual_target == player_id {
                world.effective_player_resistances()
            } else {
                world
                    .entities
                    .actor_stats(actual_target)
                    .map(|s| s.resistances)
                    .unwrap_or(Resistances::NONE)
            };
            if (wand_effect == WandEffect::Fire && target_res.contains(Resistances::FIRE))
                || (wand_effect == WandEffect::Cold && target_res.contains(Resistances::COLD))
                || (wand_effect == WandEffect::Lightning && target_res.contains(Resistances::SHOCK))
                || (wand_effect == WandEffect::Death
                    && target_res.contains(Resistances::MAGIC_RESISTANCE))
            {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: format!("The {monster_name} zaps a wand, but it is resisted!"),
                });
            } else {
                let (dmg, label) = match wand_effect {
                    WandEffect::Fire => (
                        combat::roll_die(rng, 6) + combat::roll_die(rng, 6),
                        "fire wand",
                    ),
                    WandEffect::Cold => (
                        combat::roll_die(rng, 6) + combat::roll_die(rng, 6),
                        "cold wand",
                    ),
                    WandEffect::Lightning => (
                        combat::roll_die(rng, 6) + combat::roll_die(rng, 6) + 2,
                        "lightning wand",
                    ),
                    WandEffect::Death => {
                        let cur_hp = world
                            .entities
                            .actor_stats(actual_target)
                            .map(|s| s.hp.max(1))
                            .unwrap_or(100);
                        (cur_hp, "wand of death")
                    }
                    WandEffect::Striking => (
                        combat::roll_die(rng, 12) + combat::roll_die(rng, 6),
                        "wand of striking",
                    ),
                    _ => (combat::roll_die(rng, 6) + 2, "wand"),
                };
                if let Some(s) = world.state_mut().entities.actor_stats_mut(actual_target) {
                    s.hp = s.hp.saturating_sub(dmg);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("The {monster_name} zaps a {label} for {dmg} damage!"),
                });
                events.extend(death::collect_death_events_after_attack(
                    world,
                    actor,
                    actual_target,
                )?);
            }
            return Ok(true);
        }
    }

    // 9. Standard ranged attack (GnollArcher, Dragon)
    if ai_kind == MonsterAiKind::RangedOnSight
        && ranged_shot_has_clear_line(world, actor_pos, target_pos)
    {
        apply_monster_intent(
            world,
            rng,
            MonsterIntent::RangedAttack {
                attacker: actor,
                defender: target_id,
            },
            events,
        )?;
        return Ok(true);
    }

    // 10. Movement (with door opening when chasing)
    if ai_kind == MonsterAiKind::Wander && targeting_player {
        let intent = choose_wander_intent(world, rng, actor);
        if matches!(intent, MonsterIntent::Wait) {
            return Ok(false);
        }
        apply_monster_intent(world, rng, intent, events)?;
        return Ok(true);
    }

    let can_see_target = if targeting_player {
        player_visible
    } else {
        vision::has_line_of_sight(world, actor_pos, target_pos)
    };
    if !can_see_target {
        let adjacent_closed_door = opens_doors
            && actor_pos.chebyshev_distance(target_pos) <= 4
            && Direction::ALL.into_iter().any(|dir| {
                world.map(level).tile(actor_pos.offset(dir.delta())).ok()
                    == Some(TileKind::Door(DoorState::Closed))
            });
        if !adjacent_closed_door {
            return Ok(false);
        }
    }
    step_chase_with_doors(world, actor, mk, actor_pos, target_pos, opens_doors, events)
}

fn step_watchman_ai(
    world: &mut GameWorld,
    rng: &mut GameRng,
    actor: EntityId,
    mk: MonsterKind,
    level: LevelId,
    actor_pos: Pos,
    events: &mut Vec<GameEvent>,
) -> Result<bool, String> {
    let target = world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(_), Faction::Hostile, l, pos, _, true) = e.actor()? else {
                return None;
            };
            if l != level {
                return None;
            }
            let dist = actor_pos.chebyshev_distance(pos);
            (dist <= 6 && (dist <= 1 || vision::has_line_of_sight(world, actor_pos, pos)))
                .then_some((e.id, pos, dist))
        })
        .min_by_key(|&(_, _, dist)| dist);
    let Some((target_id, target_pos, dist)) = target else {
        return Ok(false);
    };
    if dist <= 1 {
        apply_monster_intent(
            world,
            rng,
            MonsterIntent::MeleeAttack {
                attacker: actor,
                defender: target_id,
            },
            events,
        )?;
        return Ok(true);
    }
    step_chase_with_doors(world, actor, mk, actor_pos, target_pos, true, events)
}

fn step_chase_with_doors(
    world: &mut GameWorld,
    actor: EntityId,
    mk: MonsterKind,
    actor_pos: Pos,
    target_pos: Pos,
    opens_doors: bool,
    events: &mut Vec<GameEvent>,
) -> Result<bool, String> {
    let level = world.current_level();
    let current = (
        actor_pos.chebyshev_distance(target_pos),
        manhattan_distance(actor_pos, target_pos),
    );
    let mut closed_door_dir: Option<(Direction, Pos)> = None;
    for direction in Direction::ALL {
        let to = actor_pos.offset(direction.delta());
        let next = (
            to.chebyshev_distance(target_pos),
            manhattan_distance(to, target_pos),
        );
        if next >= current {
            continue;
        }
        if movement::is_passable_for_actor(world, actor, direction) {
            if movement::move_actor(world, actor, direction).is_ok() {
                events.push(GameEvent::EntityMoved {
                    entity: actor,
                    from: actor_pos,
                    to,
                });
                return Ok(true);
            }
        } else if opens_doors
            && closed_door_dir.is_none()
            && world.map(level).tile(to).ok() == Some(TileKind::Door(DoorState::Closed))
        {
            closed_door_dir = Some((direction, to));
        }
    }
    if let Some((_, door_pos)) = closed_door_dir {
        let _ = world
            .map_mut(level)
            .set_tile(door_pos, TileKind::Door(DoorState::Open));
        let name = aihack_content::monster_display_name(mk, world.content_registry());
        events.push(GameEvent::DoorChanged {
            pos: door_pos,
            from: DoorState::Closed,
            to: DoorState::Open,
        });
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("The {name} opens the door."),
        });
        return Ok(true);
    }
    Ok(false)
}

fn find_faction_rival(
    world: &GameWorld,
    actor: EntityId,
    mk: MonsterKind,
    level: LevelId,
    actor_pos: Pos,
) -> Option<(EntityId, Pos)> {
    let reg = world.content_registry();
    let hates_dwarves = aihack_content::monster_has_ai_flag(mk, "hates_dwarves", reg);
    let hates_orcs = aihack_content::monster_has_ai_flag(mk, "hates_orcs", reg);
    if !hates_dwarves && !hates_orcs {
        return None;
    }
    world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            if e.id == actor {
                return None;
            }
            let (ActorKind::Monster(other_mk), _, l, pos, _, true) = e.actor()? else {
                return None;
            };
            if l != level {
                return None;
            }
            let is_rival = (hates_dwarves
                && aihack_content::monster_has_ai_flag(other_mk, "dwarf", reg))
                || (hates_orcs && aihack_content::monster_has_ai_flag(other_mk, "orc", reg));
            if !is_rival {
                return None;
            }
            let dist = actor_pos.chebyshev_distance(pos);
            (dist <= 6 && (dist <= 1 || vision::has_line_of_sight(world, actor_pos, pos)))
                .then_some((e.id, pos, dist))
        })
        .min_by_key(|&(_, _, d)| d)
        .map(|(id, pos, _)| (id, pos))
}

fn find_adjacent_pet_or_watch(
    world: &GameWorld,
    level: LevelId,
    actor_pos: Pos,
) -> Option<(EntityId, Pos)> {
    world.entities.entities().iter().find_map(|e| {
        let (ActorKind::Monster(mk), faction, l, pos, _, true) = e.actor()? else {
            return None;
        };
        let valid = faction == Faction::Pet
            || (faction == Faction::Peaceful && mk == MonsterKind::Watchman);
        (valid && l == level && actor_pos.chebyshev_distance(pos) <= 1).then_some((e.id, pos))
    })
}

fn find_conflict_target(
    world: &GameWorld,
    actor: EntityId,
    level: LevelId,
    actor_pos: Pos,
) -> Option<(EntityId, Pos)> {
    world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            if e.id == actor {
                return None;
            }
            let (ActorKind::Monster(_), _, l, pos, _, true) = e.actor()? else {
                return None;
            };
            if l != level {
                return None;
            }
            let dist = actor_pos.chebyshev_distance(pos);
            (dist <= 6 && (dist <= 1 || vision::has_line_of_sight(world, actor_pos, pos)))
                .then_some((e.id, pos, dist))
        })
        .min_by_key(|&(_, _, d)| d)
        .map(|(id, pos, _)| (id, pos))
}

fn teleport_monster_safely(
    world: &mut GameWorld,
    rng: &mut GameRng,
    actor: EntityId,
) -> Option<Pos> {
    let (level, from) = world.entities.actor_location(actor)?;
    let map = world.map(level);
    let mut candidates = Vec::new();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = Pos { x, y };
            if pos.chebyshev_distance(from) >= 4
                && map.tile(pos).ok() == Some(TileKind::Floor)
                && world.entities.alive_actor_at(level, pos).is_none()
            {
                candidates.push(pos);
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    let idx = (rng.next_u64() as usize) % candidates.len();
    let dest = candidates[idx];
    world
        .state_mut()
        .entities
        .set_actor_location(actor, level, dest);
    Some(dest)
}

fn apply_monster_intent(
    world: &mut GameWorld,
    rng: &mut GameRng,
    intent: MonsterIntent,
    events: &mut Vec<GameEvent>,
) -> Result<(), String> {
    match intent {
        MonsterIntent::Wait => {}
        MonsterIntent::Move { entity, direction } => {
            let Some((_, from)) = world.entities.actor_location(entity) else {
                return Ok(());
            };
            if movement::move_actor(world, entity, direction).is_ok() {
                let to = world
                    .entities
                    .actor_location(entity)
                    .expect("monster move succeeded so location must exist")
                    .1;
                events.push(GameEvent::EntityMoved { entity, from, to });
            }
        }
        MonsterIntent::MeleeAttack { attacker, defender } => {
            if !melee_intent_is_still_valid(world, attacker, defender) {
                return Ok(());
            }
            let Some(resolution) = combat::resolve_attack(world, rng, attacker, defender) else {
                return Ok(());
            };
            let hit = resolution.hit;
            events.push(combat::attack_event(&resolution));
            if hit && defender == world.player_id {
                match world
                    .entities
                    .get(attacker)
                    .and_then(|entity| entity.monster_passive())
                {
                    Some(MonsterPassive::PoisonOnMelee) => {
                        if crate::systems::items::has_poison_resistance(world) {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You resist the poison!".to_string(),
                            });
                        } else {
                            let next_poison = world.poison_turns.max(3);
                            world.state_mut().poison_turns = next_poison;
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                        }
                    }
                    Some(MonsterPassive::FireOnMelee) => {
                        if crate::systems::items::has_fire_resistance(world) {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You resist the fire!".to_string(),
                            });
                        } else {
                            let fire_dmg = combat::roll_die(rng, 6);
                            if let Some(stats) =
                                world.state_mut().entities.actor_stats_mut(defender)
                            {
                                stats.hp = stats.hp.saturating_sub(fire_dmg);
                            }
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: format!("The fire ant burns you for {fire_dmg} fire damage!"),
                            });
                        }
                    }
                    Some(MonsterPassive::StealGold) if world.campaign.is_some() => {
                        let stolen =
                            world.gold.min(15_u32.saturating_add(
                                u32::try_from(combat::roll_die(rng, 25)).unwrap_or(1),
                            ));
                        if stolen > 0 {
                            world.state_mut().gold -= stolen;
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                            let from_pos = world.entities.actor_location(attacker).map(|(_, p)| p);
                            if let (Some(from), Some(to)) =
                                (from_pos, teleport_monster_safely(world, rng, attacker))
                            {
                                events.push(GameEvent::EntityMoved {
                                    entity: attacker,
                                    from,
                                    to,
                                });
                            }
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: format!(
                                    "The leprechaun steals {stolen} gold and teleports away!"
                                ),
                            });
                        }
                    }
                    Some(MonsterPassive::StealItem) if world.campaign.is_some() => {
                        if let Some((stolen_id, stolen_kind)) = pick_stealable_player_item(world) {
                            let item_name = aihack_content::item_display_name(
                                stolen_kind,
                                world.content_registry(),
                            );
                            if world
                                .entities
                                .get(stolen_id)
                                .and_then(|e| e.item_tool_active())
                                == Some(true)
                            {
                                world
                                    .state_mut()
                                    .entities
                                    .set_item_tool_active(stolen_id, false);
                            }
                            if items::remove_inventory_item(world, stolen_id).is_ok() {
                                let entities = &mut world.state_mut().entities;
                                entities.set_item_location(
                                    stolen_id,
                                    EntityLocation::Inventory { owner: attacker },
                                );
                                entities.push_monster_inventory(attacker, stolen_id);
                                events.push(GameEvent::PassiveAttackTriggered {
                                    source: attacker,
                                    target: defender,
                                });
                                let from_pos =
                                    world.entities.actor_location(attacker).map(|(_, p)| p);
                                if let (Some(from), Some(to)) =
                                    (from_pos, teleport_monster_safely(world, rng, attacker))
                                {
                                    events.push(GameEvent::EntityMoved {
                                        entity: attacker,
                                        from,
                                        to,
                                    });
                                }
                                events.push(GameEvent::Message {
                                    priority: MessagePriority::Warning,
                                    text: format!(
                                        "The wood nymph steals your {item_name} and teleports away!"
                                    ),
                                });
                            }
                        }
                    }
                    Some(MonsterPassive::RustMetal) if world.campaign.is_some() => {
                        if let Some((metal_id, metal_kind, next_erosion)) =
                            pick_rustable_equipped_item(world)
                        {
                            world
                                .state_mut()
                                .entities
                                .set_item_erosion(metal_id, next_erosion);
                            let _ = items::recompute_player_ac(world);
                            let item_name = aihack_content::item_display_name(
                                metal_kind,
                                world.content_registry(),
                            );
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: format!("The rust monster corrodes your {item_name}!"),
                            });
                        }
                    }
                    Some(MonsterPassive::SleepOnMelee) => {
                        if world
                            .effective_player_resistances()
                            .contains(Resistances::SLEEP)
                        {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "You resist the drowsy touch!".to_string(),
                            });
                        } else {
                            world.state_mut().paralysis_turns = world.paralysis_turns.max(2);
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Warning,
                                text: "A wave of drowsiness puts you to sleep!".to_string(),
                            });
                        }
                    }
                    Some(MonsterPassive::DrainLife) => {
                        if world.has_magic_resistance() {
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Info,
                                text: "Your magic resistance wards off the life-draining chill!"
                                    .to_string(),
                            });
                        } else {
                            let drain_dmg = combat::roll_die(rng, 6) + combat::roll_die(rng, 4);
                            if let Some(mut campaign) = world.campaign {
                                if campaign.xp >= 20 {
                                    campaign.xp -= 20;
                                    let cap_hp = campaign.max_hp();
                                    let cap_mp = campaign.max_mp();
                                    let state = world.state_mut();
                                    state.campaign = Some(campaign);
                                    state.max_mp = state.max_mp.min(cap_mp);
                                    state.mp = state.mp.min(state.max_mp);
                                    if let Some(stats) = state.entities.actor_stats_mut(defender) {
                                        stats.max_hp = stats.max_hp.min(cap_hp).max(10);
                                        stats.hp =
                                            stats.hp.saturating_sub(drain_dmg).min(stats.max_hp);
                                        stats.hit_bonus = campaign.hit_bonus();
                                        stats.damage_bonus = campaign.damage_bonus();
                                    }
                                } else if let Some(stats) =
                                    world.state_mut().entities.actor_stats_mut(defender)
                                {
                                    stats.max_hp = (stats.max_hp - 2).max(10);
                                    stats.hp = stats.hp.saturating_sub(drain_dmg).min(stats.max_hp);
                                }
                            } else if let Some(stats) =
                                world.state_mut().entities.actor_stats_mut(defender)
                            {
                                stats.max_hp = (stats.max_hp - 2).max(10);
                                stats.hp = stats.hp.saturating_sub(drain_dmg).min(stats.max_hp);
                            }
                            events.push(GameEvent::PassiveAttackTriggered {
                                source: attacker,
                                target: defender,
                            });
                            events.push(GameEvent::Message {
                                priority: MessagePriority::Danger,
                                text: "You feel your life force drained away!".to_string(),
                            });
                        }
                    }
                    Some(
                        MonsterPassive::ParalyzeOnMelee
                        | MonsterPassive::Regenerate
                        | MonsterPassive::FireBreath
                        | MonsterPassive::ColdBreath
                        | MonsterPassive::LightningBreath
                        | MonsterPassive::StealGold
                        | MonsterPassive::StealItem
                        | MonsterPassive::RustMetal
                        | MonsterPassive::SplitOnHit
                        | MonsterPassive::GazeParalyze
                        | MonsterPassive::SummonAllies,
                    )
                    | None => {}
                }
            }
            events.extend(death::collect_death_events_after_attack(
                world, attacker, defender,
            )?);
        }
        MonsterIntent::RangedAttack { attacker, defender } => {
            if !ranged_intent_is_still_valid(world, attacker, defender) {
                return Ok(());
            }
            let passive = world
                .entities
                .get(attacker)
                .and_then(|entity| entity.monster_passive());
            if let Some((breath_res, breath_label)) = match passive {
                Some(MonsterPassive::FireBreath) => Some((Resistances::FIRE, "fire")),
                Some(MonsterPassive::ColdBreath) => Some((Resistances::COLD, "frost")),
                Some(MonsterPassive::LightningBreath) => Some((Resistances::SHOCK, "lightning")),
                _ => None,
            } {
                if defender == world.player_id {
                    let reflected = world.has_reflection();
                    let actual_victim = if reflected { attacker } else { defender };
                    if reflected {
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!(
                                "The dragon's {breath_label} breath reflects right back at it!"
                            ),
                        });
                    }
                    let victim_res = if actual_victim == world.player_id {
                        world.effective_player_resistances()
                    } else {
                        world
                            .entities
                            .actor_stats(actual_victim)
                            .map(|s| s.resistances)
                            .unwrap_or(Resistances::NONE)
                    };
                    if victim_res.contains(breath_res) {
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: if actual_victim == world.player_id {
                                format!("You resist the dragon's {breath_label} breath!")
                            } else {
                                format!("The dragon resists its own {breath_label} breath.")
                            },
                        });
                    } else {
                        let dmg = combat::roll_die(rng, 6) + combat::roll_die(rng, 6);
                        if let Some(stats) =
                            world.state_mut().entities.actor_stats_mut(actual_victim)
                        {
                            stats.hp = stats.hp.saturating_sub(dmg);
                        }
                        events.push(GameEvent::PassiveAttackTriggered {
                            source: attacker,
                            target: actual_victim,
                        });
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Warning,
                            text: format!("The dragon breathes {breath_label} for {dmg} damage!"),
                        });
                        events.extend(death::collect_death_events_after_attack(
                            world,
                            attacker,
                            actual_victim,
                        )?);
                    }
                    return Ok(());
                }
            }
            let Some(resolution) = combat::resolve_attack(world, rng, attacker, defender) else {
                return Ok(());
            };
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world, attacker, defender,
            )?);
        }
    }
    Ok(())
}

fn pick_stealable_player_item(world: &GameWorld) -> Option<(EntityId, ItemKind)> {
    let eligible = |item_id: EntityId, require_unequipped: bool| -> Option<(EntityId, ItemKind)> {
        if require_unequipped && world.inventory.slot_of(item_id).is_some() {
            return None;
        }
        let e = world.entities.get(item_id)?;
        let (kind, data, _, _, _) = e.item()?;
        if kind == ItemKind::AmuletAscension
            || data.container_capacity.is_some()
            || e.item_unpaid() == Some(true)
        {
            return None;
        }
        let is_cursed_worn = e.item_buc() == Some(BucStatus::Cursed)
            && (world.inventory.slot_of(item_id).is_some() || e.item_tool_active() == Some(true));
        if is_cursed_worn {
            return None;
        }
        Some((item_id, kind))
    };
    world
        .inventory
        .entries
        .iter()
        .find_map(|entry| eligible(entry.item, true))
        .or_else(|| {
            world
                .inventory
                .entries
                .iter()
                .find_map(|entry| eligible(entry.item, false))
        })
}

fn pick_rustable_equipped_item(world: &GameWorld) -> Option<(EntityId, ItemKind, u8)> {
    let reg = world.content_registry();
    for (_, item_opt) in world.inventory.equipped_slots() {
        let Some(item_id) = item_opt else {
            continue;
        };
        let Some(entity) = world.entities.get(item_id) else {
            continue;
        };
        if entity.item_artifact_id().is_some() {
            continue;
        }
        let Some((kind, _, _, _, _)) = entity.item() else {
            continue;
        };
        let material = reg
            .item(aihack_content::item_id(kind))
            .and_then(|d| d.material.as_deref());
        if !matches!(material, Some("iron" | "steel")) {
            continue;
        }
        let cur = entity.item_erosion().unwrap_or(0);
        if cur < 3 {
            return Some((item_id, kind, cur + 1));
        }
    }
    None
}

fn regenerate_monsters(world: &mut GameWorld, events: &mut Vec<GameEvent>) {
    if world.player_alive() {
        for monster_id in world.current_level_hostile_monsters() {
            let Some(passive) = world
                .entities
                .get(monster_id)
                .and_then(|entity| entity.monster_passive())
            else {
                continue;
            };
            if passive != MonsterPassive::Regenerate {
                continue;
            }
            if let Some(stats) = world.state_mut().entities.actor_stats_mut(monster_id) {
                if stats.hp > 0 && stats.hp < stats.max_hp {
                    stats.hp += 1;
                    let hp_after = stats.hp;
                    events.push(GameEvent::EntityHealed {
                        entity: monster_id,
                        amount: 1,
                        hp_after,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
fn decide_monster_intent(
    world: &GameWorld,
    rng: &mut GameRng,
    actor: EntityId,
    turn: u64,
) -> MonsterIntent {
    let speed = world
        .entities
        .actor_stats(actor)
        .map(|stats| stats.speed)
        .unwrap_or_default();
    if !acts_on_turn(speed, turn) {
        return MonsterIntent::Wait;
    }
    decide_monster_action(world, rng, actor)
}

fn decide_monster_action(world: &GameWorld, rng: &mut GameRng, actor: EntityId) -> MonsterIntent {
    let Some(entity) = world.entities.get(actor) else {
        return MonsterIntent::Wait;
    };
    let Some(ai_kind) = entity.monster_ai_kind() else {
        return MonsterIntent::Wait;
    };
    let Some((_, actor_pos)) = world.entities.actor_location(actor) else {
        return MonsterIntent::Wait;
    };
    if ai_kind == MonsterAiKind::Stationary {
        return MonsterIntent::Wait;
    }
    let player_id = world.player_id;
    let player_pos = world.player_pos();
    if crate::systems::engrave::has_active_elbereth(world, player_pos)
        && entity.actor_kind() != Some(ActorKind::Monster(MonsterKind::Minotaur))
        && (actor_pos.chebyshev_distance(player_pos) <= 1
            || vision::monster_has_line_of_sight_to_player(world, actor))
    {
        return choose_flee_intent(world, actor, actor_pos, player_pos);
    }
    if actor_pos.chebyshev_distance(player_pos) <= 1 {
        return MonsterIntent::MeleeAttack {
            attacker: actor,
            defender: player_id,
        };
    }
    match ai_kind {
        MonsterAiKind::Wander => choose_wander_intent(world, rng, actor),
        MonsterAiKind::ChaseVisiblePlayer => {
            choose_chase_intent(world, actor, actor_pos, player_pos)
        }
        MonsterAiKind::RangedOnSight => {
            if ranged_shot_has_clear_line(world, actor_pos, player_pos) {
                MonsterIntent::RangedAttack {
                    attacker: actor,
                    defender: player_id,
                }
            } else {
                choose_chase_intent(world, actor, actor_pos, player_pos)
            }
        }
        MonsterAiKind::Stationary => MonsterIntent::Wait,
    }
}

#[cfg(test)]
fn acts_on_turn(speed: i16, turn: u64) -> bool {
    if speed <= 0 || turn == 0 {
        return false;
    }
    let speed = speed as u64;
    turn.saturating_mul(speed) / 12 > (turn - 1).saturating_mul(speed) / 12
}

fn choose_wander_intent(world: &GameWorld, rng: &mut GameRng, actor: EntityId) -> MonsterIntent {
    let offset = (rng.next_u64() as usize) % Direction::ALL.len();
    for index in 0..Direction::ALL.len() {
        let direction = Direction::ALL[(offset + index) % Direction::ALL.len()];
        if movement::is_passable_for_actor(world, actor, direction) {
            return MonsterIntent::Move {
                entity: actor,
                direction,
            };
        }
    }
    MonsterIntent::Wait
}

fn choose_chase_intent(
    world: &GameWorld,
    actor: EntityId,
    actor_pos: Pos,
    player_pos: Pos,
) -> MonsterIntent {
    if !vision::monster_has_line_of_sight_to_player(world, actor) {
        return MonsterIntent::Wait;
    }
    let current = (
        actor_pos.chebyshev_distance(player_pos),
        manhattan_distance(actor_pos, player_pos),
    );
    for direction in Direction::ALL {
        if !movement::is_passable_for_actor(world, actor, direction) {
            continue;
        }
        let to = actor_pos.offset(direction.delta());
        let next = (
            to.chebyshev_distance(player_pos),
            manhattan_distance(to, player_pos),
        );
        if next < current {
            return MonsterIntent::Move {
                entity: actor,
                direction,
            };
        }
    }
    MonsterIntent::Wait
}

fn choose_flee_intent(
    world: &GameWorld,
    actor: EntityId,
    actor_pos: Pos,
    player_pos: Pos,
) -> MonsterIntent {
    let current = (
        actor_pos.chebyshev_distance(player_pos),
        manhattan_distance(actor_pos, player_pos),
    );
    let mut best_dir = None;
    let mut best_score = current;
    for direction in Direction::ALL {
        if !movement::is_passable_for_actor(world, actor, direction) {
            continue;
        }
        let to = actor_pos.offset(direction.delta());
        let next = (
            to.chebyshev_distance(player_pos),
            manhattan_distance(to, player_pos),
        );
        if next > best_score {
            best_score = next;
            best_dir = Some(direction);
        }
    }
    match best_dir {
        Some(direction) => MonsterIntent::Move {
            entity: actor,
            direction,
        },
        None => MonsterIntent::Wait,
    }
}

fn melee_intent_is_still_valid(world: &GameWorld, attacker: EntityId, defender: EntityId) -> bool {
    let Some((attacker_level, attacker_pos)) = world.entities.actor_location(attacker) else {
        return false;
    };
    let Some((defender_level, defender_pos)) = world.entities.actor_location(defender) else {
        return false;
    };
    attacker_level == defender_level && attacker_pos.chebyshev_distance(defender_pos) <= 1
}

fn ranged_shot_has_clear_line(world: &GameWorld, from: Pos, to: Pos) -> bool {
    let delta = from.delta_to(to);
    if (delta.dx != 0 && delta.dy != 0) || (delta.dx == 0 && delta.dy == 0) {
        return false;
    }
    let distance = delta.dx.abs().max(delta.dy.abs());
    if distance > 5 {
        return false;
    }
    let step_x = delta.dx.signum();
    let step_y = delta.dy.signum();
    let level = world.current_level();
    let map = world.current_map();
    for step in 1..distance {
        let pos = Pos {
            x: from.x + step_x * step,
            y: from.y + step_y * step,
        };
        let blocked = matches!(
            map.tile(pos),
            Ok(TileKind::Wall
                | TileKind::Door(DoorState::Closed | DoorState::Locked)
                | TileKind::HiddenDoor)
                | Err(_)
        );
        if blocked || world.entities.alive_hostile_at(level, pos).is_some() {
            return false;
        }
    }
    true
}

fn ranged_intent_is_still_valid(world: &GameWorld, attacker: EntityId, defender: EntityId) -> bool {
    let Some((attacker_level, attacker_pos)) = world.entities.actor_location(attacker) else {
        return false;
    };
    let Some((defender_level, defender_pos)) = world.entities.actor_location(defender) else {
        return false;
    };
    attacker_level == defender_level
        && attacker_level == world.current_level()
        && ranged_shot_has_clear_line(world, attacker_pos, defender_pos)
}

fn manhattan_distance(from: Pos, to: Pos) -> i16 {
    let delta = from.delta_to(to);
    delta.dx.abs() + delta.dy.abs()
}

#[allow(dead_code)]
fn _profile_check() -> (AttackProfile, DamageRoll) {
    (
        AttackProfile::natural("bolt", DamageRoll::new(1, 6)),
        DamageRoll::new(1, 6),
    )
}

#[cfg(test)]
mod tests {
    use aihack_core::rng::GameRng;

    use super::collect_monster_turn_at;
    use crate::world::GameWorld;

    #[test]
    fn monster_plan_is_seed_deterministic_inside_the_transaction_boundary() {
        let world = GameWorld::fixture_phase5();
        let mut first = GameRng::new(42);
        let mut second = GameRng::new(42);

        assert_eq!(
            collect_monster_turn_at(&world, &mut first, 1),
            collect_monster_turn_at(&world, &mut second, 1)
        );
    }
}
