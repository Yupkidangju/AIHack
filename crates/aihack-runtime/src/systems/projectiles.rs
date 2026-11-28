use aihack_core::{
    action::WishChoice,
    campaign::{artifact_def, SkillCategory},
    domain::{
        combat::{AttackProfile, DamageRoll},
        entity::{ActorKind, EntityLocation, Faction},
        item::{BucStatus, ItemClass, ItemKind, RingEffect, WandEffect},
        monster::MonsterKind,
        status::Resistances,
        tile::{DoorState, TileKind},
    },
    event::{GameEvent, MessagePriority},
    ids::EntityId,
    position::{Direction, Pos},
    rng::GameRng,
};

use crate::{
    domain::{item::try_item_data_from_registry, monster::try_monster_template_from_registry},
    systems::{artifacts, combat, death, items, skills, spells},
    world::GameWorld,
};

pub const CURATED_WISHES: &[(&str, WishChoice)] = &[
    (
        "+2 blessed silver dragon scale mail",
        WishChoice::Item {
            kind: ItemKind::ArmorSilverDragonScale,
            blessed: true,
            enchant: 2,
        },
    ),
    (
        "+2 blessed gray dragon scale mail",
        WishChoice::Item {
            kind: ItemKind::ArmorGrayDragonScale,
            blessed: true,
            enchant: 2,
        },
    ),
    (
        "+2 blessed speed boots",
        WishChoice::Item {
            kind: ItemKind::ArmorSpeedBoots,
            blessed: true,
            enchant: 2,
        },
    ),
    (
        "blessed amulet of life saving",
        WishChoice::Item {
            kind: ItemKind::AmuletLifeSaving,
            blessed: true,
            enchant: 0,
        },
    ),
    (
        "blessed amulet of reflection",
        WishChoice::Item {
            kind: ItemKind::AmuletReflection,
            blessed: true,
            enchant: 0,
        },
    ),
    (
        "+2 blessed silver saber",
        WishChoice::Item {
            kind: ItemKind::SilverSaber,
            blessed: true,
            enchant: 2,
        },
    ),
    (
        "blessed bag of holding",
        WishChoice::Item {
            kind: ItemKind::BagOfHolding,
            blessed: true,
            enchant: 0,
        },
    ),
    ("Excalibur", WishChoice::ArtifactExcalibur),
];

