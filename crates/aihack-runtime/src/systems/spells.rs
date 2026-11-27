use aihack_core::{
    campaign::{Role, SkillCategory, SpellKind},
    domain::combat::{AttackProfile, DamageRoll},
    event::{GameEvent, MessagePriority},
    position::Direction,
    rng::GameRng,
};

use crate::{
    systems::{combat, death, projectiles, skills, traps},
    world::GameWorld,
};

pub fn cast(
    world: &mut GameWorld,
    rng: &mut GameRng,
    spell: SpellKind,
    direction: Option<Direction>,
) -> Result<Vec<GameEvent>, String> {
    let Some(campaign) = world.campaign else {
        return Err("spells are only available in campaign mode".to_string());
    };
    if !campaign.knows_spell(spell) {
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
                let extra_damage_bonus = i16::from(campaign.level() / 2)
                    .saturating_add(campaign.attributes.int_spell_bonus())
                    .saturating_add(spell_bonus);
                if let Some(resolution) = combat::resolve_attack_with_profile_and_bonus(
                    world,
                    rng,
                    player_id,
                    target,
                    profile,
                    extra_damage_bonus,
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
        SpellKind::Heal => {
            let heal = combat::roll_die(rng, 6)
                + combat::roll_die(rng, 6)
                + 4
                + i16::from(campaign.level())
                + campaign.attributes.wis_heal_bonus()
                + spell_bonus;
            let stats = world
                .state_mut()
                .entities
                .actor_stats_mut(player_id)
                .ok_or_else(|| "player actor stats are missing".to_string())?;
            let before = stats.hp;
            stats.hp = stats.hp.saturating_add(heal).min(stats.max_hp);
            let amount = stats.hp - before;
            let hp_after = stats.hp;
            events.push(GameEvent::EntityHealed {
                entity: player_id,
                amount,
                hp_after,
            });
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
        SpellKind::Detect => {
            events.extend(traps::reveal_all_hidden_tiles(world));
            let count = world.current_level_hostile_monsters().len();
            events.push(GameEvent::Message {
                priority: MessagePriority::Info,
                text: format!("You sense {count} hostile creatures on this level."),
            });
        }
    }
    skills::record_practice(world, SkillCategory::Spellcasting, 5);

    Ok(events)
}
