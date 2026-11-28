use aihack_core::{
    campaign::{
        artifact_def, eligible_sacrifice_artifacts, Alignment, ArtifactDef,
        ARTIFACT_INVOKE_COOLDOWN_TURNS,
    },
    domain::{
        combat::DeathCause,
        entity::{ActorKind, EntityLocation, Faction},
        item::{BucStatus, ItemKind},
        status::Resistances,
        tile::TileKind,
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    rng::GameRng,
};

use crate::{
    domain::item::try_item_data_from_registry,
    systems::{combat, death, traps},
    world::GameWorld,
};

pub(crate) fn artifact_def_for_item(
    world: &GameWorld,
    item: EntityId,
) -> Option<&'static ArtifactDef> {
    world
        .entities
        .get(item)
        .and_then(|entity| entity.item_artifact_id())
        .and_then(artifact_def)
}

pub(crate) fn is_artifact_hostile_to(def: &ArtifactDef, player_align: Alignment) -> bool {
    def.alignment.is_some_and(|align| align != player_align)
}

pub(crate) fn is_light_emitting_artifact_held(world: &GameWorld) -> bool {
    world
        .inventory
        .entries
        .iter()
        .any(|entry| artifact_def_for_item(world, entry.item).is_some_and(|def| def.emits_light))
}

pub(crate) fn inventory_artifact_resistances(world: &GameWorld) -> Resistances {
    let mut total = Resistances::NONE;
    for entry in &world.inventory.entries {
        if let Some(def) = artifact_def_for_item(world, entry.item) {
            total.insert(def.resistances);
        }
    }
    total
}

pub(crate) fn apply_artifact_blast(
    world: &mut GameWorld,
    rng: &mut GameRng,
    def: &ArtifactDef,
    dice_count: i16,
    dice_sides: i16,
    verb: &str,
) -> Result<Vec<GameEvent>, String> {
    let mut dmg = 0_i16;
    for _ in 0..dice_count {
        dmg = dmg.saturating_add(combat::roll_die(rng, dice_sides));
    }
    let player_id = world.player_id;
    if let Some(stats) = world.state_mut().entities.actor_stats_mut(player_id) {
        stats.hp = stats.hp.saturating_sub(dmg);
    }
    let mut events = vec![GameEvent::Message {
        priority: MessagePriority::Danger,
        text: format!(
            "You are blasted by {} when trying to {verb} it for {dmg} damage!",
            def.name
        ),
    }];
    events.extend(death::collect_death_events_if_hp_depleted(
        world,
        player_id,
        DeathCause::ArtifactBlast,
    )?);
    Ok(events)
}

pub(crate) fn try_grant_sacrifice_artifact(
    world: &mut GameWorld,
    rng: &mut GameRng,
) -> Result<Option<(EntityId, &'static ArtifactDef)>, String> {
    let Some(campaign) = world.campaign else {
        return Ok(None);
    };
    let candidates = eligible_sacrifice_artifacts(campaign.alignment, &world.generated_artifacts);
    if candidates.is_empty() {
        return Ok(None);
    }
    let idx = (rng.next_u64() as usize) % candidates.len();
    let def = candidates[idx];
    let data = try_item_data_from_registry(def.base_kind, world.content_registry())
        .map_err(|err| err.to_string())?;
    let level = world.current_level();
    let pos = world.player_pos();
    let item_id = world
        .state_mut()
        .entities
        .spawn_item_with_data(def.base_kind, data, EntityLocation::OnMap { level, pos })
        .map_err(|err| err.to_string())?;
    let state = world.state_mut();
    state
        .entities
        .set_item_artifact_id(item_id, Some(def.id.to_owned()));
    state.entities.set_item_buc(item_id, BucStatus::Blessed);
    state.entities.set_item_buc_known(item_id, true);
    state.entities.set_item_enchantment(item_id, 1);
    state.entities.set_item_enchant_known(item_id, true);
    state.entities.set_item_erosion(item_id, 0);
    state.generated_artifacts.insert(def.id.to_owned());
    world.identify_item_kind(def.base_kind);
    Ok(Some((item_id, def)))
}