pub fn wish(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    choice: WishChoice,
) -> Result<Vec<GameEvent>, String> {
    if world.campaign.is_none() {
        return Err("wishing is only available in campaign mode".to_string());
    }
    if !world.inventory.contains(item) {
        return Err("wand is not in player inventory".to_string());
    }
    let Some((wand_kind, data, _, _, charges)) = world.entities.get(item).and_then(|e| e.item())
    else {
        return Err("entity is not an item".to_string());
    };
    if data.wand_effect != Some(WandEffect::Wishing) {
        return Err("item is not a wand of wishing".to_string());
    }
    let Some(charges_before) = charges else {
        return Err("wand has no charge state".to_string());
    };
    if charges_before == 0 {
        return Err("wand has no charges".to_string());
    }

    let (spawn_kind, blessed, raw_enchant, artifact_id) = match choice {
        WishChoice::Item {
            kind,
            blessed,
            enchant,
        } => {
            if matches!(
                kind,
                ItemKind::AmuletAscension
                    | ItemKind::WandWishing
                    | ItemKind::Corpse
                    | ItemKind::CorpseJackal
            ) {
                return Err("that item cannot be wished for".to_string());
            }
            if !(-3..=3).contains(&enchant) {
                return Err("wished enchantment must be within -3..=+3".to_string());
            }
            (kind, blessed, enchant, None)
        }
        WishChoice::ArtifactExcalibur => resolve_artifact_wish(world, "excalibur"),
        WishChoice::ArtifactSunsword => resolve_artifact_wish(world, "sunsword"),
        WishChoice::ArtifactMagicbane => resolve_artifact_wish(world, "magicbane"),
        WishChoice::ArtifactCleaver => resolve_artifact_wish(world, "cleaver"),
        WishChoice::ArtifactStormbringer => resolve_artifact_wish(world, "stormbringer"),
        WishChoice::ArtifactSting => resolve_artifact_wish(world, "sting"),
    };

    let item_data = try_item_data_from_registry(spawn_kind, world.content_registry())
        .map_err(|e| e.to_string())?;
    let charges_after = charges_before - 1;
    world
        .state_mut()
        .entities
        .set_item_charges(item, Some(charges_after));
    world.identify_item_kind(wand_kind);
    world.identify_item_kind(spawn_kind);

    let level = world.current_level();
    let ppos = world.player_pos();
    let player_id = world.player_id;
    let spawned_id = world
        .state_mut()
        .entities
        .spawn_item_with_data(
            spawn_kind,
            item_data,
            EntityLocation::OnMap { level, pos: ppos },
        )
        .map_err(|e| e.to_string())?;

    let applied_enchant = if matches!(item_data.class, ItemClass::Weapon | ItemClass::Armor)
        || item_data.ring_effect == Some(RingEffect::Protection)
    {
        raw_enchant
    } else {
        0
    };
    {
        let state = world.state_mut();
        if let Some(art_id) = artifact_id {
            state.generated_artifacts.insert(art_id.to_owned());
            state
                .entities
                .set_item_artifact_id(spawned_id, Some(art_id.to_owned()));
        }
        state.entities.set_item_buc(
            spawned_id,
            if blessed {
                BucStatus::Blessed
            } else {
                BucStatus::Uncursed
            },
        );
        state
            .entities
            .set_item_enchantment(spawned_id, applied_enchant);
        state.entities.set_item_buc_known(spawned_id, true);
        state.entities.set_item_enchant_known(spawned_id, true);
        if let Some(letter) = state.inventory.add_existing_with_next_letter(spawned_id) {
            state
                .entities
                .set_item_location(spawned_id, EntityLocation::Inventory { owner: player_id });
            state.entities.set_item_letter(spawned_id, letter);
        }
    }

    let item_name = aihack_content::item_display_name(spawn_kind, world.content_registry());
    let label = artifact_id
        .and_then(artifact_def)
        .map(|d| d.name.to_string())
        .unwrap_or(item_name);
    let mut events = vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: format!("Your wish is granted! You receive {label}."),
    }];
    if let Some(def) = artifact_id.and_then(artifact_def) {
        if world
            .campaign
            .is_some_and(|c| artifacts::is_artifact_hostile_to(def, c.alignment))
        {
            events.extend(artifacts::apply_artifact_blast(
                world, rng, def, 2, 6, "wish for",
            )?);
        }
    }
    Ok(events)
}

