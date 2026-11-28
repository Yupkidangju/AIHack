use aihack_core::{
    campaign::{
        LearnedSpell, Role, SkillCategory, SpellKind, MAX_LEARNED_SPELLS, MAX_LUCK,
        SPELL_MEMORY_TURNS,
    },
    domain::{
        combat::{AttackProfile, DamageRoll, DeathCause},
        entity::{ActorKind, EntityLocation, Faction, DEFAULT_PET_HUNGER},
        item::{BucStatus, ItemClass},
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
    domain::monster::try_monster_template_from_registry,
    systems::{combat, death, items, projectiles, skills, traps},
    world::GameWorld,
};

pub fn knows_spell_at(world: &GameWorld, spell: SpellKind, turn: u64) -> bool {
    let Some(campaign) = world.campaign else {
        return false;
    };
    if campaign.knows_spell(spell) {
        return true;
    }
    world.learned_spells.iter().any(|entry| {
        entry.spell == spell && turn.saturating_sub(entry.learned_at_turn) < SPELL_MEMORY_TURNS
    })
}

pub fn is_spell_expired(world: &GameWorld, spell: SpellKind, turn: u64) -> bool {
    let Some(campaign) = world.campaign else {
        return false;
    };
    if campaign.knows_spell(spell) {
        return false;
    }
    world.learned_spells.iter().any(|entry| {
        entry.spell == spell && turn.saturating_sub(entry.learned_at_turn) >= SPELL_MEMORY_TURNS
    })
}

pub fn spell_failure_percent(world: &GameWorld, spell: SpellKind) -> u8 {
    let Some(campaign) = world.campaign else {
        return 0;
    };
    let registry = world.content_registry();
    let is_metal_material = |slot_item: Option<EntityId>| -> bool {
        slot_item
            .and_then(|id| world.entities.item_data(id))
            .and_then(|data| aihack_content::item_material(data.kind, registry))
            .is_some_and(|mat| matches!(mat, "iron" | "steel"))
    };
    let mut metal_penalty = 0_i16;
    if is_metal_material(world.inventory.equipped_body) {
        metal_penalty += 25;
    }
    if is_metal_material(world.inventory.equipped_shield) {
        metal_penalty += 15;
    }
    if campaign.knows_spell(spell) && metal_penalty == 0 {
        return 0;
    }
    let rank = skills::current_rank(world, SkillCategory::Spellcasting);
    let base = i16::from(spell.spell_level()) * 16 + metal_penalty;
    let reduction = campaign.attributes.int_spell_bonus() * 6
        + i16::from(campaign.level().saturating_sub(1)) * 4
        + i16::from(rank.as_u8()) * 10
        + if campaign.knows_spell(spell) { 20 } else { 0 };
    (base - reduction).clamp(0, 100) as u8
}

pub fn study_spellbook(
    world: &mut GameWorld,
    rng: &mut GameRng,
    item: EntityId,
    turn: u64,
) -> Result<Vec<GameEvent>, String> {
    let Some(campaign) = world.campaign else {
        return Err("spellbooks are only available in campaign mode".to_string());
    };
    if !world.inventory.contains(item) {
        return Err("item is not in player inventory".to_string());
    }
    let Some((kind, data, _, _, charges)) = world.entities.get(item).and_then(|e| e.item()) else {
        return Err("entity is not an item".to_string());
    };
    let data = *data;
    if data.class != ItemClass::Spellbook {
        return Err("item is not a spellbook".to_string());
    }
    let Some(spell) = data.spellbook_spell else {
        return Err("spellbook has no encoded spell".to_string());
    };
    let remaining = charges.unwrap_or(0);
    if remaining == 0 {
        return Err("spellbook crumbles away before it can be read".to_string());
    }
    let buc = world
        .entities
        .get(item)
        .and_then(|e| e.item_buc())
        .unwrap_or(BucStatus::Uncursed);
    let already_known = world
        .learned_spells
        .iter()
        .any(|entry| entry.spell == spell);
    if buc != BucStatus::Cursed
        && !already_known
        && world.learned_spells.len() >= MAX_LEARNED_SPELLS
    {
        return Err("your spell memory is full".to_string());
    }

    world.identify_item_kind(kind);
    world.state_mut().entities.set_item_buc_known(item, true);

    let next_charges = remaining - 1;
    world
        .state_mut()
        .entities
        .set_item_charges(item, Some(next_charges));

    let mut events = Vec::new();
    if buc == BucStatus::Cursed {
        let dmg = combat::roll_die(rng, 6);
        let player_id = world.player_id;
        {
            let state = world.state_mut();
            state.status_timers.confused = state.status_timers.confused.max(4);
            state.status_timers.blind = state.status_timers.blind.max(3);
            if let Some(stats) = state.entities.actor_stats_mut(player_id) {
                stats.hp = stats.hp.saturating_sub(dmg);
            }
        }
        events.push(GameEvent::Message {
            priority: MessagePriority::Danger,
            text: format!(
                "The cursed spellbook's runes explode in blind confusion for {dmg} damage!"
            ),
        });
        events.extend(death::collect_death_events_if_hp_depleted(
            world,
            player_id,
            DeathCause::Explosion,
        )?);
    } else {
        let rank = skills::current_rank(world, SkillCategory::Spellcasting);
        let study_difficulty = i16::from(spell.spell_level()) * 3;
        let study_power = i16::from(campaign.level())
            + campaign.attributes.int_spell_bonus() * 2
            + i16::from(rank.as_u8()) * 2
            + 4;
        let study_failed = if buc == BucStatus::Uncursed && study_power < study_difficulty {
            let fail_chance =
                u64::try_from(((study_difficulty - study_power) * 12).clamp(15, 90)).unwrap_or(50);
            (rng.next_u64() % 100) < fail_chance
        } else {
            false
        };
        if study_failed {
            world.state_mut().status_timers.confused = world.status_timers.confused.max(2);
            skills::record_practice(world, SkillCategory::Spellcasting, 1);
            events.push(GameEvent::Message {
                priority: MessagePriority::Warning,
                text: format!(
                    "You struggle to decipher {} and grow confused.",
                    spell.display_name()
                ),
            });
        } else {
            let learned = &mut world.state_mut().learned_spells;
            if let Some(existing) = learned.iter_mut().find(|entry| entry.spell == spell) {
                existing.learned_at_turn = turn;
            } else {
                learned.push(LearnedSpell {
                    spell,
                    learned_at_turn: turn,
                });
            }
            skills::record_practice(world, SkillCategory::Spellcasting, 4);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!(
                    "You study the spellbook and learn {}!",
                    spell.display_name()
                ),
            });
        }
    }
    if next_charges == 0 {
        items::remove_inventory_item(world, item)?;
        items::mark_item_consumed(&mut world.state_mut().entities, item);
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: "The spellbook turns to dust.".to_string(),
        });
    }
    Ok(events)
}