pub(crate) fn try_fountain_excalibur(
    world: &mut GameWorld,
    item: EntityId,
) -> Result<Option<Vec<GameEvent>>, String> {
    let Some(campaign) = world.campaign else {
        return Ok(None);
    };
    if campaign.alignment != Alignment::Lawful {
        return Ok(None);
    }
    let Some(entity) = world.entities.get(item) else {
        return Ok(None);
    };
    let Some((kind, _, _, _, _)) = entity.item() else {
        return Ok(None);
    };
    if kind != ItemKind::LongSword
        || entity.item_artifact_id().is_some()
        || world.generated_artifacts.contains("excalibur")
    {
        return Ok(None);
    }
    if campaign.level() < 5 {
        return Ok(Some(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "The fountain water ripples around your sword, but you lack the experience to claim Excalibur (requires level 5).".to_string(),
        }]));
    }
    let pos = world.player_pos();
    let enchant = entity.item_enchantment().unwrap_or(0).max(1);
    let _ = world.current_map_mut().set_tile(pos, TileKind::DryFountain);
    let state = world.state_mut();
    state
        .entities
        .set_item_artifact_id(item, Some("excalibur".to_owned()));
    state.entities.set_item_buc(item, BucStatus::Blessed);
    state.entities.set_item_buc_known(item, true);
    state.entities.set_item_enchantment(item, enchant);
    state.entities.set_item_enchant_known(item, true);
    state.entities.set_item_erosion(item, 0);
    state.generated_artifacts.insert("excalibur".to_owned());
    world.identify_item_kind(ItemKind::LongSword);
    Ok(Some(vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "From the murky depths, a hand rises and anoints your sword as Excalibur!"
            .to_string(),
    }]))
}