fn resolve_artifact_wish(
    world: &GameWorld,
    art_id: &'static str,
) -> (ItemKind, bool, i8, Option<&'static str>) {
    let def = artifact_def(art_id).expect("known wish artifact");
    if world.generated_artifacts.contains(art_id) {
        (def.base_kind, true, 2, None)
    } else {
        (def.base_kind, true, 2, Some(art_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileOutcome {
    pub landing: Pos,
    pub hit_target: Option<EntityId>,
}

pub fn throw_item(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    direction: Direction,
) -> Result<Vec<GameEvent>, String> {
    let Some((kind, data, _, _, _)) = world.entities.get(item).and_then(|entity| entity.item())
    else {
        return Err("entity is not an item".to_string());
    };
    let data = *data;
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    if !world
        .entities
        .get(item)
        .and_then(|e| e.item_contained_items())
        .is_none_or(<[EntityId]>::is_empty)
    {
        return Err("empty container before throwing".to_string());
    }
    let is_campaign_food_throw = world.campaign.is_some()
        && matches!(
            data.class,
            aihack_core::domain::item::ItemClass::Food
                | aihack_core::domain::item::ItemClass::Corpse
        );
    if data.attack_profile.is_none() && !is_campaign_food_throw {
        return Err("item cannot be thrown".to_string());
    }

    let from = world.player_pos();
    let outcome = trace_path(world, from, direction);
    let level = world.current_level();
    let (qty, buc, enchantment, buc_known, enchant_known) = world
        .entities
        .get(item)
        .map(|entity| {
            (
                entity.item_quantity().unwrap_or(1),
                entity.item_buc().unwrap_or(BucStatus::Uncursed),
                entity.item_enchantment().unwrap_or(0),
                entity.item_buc_known().unwrap_or(false),
                entity.item_enchant_known().unwrap_or(false),
            )
        })
        .unwrap_or((1, BucStatus::Uncursed, 0, false, false));
    let thrown_id = if qty > 1 {
        let spawned = world
            .state_mut()
            .entities
            .spawn_item_with_data(
                kind,
                data,
                EntityLocation::OnMap {
                    level,
                    pos: outcome.landing,
                },
            )
            .map_err(|error| error.to_string())?;
        let entities = &mut world.state_mut().entities;
        entities.set_item_buc(spawned, buc);
        entities.set_item_enchantment(spawned, enchantment);
        entities.set_item_buc_known(spawned, buc_known);
        entities.set_item_enchant_known(spawned, enchant_known);
        entities.set_item_quantity(item, qty - 1);
        spawned
    } else {
        items::remove_inventory_item(world, item)?;
        let entities = &mut world.state_mut().entities;
        if entities.get(item).and_then(|e| e.item_unpaid()) == Some(true) {
            entities.set_item_unpaid(item, false);
            entities.set_item_unpaid_level(item, None);
        }
        entities.set_item_location(
            item,
            EntityLocation::OnMap {
                level,
                pos: outcome.landing,
            },
        );
        item
    };

    let mut events = vec![GameEvent::ItemThrown {
        entity: world.player_id,
        item: thrown_id,
        from,
        to: outcome.landing,
    }];
    if let Some(target) = outcome.hit_target {
        if let Some(feed_events) =
            crate::systems::pets::try_feed_or_tame_by_throw(world, target, thrown_id)
        {
            events.extend(feed_events);
            return Ok(events);
        }
        events.extend(crate::systems::npc::provoke_actor(world, target));
        let mut profile = projectile_profile(kind, data.attack_profile);
        let mut extra_bonus = 0_i16;
        if world.campaign.is_some() {
            profile.hit_bonus = profile.hit_bonus.saturating_add(world.luck);
            if let Some(campaign) = world.campaign {
                profile.hit_bonus = profile
                    .hit_bonus
                    .saturating_add(campaign.attributes.dex_hit_bonus());
            }
            if let Some(thrown_entity) = world.entities.get(thrown_id) {
                let enchant = i16::from(thrown_entity.item_enchantment().unwrap_or(0));
                let erosion = i16::from(thrown_entity.item_erosion().unwrap_or(0));
                let bless_hit = if thrown_entity.item_buc()
                    == Some(aihack_core::domain::item::BucStatus::Blessed)
                {
                    1
                } else {
                    0
                };
                profile.hit_bonus = profile
                    .hit_bonus
                    .saturating_add(enchant)
                    .saturating_add(bless_hit);
                extra_bonus = extra_bonus.saturating_add(enchant).saturating_sub(erosion);
            }
            if let Some(def) = crate::systems::artifacts::artifact_def_for_item(world, thrown_id) {
                profile.hit_bonus = profile.hit_bonus.saturating_add(def.hit_bonus);
                extra_bonus = extra_bonus.saturating_add(def.damage_bonus);
                if def.id == "sting"
                    && matches!(
                        world.entities.get(target).and_then(|e| e.actor_kind()),
                        Some(aihack_core::domain::entity::ActorKind::Monster(
                            aihack_core::domain::monster::MonsterKind::Orc
                                | aihack_core::domain::monster::MonsterKind::Kobold
                                | aihack_core::domain::monster::MonsterKind::Ogre
                        ))
                    )
                {
                    extra_bonus = extra_bonus.saturating_add(3);
                }
            }
        }
        if let Some(cat) = SkillCategory::for_weapon_kind(kind) {
            if world.campaign.is_some() {
                let rank = skills::current_rank(world, cat);
                profile.hit_bonus = profile.hit_bonus.saturating_add(rank.hit_bonus());
                extra_bonus = extra_bonus.saturating_add(rank.damage_bonus());
                skills::record_practice(world, cat, 4);
            }
        }
        if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
            world,
            rng,
            world.player_id,
            target,
            profile,
            extra_bonus,
        ) {
            events.push(combat::attack_event(&resolution));
            events.extend(death::collect_death_events_after_attack(
                world,
                world.player_id,
                target,
            )?);
        }
    } else if world.campaign.is_some() {
        if let Some(cat) = SkillCategory::for_weapon_kind(kind) {
            skills::record_practice(world, cat, 2);
        }
    }
    Ok(events)
}

pub fn zap_wand(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    direction: Direction,
) -> Result<Vec<GameEvent>, String> {
    let Some((wand_kind, data, _, _, charges)) =
        world.entities.get(item).and_then(|entity| entity.item())
    else {
        return Err("entity is not an item".to_string());
    };
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let Some(wand_effect) = data.wand_effect else {
        return Err("item is not a wand".to_string());
    };
    let Some(charges_before) = charges else {
        return Err("wand has no charge state".to_string());
    };
    if charges_before == 0 {
        return Err("wand has no charges".to_string());
    }
    if wand_effect == WandEffect::Wishing {
        if world.campaign.is_some() {
            world.identify_item_kind(wand_kind);
        }
        return Ok(vec![GameEvent::Message {
            priority: MessagePriority::Info,
            text: "Use the wish action to make a wish from this wand.".to_string(),
        }]);
    }

    let charges_after = charges_before - 1;
    world
        .state_mut()
        .entities
        .set_item_charges(item, Some(charges_after));
    if world.campaign.is_some()
        && !matches!(wand_kind, ItemKind::WandMagicMissile | ItemKind::WandFire)
    {
        world.identify_item_kind(wand_kind);
    }

    let player_id = world.player_id;
    let ppos = world.player_pos();
    let outcome = trace_path(world, ppos, direction);
    let mut events = vec![GameEvent::WandZapped {
        entity: player_id,
        item,
        direction,
        charges_after,
    }];

    match wand_effect {
        WandEffect::MagicMissile => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let profile = AttackProfile::natural("magic missile", DamageRoll::new(1, 6));
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world, rng, player_id, target, profile, 0,
                ) {
                    events.push(combat::attack_event(&resolution));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            }
        }
        WandEffect::Fire => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if target_has_resistance(world, target, Resistances::FIRE) {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The fire bolt has no effect on the fire-resistant creature!"
                            .to_string(),
                    });
                    return Ok(events);
                }
                let profile = AttackProfile::natural("fire bolt", DamageRoll::new(2, 6));
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world, rng, player_id, target, profile, 2,
                ) {
                    events.push(combat::attack_event(&resolution));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            }
        }
        WandEffect::Cold => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if target_has_resistance(world, target, Resistances::COLD) {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The cold ray has no effect on the cold-resistant creature!"
                            .to_string(),
                    });
                } else {
                    let profile = AttackProfile::natural("cold ray", DamageRoll::new(2, 6));
                    if let Some(res) = combat::resolve_attack_with_profile_and_bonus(
                        world, rng, player_id, target, profile, 2,
                    ) {
                        events.push(combat::attack_event(&res));
                        events.extend(death::collect_death_events_after_attack(
                            world, player_id, target,
                        )?);
                    }
                }
            }
        }
        WandEffect::Lightning => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if target_has_resistance(world, target, Resistances::SHOCK) {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The lightning bolt leaves the shock-resistant creature unharmed!"
                            .to_string(),
                    });
                } else {
                    let profile = AttackProfile::natural("lightning bolt", DamageRoll::new(3, 6));
                    if let Some(res) = combat::resolve_attack_with_profile_and_bonus(
                        world, rng, player_id, target, profile, 2,
                    ) {
                        events.push(combat::attack_event(&res));
                        events.extend(death::collect_death_events_after_attack(
                            world, player_id, target,
                        )?);
                    }
                }
            }
        }
        WandEffect::Striking => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let profile = AttackProfile::natural("force strike", DamageRoll::new(2, 12));
                if let Some(res) = combat::resolve_attack_with_profile_and_bonus(
                    world, rng, player_id, target, profile, 0,
                ) {
                    events.push(combat::attack_event(&res));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            } else {
                let next = outcome.landing.offset(direction.delta());
                if let Ok(TileKind::Door(ds @ (DoorState::Closed | DoorState::Locked))) =
                    world.current_map().tile(next)
                {
                    let _ = world
                        .current_map_mut()
                        .set_tile(next, TileKind::Door(DoorState::Open));
                    events.push(GameEvent::DoorChanged {
                        pos: next,
                        from: ds,
                        to: DoorState::Open,
                    });
                }
            }
        }
        WandEffect::Sleep => {
            if let Some(target) = outcome.hit_target {
                if target_has_resistance(world, target, Resistances::SLEEP) {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature resists the sleep beam.".to_string(),
                    });
                } else {
                    if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                        stats.energy = -36;
                    }
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature collapses into enchanted slumber!".to_string(),
                    });
                }
            }
        }
        WandEffect::SlowMonster => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.speed = (stats.speed / 2).max(3);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The creature moves more sluggishly.".to_string(),
                });
            }
        }
        WandEffect::SpeedMonster => {
            if let Some(target) = outcome.hit_target {
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.speed = (stats.speed + 6).min(24);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: "The creature speeds up!".to_string(),
                });
            }
        }
        WandEffect::MakeInvisible => {
            if let Some(target) = outcome.hit_target {
                world.state_mut().entities.set_actor_invisible(target, true);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The shimmer renders the creature invisible!".to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The shimmering beam fades away.".to_string(),
                });
            }
        }
        WandEffect::Digging => {
            let carved = spells::carve_interior_walls(world, ppos, direction, 4);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: if carved > 0 {
                    format!("The wand blasts through {carved} rock tiles!")
                } else {
                    "Nothing gives way before the digging beam.".to_string()
                },
            });
        }
        WandEffect::Opening => {
            let mut cur = ppos;
            for _ in 0..8 {
                cur = cur.offset(direction.delta());
                let Ok(tile) = world.current_map().tile(cur) else {
                    break;
                };
                if let Some(prev_state) = match tile {
                    TileKind::Door(ds @ (DoorState::Closed | DoorState::Locked)) => Some(ds),
                    TileKind::HiddenDoor => Some(DoorState::Closed),
                    _ => None,
                } {
                    let open = TileKind::Door(DoorState::Open);
                    let _ = world.current_map_mut().set_tile(cur, open);
                    events.push(GameEvent::DoorChanged {
                        pos: cur,
                        from: prev_state,
                        to: DoorState::Open,
                    });
                    break;
                }
                if tile == TileKind::Wall {
                    break;
                }
            }
        }
        WandEffect::Locking => {
            let mut cur = ppos;
            for _ in 0..8 {
                cur = cur.offset(direction.delta());
                let Ok(tile) = world.current_map().tile(cur) else {
                    break;
                };
                if let TileKind::Door(ds @ (DoorState::Open | DoorState::Closed)) = tile {
                    if world
                        .entities
                        .alive_actor_at(world.current_level(), cur)
                        .is_none()
                    {
                        let locked = TileKind::Door(DoorState::Locked);
                        let _ = world.current_map_mut().set_tile(cur, locked);
                        events.push(GameEvent::DoorChanged {
                            pos: cur,
                            from: ds,
                            to: DoorState::Locked,
                        });
                        break;
                    }
                }
                if tile == TileKind::Wall {
                    break;
                }
            }
        }
        WandEffect::Teleport => {
            if let Some(target) = outcome.hit_target {
                let level = world.current_level();
                let from_pos = world.entities.actor_location(target).map(|(_, p)| p);
                if let (Some(from), Some(dest)) = (
                    from_pos,
                    find_teleport_destination(world, rng, level, from_pos.unwrap_or(ppos)),
                ) {
                    world
                        .state_mut()
                        .entities
                        .set_actor_location(target, level, dest);
                    events.push(GameEvent::EntityMoved {
                        entity: target,
                        from,
                        to: dest,
                    });
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature disappears in a flash!".to_string(),
                    });
                }
            }
        }
        WandEffect::Cancellation => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                world
                    .state_mut()
                    .entities
                    .set_actor_invisible(target, false);
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.resistances = Resistances::NONE;
                    stats.hit_bonus = stats.hit_bonus.saturating_sub(1);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A gray flash strips the creature's magical defenses!".to_string(),
                });
            }
            let floor_items = world
                .entities
                .items_at(world.current_level(), outcome.landing);
            for fid in floor_items {
                if world.entities.get(fid).and_then(|e| e.item_buc()) == Some(BucStatus::Cursed) {
                    world
                        .state_mut()
                        .entities
                        .set_item_buc(fid, BucStatus::Uncursed);
                }
            }
        }
        WandEffect::Death => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let reg = world.content_registry();
                let is_undead = matches!(
                    world.entities.get(target).and_then(|e| e.actor()),
                    Some((ActorKind::Monster(mk), _, _, _, _, _))
                        if aihack_content::monster_has_ai_flag(mk, "undead", reg)
                );
                if is_undead || target_has_resistance(world, target, Resistances::MAGIC_RESISTANCE)
                {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The death ray has no effect!".to_string(),
                    });
                } else {
                    if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                        stats.hp = 0;
                    }
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "A lethal dark ray strikes down your foe!".to_string(),
                    });
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            }
        }
        WandEffect::Polymorph => {
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let reg = world.content_registry();
                let target_mk = world.entities.get(target).and_then(|e| match e.actor() {
                    Some((ActorKind::Monster(mk), _, _, _, _, true)) => Some(mk),
                    _ => None,
                });
                if let Some(old_mk) = target_mk {
                    if aihack_content::monster_has_ai_flag(old_mk, "npc", reg)
                        || old_mk == MonsterKind::Minotaur
                    {
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: "The creature resists transformation!".to_string(),
                        });
                    } else {
                        let level = world.current_level();
                        let candidates: Vec<MonsterKind> =
                            aihack_content::weighted_monster_kinds_for_depth(level.depth, reg)
                                .into_iter()
                                .map(|(mk, _)| mk)
                                .filter(|&mk| mk != old_mk)
                                .collect();
                        let new_mk = if !candidates.is_empty() {
                            candidates[(rng.next_u64() as usize) % candidates.len()]
                        } else if old_mk == MonsterKind::Jackal {
                            MonsterKind::Goblin
                        } else {
                            MonsterKind::Jackal
                        };
                        let tmpl = try_monster_template_from_registry(new_mk, reg)
                            .map_err(|e| e.to_string())?;
                        let hp = aihack_content::roll_monster_hp_from_dice(new_mk, reg, rng)
                            .unwrap_or(tmpl.hp)
                            .max(1);
                        let new_name = aihack_content::monster_display_name(new_mk, reg);
                        let next_stats = aihack_core::domain::entity::ActorStats {
                            hp,
                            max_hp: hp,
                            ac: tmpl.ac,
                            hit_bonus: tmpl.hit_bonus,
                            damage_bonus: tmpl.damage_bonus,
                            damage_reduction: 0,
                            damage: tmpl.attack_profile.damage,
                            weapon_hit_bonus: tmpl.attack_profile.hit_bonus,
                            speed: tmpl.speed,
                            energy: 0,
                            resistances: tmpl.resistances,
                            ai_kind: Some(tmpl.ai_kind),
                            passive: tmpl.passive,
                            difficulty: tmpl.difficulty,
                        };
                        world
                            .state_mut()
                            .entities
                            .polymorph_monster(target, new_mk, next_stats);
                        events.push(GameEvent::Message {
                            priority: MessagePriority::Info,
                            text: format!("The creature transforms into a {new_name}!"),
                        });
                    }
                }
            }
        }
        WandEffect::CreateMonster => {
            let level = world.current_level();
            if let Some(spawn_pos) = Direction::ALL.into_iter().find_map(|d| {
                let candidate = ppos.offset(d.delta());
                let passable = world
                    .map(level)
                    .tile(candidate)
                    .is_ok_and(|t| matches!(t, TileKind::Floor | TileKind::Shop | TileKind::Altar));
                (passable && world.entities.alive_actor_at(level, candidate).is_none())
                    .then_some(candidate)
            }) {
                let reg = world.content_registry();
                let pool = aihack_content::weighted_monster_kinds_for_depth(level.depth, reg);
                let new_mk = if pool.is_empty() {
                    MonsterKind::Jackal
                } else {
                    pool[(rng.next_u64() as usize) % pool.len()].0
                };
                let tmpl =
                    try_monster_template_from_registry(new_mk, reg).map_err(|e| e.to_string())?;
                let name = aihack_content::monster_display_name(new_mk, reg);
                let spawned = world
                    .state_mut()
                    .entities
                    .spawn_monster_with_faction(new_mk, tmpl, Faction::Hostile, spawn_pos)
                    .map_err(|e| e.to_string())?;
                world
                    .state_mut()
                    .entities
                    .set_actor_location(spawned, level, spawn_pos);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text: format!("A {name} materializes nearby!"),
                });
            }
        }
        WandEffect::Wishing => {}
    }
    Ok(events)
}