pub fn cast_at(
    world: &mut GameWorld,
    rng: &mut GameRng,
    spell: SpellKind,
    direction: Option<Direction>,
    turn: u64,
) -> Result<Vec<GameEvent>, String> {
    let Some(campaign) = world.campaign else {
        return Err("spells are only available in campaign mode".to_string());
    };
    if is_spell_expired(world, spell, turn) {
        return Err("your memory of this spell has faded; re-study its spellbook".to_string());
    }
    if !knows_spell_at(world, spell, turn) {
        return Err("your role does not know this spell".to_string());
    }
    let cost = spell.mana_cost();
    if world.mp < cost {
        return Err("not enough MP".to_string());
    }
    let validated_direction = match (spell.is_directional(), direction) {
        (true, Some(dir)) => Some(dir),
        (true, None) => return Err("this spell requires a direction".to_string()),
        (false, None) => None,
        (false, Some(_)) => return Err("this spell does not take a direction".to_string()),
    };

    let fail_pct = spell_failure_percent(world, spell);
    if fail_pct > 0 && (rng.next_u64() % 100) < u64::from(fail_pct) {
        let fail_cost = (cost / 2).max(1);
        world.state_mut().mp = world.mp.saturating_sub(fail_cost);
        let mp_after = world.mp;
        let player_id = world.player_id;
        skills::record_practice(world, SkillCategory::Spellcasting, 1);
        return Ok(vec![
            GameEvent::SpellCast {
                entity: player_id,
                spell,
                direction: validated_direction,
                mp_after,
            },
            GameEvent::Message {
                priority: MessagePriority::Warning,
                text: format!(
                    "You fail to cast {} cleanly ({fail_pct}%).",
                    spell.display_name()
                ),
            },
        ]);
    }

    world.state_mut().mp -= cost;
    let mp_after = world.mp;
    let player_id = world.player_id;
    let mut events = vec![GameEvent::SpellCast {
        entity: player_id,
        spell,
        direction: validated_direction,
        mp_after,
    }];

    let spell_bonus = skills::current_rank(world, SkillCategory::Spellcasting).spell_power_bonus();
    let power_bonus = i16::from(campaign.level() / 2)
        .saturating_add(campaign.attributes.int_spell_bonus())
        .saturating_add(spell_bonus);

    match spell {
        SpellKind::ForceBolt => {
            let dir = validated_direction.expect("validated directional spell has direction");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 5);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let profile = AttackProfile {
                    name: "force bolt",
                    hit_bonus: 4 + (spell_bonus / 2),
                    damage: DamageRoll::new(2, 6),
                };
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world,
                    rng,
                    player_id,
                    target,
                    profile,
                    power_bonus,
                ) {
                    events.push(combat::attack_event(&resolution));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The force bolt dissipates harmlessly.".to_string(),
                });
            }
        }
        SpellKind::MagicMissile => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 7);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let profile = AttackProfile {
                    name: "magic missile",
                    hit_bonus: 6 + (spell_bonus / 2),
                    damage: DamageRoll::new(3, 6),
                };
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world,
                    rng,
                    player_id,
                    target,
                    profile,
                    power_bonus,
                ) {
                    events.push(combat::attack_event(&resolution));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The magic missile streaks past empty air.".to_string(),
                });
            }
        }
        SpellKind::Fireball => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            events.extend(apply_elemental_blast(
                world,
                rng,
                outcome.landing,
                1,
                "fireball",
                DamageRoll::new(4, 6),
                power_bonus,
                Resistances::FIRE,
            )?);
        }
        SpellKind::ConeOfCold => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            events.extend(apply_elemental_blast(
                world,
                rng,
                outcome.landing,
                1,
                "cone of cold",
                DamageRoll::new(4, 6),
                power_bonus,
                Resistances::COLD,
            )?);
        }
        SpellKind::LightningBolt => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 7);
            if let Some(target) = outcome.hit_target {
                events.extend(apply_elemental_hit(
                    world,
                    rng,
                    target,
                    "lightning bolt",
                    DamageRoll::new(5, 6),
                    power_bonus,
                    Resistances::SHOCK,
                )?);
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The lightning bolt grounds itself harmlessly.".to_string(),
                });
            }
        }
        SpellKind::FingerOfDeath => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 7);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let reg = world.content_registry();
                let is_undead = matches!(
                    world.entities.get(target).and_then(|e| e.actor()),
                    Some((ActorKind::Monster(mk), _, _, _, _, _))
                        if aihack_content::monster_has_ai_flag(mk, "undead", reg)
                );
                let has_mr = world
                    .entities
                    .actor_stats(target)
                    .is_some_and(|s| s.resistances.contains(Resistances::MAGIC_RESISTANCE));
                if is_undead || has_mr {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The deadly beam leaves the creature untouched.".to_string(),
                    });
                } else {
                    if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                        stats.hp = 0;
                    }
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "A dark ray slays the target instantly!".to_string(),
                    });
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            }
        }
        SpellKind::Heal => {
            let heal = combat::roll_die(rng, 6)
                + combat::roll_die(rng, 6)
                + 4
                + i16::from(campaign.level())
                + campaign.attributes.wis_heal_bonus()
                + spell_bonus;
            events.push(apply_player_heal(world, heal)?);
            if matches!(campaign.role, Role::Cleric | Role::Priest | Role::Monk)
                && world.poison_turns > 0
            {
                world.state_mut().poison_turns = 0;
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Holy magic purges the poison!".to_string(),
                });
            }
        }
        SpellKind::ExtraHealing => {
            let heal = (0..4).map(|_| combat::roll_die(rng, 6)).sum::<i16>()
                + 10
                + i16::from(campaign.level())
                + campaign.attributes.wis_heal_bonus()
                + spell_bonus;
            clear_player_ailments(world);
            events.push(apply_player_heal(world, heal)?);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "You feel vastly restored!".to_string(),
            });
        }
        SpellKind::CureSickness => {
            clear_player_ailments(world);
            let heal = combat::roll_die(rng, 6) + 2;
            events.push(apply_player_heal(world, heal)?);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Your ailments vanish.".to_string(),
            });
        }
        SpellKind::StoneToFlesh => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 5);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                let profile = AttackProfile {
                    name: "transmutation",
                    hit_bonus: 5 + (spell_bonus / 2),
                    damage: DamageRoll::new(3, 6),
                };
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world,
                    rng,
                    player_id,
                    target,
                    profile,
                    power_bonus,
                ) {
                    events.push(combat::attack_event(&resolution));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The surrounding stone softens slightly.".to_string(),
                });
            }
        }
        SpellKind::Detect => {
            events.extend(traps::reveal_all_hidden_tiles(world));
            let count = world.current_level_hostile_monsters().len();
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("You sense {count} hostile creatures on this level."),
            });
        }
        SpellKind::IdentifySpell => {
            let unidentified = world
                .inventory
                .entries
                .iter()
                .map(|e| e.item)
                .find(|id| {
                    world
                        .entities
                        .item_data(*id)
                        .is_some_and(|d| !world.is_item_identified(d.kind))
                })
                .or_else(|| {
                    world.inventory.entries.iter().map(|e| e.item).find(|id| {
                        world.entities.get(*id).is_some_and(|e| {
                            !e.item_buc_known().unwrap_or(false)
                                || !e.item_enchant_known().unwrap_or(false)
                        })
                    })
                });
            if let Some(target) = unidentified {
                events.extend(items::identify_inventory_target(world, target));
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "Your possessions are already thoroughly understood.".to_string(),
                });
            }
        }
        SpellKind::MagicMapping => {
            events.extend(reveal_entire_level_map(world));
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "A mystical map of this floor forms in your mind.".to_string(),
            });
        }
        SpellKind::DetectTreasure => {
            let level = world.current_level();
            let positions: Vec<Pos> = world
                .entities
                .entities()
                .iter()
                .filter_map(|e| match e.item()?.2 {
                    EntityLocation::OnMap { level: l, pos } if l == level => Some(pos),
                    _ => None,
                })
                .collect();
            let count = positions.len();
            if let Some(lvl) = world.state_mut().levels.get_mut(level) {
                lvl.mark_explored(&positions);
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("You detect {count} unattended items on this level."),
            });
        }
        SpellKind::SleepSpell => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            if let Some(target) = outcome.hit_target {
                let resists_sleep = world
                    .entities
                    .actor_stats(target)
                    .is_some_and(|s| s.resistances.contains(Resistances::SLEEP));
                if resists_sleep {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature shrugs off the soporific wave.".to_string(),
                    });
                } else {
                    if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                        stats.energy = -36;
                    }
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature falls fast asleep!".to_string(),
                    });
                }
            }
        }
        SpellKind::ConfuseMonster => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.energy = stats.energy.saturating_sub(24).max(-36);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The creature reels in magical confusion!".to_string(),
                });
            }
        }
        SpellKind::CharmMonster => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            if let Some(target) = outcome.hit_target {
                let reg = world.content_registry();
                let eligible = matches!(
                    world.entities.get(target).and_then(|e| e.actor()),
                    Some((ActorKind::Monster(mk), faction, _, _, _, true))
                        if matches!(faction, Faction::Hostile | Faction::Neutral)
                            && !aihack_content::monster_has_ai_flag(mk, "npc", reg)
                            && mk != MonsterKind::Minotaur
                );
                if eligible {
                    let entities = &mut world.state_mut().entities;
                    entities.set_actor_faction(target, Faction::Pet);
                    entities.set_actor_tameness(target, 5);
                    entities.set_actor_pet_hunger(target, DEFAULT_PET_HUNGER);
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature gazes at you with newfound loyalty!".to_string(),
                    });
                } else {
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature resists your enchantment.".to_string(),
                    });
                }
            }
        }
        SpellKind::SlowMonster => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 6);
            if let Some(target) = outcome.hit_target {
                events.extend(crate::systems::npc::provoke_actor(world, target));
                if let Some(stats) = world.state_mut().entities.actor_stats_mut(target) {
                    stats.speed = (stats.speed / 2).max(3);
                }
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "The creature's movements noticeably slow!".to_string(),
                });
            }
        }
        SpellKind::RemoveCurseSpell => {
            let cleared = items::uncurse_equipped_items(world);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: if cleared > 0 {
                    "You feel a comforting aura lift curses from your gear.".to_string()
                } else {
                    "You feel as if someone is watching over you.".to_string()
                },
            });
        }
        SpellKind::Protection => {
            let heal = combat::roll_die(rng, 4) + combat::roll_die(rng, 4);
            events.push(apply_player_heal(world, heal)?);
            let next_luck = world.luck.saturating_add(1).min(MAX_LUCK);
            world.state_mut().luck = next_luck;
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "A protective grace strengthens your resolve.".to_string(),
            });
        }
        SpellKind::TurnUndead => {
            let level = world.current_level();
            let ppos = world.player_pos();
            let reg = world.content_registry();
            let targets: Vec<EntityId> = world
                .entities
                .entities()
                .iter()
                .filter_map(|e| {
                    let (ActorKind::Monster(mk), _, l, pos, _, true) = e.actor()? else {
                        return None;
                    };
                    (l == level
                        && ppos.chebyshev_distance(pos) <= 5
                        && aihack_content::monster_has_ai_flag(mk, "undead", reg))
                    .then_some(e.id)
                })
                .collect();
            for target in targets {
                let profile = AttackProfile {
                    name: "holy light",
                    hit_bonus: 10,
                    damage: DamageRoll::new(2, 8),
                };
                if let Some(res) = combat::resolve_attack_with_profile_and_bonus(
                    world,
                    rng,
                    player_id,
                    target,
                    profile,
                    power_bonus + 4,
                ) {
                    events.push(combat::attack_event(&res));
                    events.extend(death::collect_death_events_after_attack(
                        world, player_id, target,
                    )?);
                }
            }
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "Sacred light radiates outward against the undead!".to_string(),
            });
        }
        SpellKind::CreateFamiliar => {
            let level = world.current_level();
            let ppos = world.player_pos();
            if let Some(spawn_pos) = Direction::ALL.into_iter().find_map(|d| {
                let candidate = ppos.offset(d.delta());
                let passable = world
                    .map(level)
                    .tile(candidate)
                    .is_ok_and(|t| matches!(t, TileKind::Floor | TileKind::Shop | TileKind::Altar));
                (passable && world.entities.alive_actor_at(level, candidate).is_none())
                    .then_some(candidate)
            }) {
                let kind = if turn.is_multiple_of(2) {
                    MonsterKind::Kitten
                } else {
                    MonsterKind::LittleDog
                };
                let template = try_monster_template_from_registry(kind, world.content_registry())
                    .map_err(|e| e.to_string())?;
                let fam_id = world
                    .state_mut()
                    .entities
                    .spawn_monster_with_faction(kind, template, Faction::Pet, spawn_pos)
                    .map_err(|e| e.to_string())?;
                let entities = &mut world.state_mut().entities;
                entities.set_actor_location(fam_id, level, spawn_pos);
                entities.set_actor_tameness(fam_id, 5);
                entities.set_actor_pet_hunger(fam_id, DEFAULT_PET_HUNGER);
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "A loyal familiar appears at your side!".to_string(),
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "There is no room for a familiar nearby.".to_string(),
                });
            }
        }
        SpellKind::Jumping => {
            let dir = validated_direction.expect("directional spell");
            let level = world.current_level();
            let from = world.player_pos();
            let mut dest = None;
            let mut candidate = from;
            for _step in 1..=2 {
                candidate = candidate.offset(dir.delta());
                let passable = world.map(level).tile(candidate).is_ok_and(|t| {
                    t.is_movement_passable()
                        && !matches!(t, TileKind::Water | TileKind::Lava)
                        && !crate::systems::movement::sokoban_unplugged_pit_at(
                            world, level, candidate,
                        )
                });
                if !passable {
                    break;
                }
                if world.entities.alive_actor_at(level, candidate).is_none() {
                    dest = Some(candidate);
                }
            }
            if let Some(to) = dest {
                world.set_player_location(level, to);
                events.push(GameEvent::EntityMoved {
                    entity: player_id,
                    from,
                    to,
                });
            } else {
                events.push(GameEvent::Message {
                    priority: MessagePriority::Info,
                    text: "You cannot jump that way.".to_string(),
                });
            }
        }
        SpellKind::HasteSelf => {
            let state = world.state_mut();
            state.status_timers.fast = state.status_timers.fast.max(20);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: "You accelerate dramatically!".to_string(),
            });
        }
        SpellKind::TeleportAway => {
            let dir = validated_direction.expect("directional spell");
            let outcome =
                projectiles::trace_path_with_max_distance(world, world.player_pos(), dir, 7);
            if let Some(target) = outcome.hit_target {
                let level = world.current_level();
                let from_pos = world.entities.actor_location(target).map(|(_, p)| p);
                if let (Some(from), Some(to)) = (
                    from_pos,
                    pick_safe_floor_far(world, rng, level, from_pos.unwrap_or(world.player_pos())),
                ) {
                    world
                        .state_mut()
                        .entities
                        .set_actor_location(target, level, to);
                    events.push(GameEvent::EntityMoved {
                        entity: target,
                        from,
                        to,
                    });
                    events.push(GameEvent::Message {
                        priority: MessagePriority::Info,
                        text: "The creature vanishes in a flash of displaced space!".to_string(),
                    });
                }
            }
        }
        SpellKind::Knock => {
            let dir = validated_direction.expect("directional spell");
            let level = world.current_level();
            let mut cur = world.player_pos();
            for _ in 0..4 {
                cur = cur.offset(dir.delta());
                let Ok(tile) = world.map(level).tile(cur) else {
                    break;
                };
                if let Some(prev_state) = match tile {
                    TileKind::Door(ds @ (DoorState::Closed | DoorState::Locked)) => Some(ds),
                    TileKind::HiddenDoor => Some(DoorState::Closed),
                    _ => None,
                } {
                    let _ = world
                        .current_map_mut()
                        .set_tile(cur, TileKind::Door(DoorState::Open));
                    events.push(GameEvent::DoorChanged {
                        pos: cur,
                        from: prev_state,
                        to: DoorState::Open,
                    });
                    break;
                }
            }
        }
        SpellKind::WizardLock => {
            let dir = validated_direction.expect("directional spell");
            let level = world.current_level();
            let mut cur = world.player_pos();
            for _ in 0..3 {
                cur = cur.offset(dir.delta());
                let Ok(tile) = world.map(level).tile(cur) else {
                    break;
                };
                if let TileKind::Door(ds @ (DoorState::Open | DoorState::Closed)) = tile {
                    if world.entities.alive_actor_at(level, cur).is_none() {
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
            }
        }
        SpellKind::DigSpell => {
            let dir = validated_direction.expect("directional spell");
            let carved = carve_interior_walls(world, world.player_pos(), dir, 3);
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: if carved > 0 {
                    format!("Stone crumbles as you carve {carved} passage tiles!")
                } else {
                    "The stone holds firm.".to_string()
                },
            });
        }
    }
    skills::record_practice(world, SkillCategory::Spellcasting, 5);

    Ok(events)
}