pub(crate) fn invoke(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
) -> Result<Vec<GameEvent>, String> {
    let Some(mut campaign) = world.campaign else {
        return Err("artifact invocation is only available in campaign mode".to_string());
    };
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let Some(def) = artifact_def_for_item(world, item) else {
        return Err("that item is not a unique artifact".to_string());
    };
    if is_artifact_hostile_to(def, campaign.alignment) {
        return apply_artifact_blast(world, rng, def, 4, 4, "invoke");
    }
    if campaign.artifact_invoke_cooldown > 0 {
        return Err("artifact invoke is on cooldown".to_string());
    }

    campaign.artifact_invoke_cooldown = ARTIFACT_INVOKE_COOLDOWN_TURNS;
    world.state_mut().campaign = Some(campaign);

    let player_id = world.player_id;
    let level = world.current_level();
    let pos = world.player_pos();
    let mut events = Vec::new();

    match def.id {
        "excalibur" => {
            let heal = combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
            let state = world.state_mut();
            state.status_timers.blind = 0;
            state.status_timers.confused = 0;
            if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                let before = stats.hp;
                stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
                let amount = stats.hp - before;
                if amount > 0 {
                    events.push(GameEvent::EntityHealed {
                        entity: player_id,
                        amount,
                        hp_after: stats.hp,
                    });
                }
            }
            events.extend(traps::search(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Excalibur shines with restorative clarity!".to_string(),
            });
        }
        "sunsword" => {
            let targets = adjacent_hostile_targets(world, level, pos);
            for target in targets {
                let dmg = combat::roll_die(rng, 6) + combat::roll_die(rng, 6);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            world.mark_current_level_explored();
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Sunsword erupts in a blinding solar flare!".to_string(),
            });
        }
        "magicbane" => {
            let state = world.state_mut();
            state.mp = state.mp.saturating_add(8).min(state.max_mp);
            state.status_timers.confused = 0;
            state.status_timers.stunned = 0;
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Magicbane weaves an arcane ward around you!".to_string(),
            });
        }
        "cleaver" => {
            let targets = adjacent_hostile_targets(world, level, pos);
            for target in targets {
                let dmg = combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Cleaver carves a devastating arc around you!".to_string(),
            });
        }
        "stormbringer" => {
            let nearest = nearest_hostile_target(world, level, pos, 5);
            if let Some(target) = nearest {
                let dmg = combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                if let Some(pstats) = world.state_mut().entities.actor_stats_mut(player_id) {
                    pstats.hp = pstats.hp.saturating_add(3).min(pstats.max_hp);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Stormbringer drinks life through a dark lightning bolt!".to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Stormbringer crackles hungrily in the air.".to_string(),
                });
            }
        }
        "sting" => {
            events.extend(traps::reveal_all_hidden_tiles(world));
            let state = world.state_mut();
            state.status_timers.fast = state.status_timers.fast.max(4);
            let nearby = count_hostiles_within(world, level, pos, 10);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("Sting glows cold blue — you sense {nearby} hostile foes nearby!"),
            });
        }
        "magic_mirror_of_merlin" => {
            events.extend(traps::reveal_all_hidden_tiles(world));
            let state = world.state_mut();
            state.status_timers.blind = 0;
            state.status_timers.hallucinating = 0;
            state.hallucinating = false;
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Magic Mirror of Merlin reveals hidden truths!".to_string(),
            });
        }
        "cloak_of_the_wayfarer" => {
            let state = world.state_mut();
            state.status_timers.invisible = state.status_timers.invisible.max(8);
            state.status_timers.fast = state.status_timers.fast.max(5);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Cloak of the Wayfarer conceals your form in wind and shadow!"
                    .to_string(),
            });
        }
        "tome_of_arcana" => {
            let state = world.state_mut();
            state.mp = state.mp.saturating_add(16).min(state.max_mp);
            state.status_timers.confused = 0;
            state.status_timers.stunned = 0;
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Tome of Arcana surges with arcane mana!".to_string(),
            });
        }
        "candelabrum_of_grace" => {
            let heal = combat::roll_die(rng, 8) + combat::roll_die(rng, 8) + 4;
            let state = world.state_mut();
            state.poison_turns = 0;
            if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                let before = stats.hp;
                stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
                let amount = stats.hp - before;
                if amount > 0 {
                    events.push(GameEvent::EntityHealed {
                        entity: player_id,
                        amount,
                        hp_after: stats.hp,
                    });
                }
            }
            events.extend(traps::reveal_all_hidden_tiles(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Candelabrum of Grace shines with consecrating warmth!".to_string(),
            });
        }
        "orb_of_fate" => {
            for target in adjacent_hostile_targets(world, level, pos) {
                let dmg = combat::roll_die(rng, 6) + combat::roll_die(rng, 6);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            let state = world.state_mut();
            state.status_timers.fast = state.status_timers.fast.max(4);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Orb of Fate bends battle fortune in your favor!".to_string(),
            });
        }
        "eye_of_the_aethiopica" => {
            let state = world.state_mut();
            state.mp = state.max_mp;
            state.status_timers.confused = 0;
            events.extend(traps::reveal_all_hidden_tiles(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Eye of the Aethiopica opens your mind to total clarity!".to_string(),
            });
        }
        "master_key_of_thievery" => {
            for dir in aihack_core::position::Direction::ALL {
                let adj = pos.offset(dir.delta());
                if world.current_map().tile(adj).ok()
                    == Some(TileKind::Door(aihack_core::domain::tile::DoorState::Locked))
                {
                    let _ = world.current_map_mut().set_tile(
                        adj,
                        TileKind::Door(aihack_core::domain::tile::DoorState::Closed),
                    );
                    events.push(GameEvent::DoorChanged {
                        pos: adj,
                        from: aihack_core::domain::tile::DoorState::Locked,
                        to: aihack_core::domain::tile::DoorState::Closed,
                    });
                }
            }
            events.extend(traps::reveal_all_hidden_tiles(world));
            let state = world.state_mut();
            state.status_timers.invisible = state.status_timers.invisible.max(6);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Master Key of Thievery unlocks nearby wards!".to_string(),
            });
        }
        "mitre_of_holiness" => {
            let heal =
                combat::roll_die(rng, 8) + combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
            let state = world.state_mut();
            state.mp = state.mp.saturating_add(6).min(state.max_mp);
            state.poison_turns = 0;
            if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                let before = stats.hp;
                stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
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
                text: "The Mitre of Holiness blesses you with sacred vigor!".to_string(),
            });
        }
        "tsurugi_of_muramasa" => {
            for target in adjacent_hostile_targets(world, level, pos) {
                let dmg =
                    combat::roll_die(rng, 8) + combat::roll_die(rng, 8) + combat::roll_die(rng, 8);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Tsurugi of Muramasa unleashes a razor-sharp tempest!".to_string(),
            });
        }
        "heart_of_ahriman" => {
            if let Some(target) = nearest_hostile_target(world, level, pos, 6) {
                let dmg =
                    combat::roll_die(rng, 6) + combat::roll_die(rng, 6) + combat::roll_die(rng, 6);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            let state = world.state_mut();
            state.status_timers.fast = state.status_timers.fast.max(6);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Heart of Ahriman pulses with relentless battle fury!".to_string(),
            });
        }
        "longbow_of_diana" => {
            if let Some(target) = nearest_hostile_target(world, level, pos, 8) {
                let dmg = combat::roll_die(rng, 10) + combat::roll_die(rng, 10);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.hp = stats.hp.saturating_sub(dmg);
                }
                events.extend(death::collect_death_events_after_attack(
                    world, player_id, target,
                )?);
            }
            events.extend(traps::search(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Longbow of Diana looses an unerring silver shaft!".to_string(),
            });
        }
        "eyes_of_the_overworld" => {
            let state = world.state_mut();
            state.status_timers.blind = 0;
            state.status_timers.confused = 0;
            state.status_timers.hallucinating = 0;
            state.hallucinating = false;
            events.extend(traps::reveal_all_hidden_tiles(world));
            world.mark_current_level_explored();
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "The Eyes of the Overworld pierce every veil of darkness!".to_string(),
            });
        }
        "orb_of_detection" => {
            events.extend(traps::reveal_all_hidden_tiles(world));
            world.mark_current_level_explored();
            let nearby = count_hostiles_within(world, level, pos, 12);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!(
                    "The Orb of Detection maps your surroundings ({nearby} foes sensed)!"
                ),
            });
        }
        _ => {}
    }

    Ok(events)
}