fn target_has_resistance(world: &GameWorld, target: EntityId, flag: Resistances) -> bool {
    world
        .entities
        .actor_stats(target)
        .is_some_and(|stats| stats.resistances.contains(flag))
}

fn find_teleport_destination(
    world: &GameWorld,
    rng: &mut GameRng,
    level: aihack_core::ids::LevelId,
    from: Pos,
) -> Option<Pos> {
    let map = world.map(level);
    let mut candidates = Vec::new();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = Pos { x, y };
            if pos.chebyshev_distance(from) >= 3
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
    Some(candidates[idx])
}

fn projectile_profile(kind: ItemKind, base: Option<AttackProfile>) -> AttackProfile {
    match kind {
        ItemKind::Rock => AttackProfile::natural("rock", DamageRoll::new(1, 3)),
        _ => base.unwrap_or(AttackProfile::natural("thrown item", DamageRoll::new(1, 2))),
    }
}

fn trace_path(world: &GameWorld, from: Pos, direction: Direction) -> ProjectileOutcome {
    trace_path_with_max_distance(world, from, direction, i16::MAX)
}

pub(crate) fn trace_path_with_max_distance(
    world: &GameWorld,
    from: Pos,
    direction: Direction,
    max_distance: i16,
) -> ProjectileOutcome {
    let mut current = from;
    let mut steps = 0_i16;
    loop {
        if steps >= max_distance {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        steps += 1;
        let next = current.offset(direction.delta());
        if !world.current_map().contains(next) {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        if let Some(target) = world.entities.alive_actor_at(world.current_level(), next) {
            if target != world.player_id {
                return ProjectileOutcome {
                    landing: next,
                    hit_target: Some(target),
                };
            }
        }
        let Ok(tile) = world.current_map().tile(next) else {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        };
        if matches!(
            tile,
            TileKind::Wall
                | TileKind::Boulder
                | TileKind::Door(DoorState::Closed | DoorState::Locked)
                | TileKind::HiddenDoor
        ) {
            return ProjectileOutcome {
                landing: current,
                hit_target: None,
            };
        }
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use aihack_core::{
        domain::{
            combat::{AttackProfile, DamageRoll},
            entity::EntityLocation,
            item::ItemKind,
        },
        position::{Direction, Pos},
        rng::GameRng,
    };

    use super::throw_item;
    use crate::{domain::item::item_data, systems::items, world::GameWorld};

    #[test]
    fn throwing_an_equipped_body_item_uses_the_common_unequip_lifecycle() {
        let mut world = GameWorld::fixture_without_monsters();
        let owner = world.player_id();
        let mut armor_data = item_data(ItemKind::ArmorLeather);
        armor_data.attack_profile = Some(AttackProfile::natural("fixture", DamageRoll::new(1, 4)));
        let armor = world
            .state_mut()
            .entities
            .spawn_item_with_data(
                ItemKind::ArmorLeather,
                armor_data,
                EntityLocation::Inventory { owner },
            )
            .unwrap();
        let letter = world
            .state_mut()
            .inventory
            .add_existing_with_next_letter(armor)
            .unwrap();
        assert!(world.state_mut().entities.set_item_letter(armor, letter));
        assert!(items::wear(&mut world, armor).unwrap().is_some());
        assert_eq!(
            world.entities.actor_stats(owner).unwrap().ac,
            10 - armor_data.ac_bonus
        );

        let mut rng = GameRng::new(42);
        throw_item(&mut world, &mut rng, armor, Direction::East).unwrap();

        assert_eq!(world.inventory.equipped_body, None);
        assert_eq!(world.entities.actor_stats(owner).unwrap().ac, 10);
        assert!(!world.inventory.contains(armor));
        assert_eq!(
            world.entities.item_location(armor),
            Some(EntityLocation::OnMap {
                level: world.current_level(),
                pos: Pos { x: 9, y: 5 },
            })
        );
    }
}