pub(crate) fn reveal_entire_level_map(world: &mut GameWorld) -> Vec<GameEvent> {
    let level = world.current_level();
    let map = world.map(level);
    let mut positions = Vec::with_capacity((map.width * map.height).max(0) as usize);
    for y in 0..map.height {
        for x in 0..map.width {
            positions.push(Pos { x, y });
        }
    }
    if let Some(lvl) = world.state_mut().levels.get_mut(level) {
        lvl.mark_explored(&positions);
    }
    traps::reveal_all_hidden_tiles(world)
}

pub(crate) fn carve_interior_walls(
    world: &mut GameWorld,
    from: Pos,
    dir: Direction,
    max_tiles: usize,
) -> usize {
    let current = world.current_level();
    if current.branch == aihack_core::ids::BranchId::Sokoban
        || (current.branch == aihack_core::ids::BranchId::Planes && current.depth > 1)
    {
        return 0;
    }
    let mut carved = 0;
    let mut cur = from;
    let (width, height) = {
        let map = world.current_map();
        (map.width, map.height)
    };
    for _ in 0..6 {
        if carved >= max_tiles {
            break;
        }
        cur = cur.offset(dir.delta());
        if cur.x <= 0 || cur.x >= width - 1 || cur.y <= 0 || cur.y >= height - 1 {
            break;
        }
        if matches!(world.current_map().tile(cur), Ok(TileKind::Wall))
            && world
                .current_map_mut()
                .set_tile(cur, TileKind::Floor)
                .is_ok()
        {
            carved += 1;
        }
    }
    carved
}