fn adjacent_hostile_targets(
    world: &GameWorld,
    level: aihack_core::ids::LevelId,
    pos: aihack_core::position::Pos,
) -> Vec<EntityId> {
    let mut targets: Vec<EntityId> = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            let (ActorKind::Monster(_), faction, m_level, m_pos, _, alive) = entity.actor()? else {
                return None;
            };
            (alive
                && faction == Faction::Hostile
                && m_level == level
                && pos.chebyshev_distance(m_pos) <= 1)
                .then_some(entity.id)
        })
        .collect();
    targets.sort_by_key(|id| id.0);
    targets
}

fn nearest_hostile_target(
    world: &GameWorld,
    level: aihack_core::ids::LevelId,
    pos: aihack_core::position::Pos,
    max_dist: i16,
) -> Option<EntityId> {
    world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            let (ActorKind::Monster(_), faction, m_level, m_pos, _, alive) = entity.actor()? else {
                return None;
            };
            let dist = pos.chebyshev_distance(m_pos);
            (alive
                && faction == Faction::Hostile
                && m_level == level
                && dist <= max_dist
                && crate::systems::vision::has_line_of_sight(world, pos, m_pos))
            .then_some((dist, entity.id))
        })
        .min_by_key(|(dist, id)| (*dist, id.0))
        .map(|(_, id)| id)
}

fn count_hostiles_within(
    world: &GameWorld,
    level: aihack_core::ids::LevelId,
    pos: aihack_core::position::Pos,
    max_dist: i16,
) -> usize {
    world
        .entities
        .entities()
        .iter()
        .filter(|entity| {
            entity
                .actor()
                .is_some_and(|(ak, faction, m_level, m_pos, _, alive)| {
                    matches!(ak, ActorKind::Monster(_))
                        && alive
                        && faction == Faction::Hostile
                        && m_level == level
                        && pos.chebyshev_distance(m_pos) <= max_dist
                })
        })
        .count()
}