fn apply_player_heal(world: &mut GameWorld, heal: i16) -> Result<GameEvent, String> {
    let player_id = world.player_id;
    let stats = world
        .state_mut()
        .entities
        .actor_stats_mut(player_id)
        .ok_or_else(|| "player actor stats are missing".to_string())?;
    let before = stats.hp;
    stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
    Ok(GameEvent::EntityHealed {
        entity: player_id,
        amount: stats.hp - before,
        hp_after: stats.hp,
    })
}

fn clear_player_ailments(world: &mut GameWorld) {
    let state = world.state_mut();
    state.poison_turns = 0;
    state.paralysis_turns = 0;
    state.hallucinating = false;
    state.status_timers.blind = 0;
    state.status_timers.confused = 0;
    state.status_timers.stunned = 0;
    state.status_timers.hallucinating = 0;
}

fn apply_elemental_hit(
    world: &mut GameWorld,
    rng: &mut GameRng,
    target: EntityId,
    name: &'static str,
    damage: DamageRoll,
    bonus: i16,
    resisted_by: Resistances,
) -> Result<Vec<GameEvent>, String> {
    let player_id = world.player_id;
    let mut events = crate::systems::npc::provoke_actor(world, target);
    if world
        .entities
        .actor_stats(target)
        .is_some_and(|s| s.resistances.contains(resisted_by))
    {
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("The {name} has no effect on the resistant creature!"),
        });
        return Ok(events);
    }
    let profile = AttackProfile {
        name,
        hit_bonus: 8,
        damage,
    };
    if let Some(res) =
        combat::resolve_attack_with_profile_and_bonus(world, rng, player_id, target, profile, bonus)
    {
        events.push(combat::attack_event(&res));
        events.extend(death::collect_death_events_after_attack(
            world, player_id, target,
        )?);
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
fn apply_elemental_blast(
    world: &mut GameWorld,
    rng: &mut GameRng,
    center: Pos,
    radius: i16,
    name: &'static str,
    damage: DamageRoll,
    bonus: i16,
    resisted_by: Resistances,
) -> Result<Vec<GameEvent>, String> {
    let level = world.current_level();
    let player_id = world.player_id;
    let victims: Vec<EntityId> = world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(_), _, l, pos, _, true) = e.actor()? else {
                return None;
            };
            (l == level && e.id != player_id && center.chebyshev_distance(pos) <= radius)
                .then_some(e.id)
        })
        .collect();
    let mut events = Vec::new();
    for target in victims {
        events.extend(apply_elemental_hit(
            world,
            rng,
            target,
            name,
            damage,
            bonus,
            resisted_by,
        )?);
    }
    if events.is_empty() {
        events.push(GameEvent::Message {
            priority: MessagePriority::Info,
            text: format!("The {name} erupts and fades."),
        });
    }
    Ok(events)
}

fn pick_safe_floor_far(
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
