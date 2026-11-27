use aihack_core::{
    domain::{
        combat::DeathCause,
        entity::{EntityKind, EntityLocation},
        item::ItemKind,
        monster::MonsterKind,
    },
    event::GameEvent,
    ids::EntityId,
    run_state::RunState,
};

use crate::{systems::score, world::GameWorld};

pub fn collect_death_events_after_attack(
    world: &mut GameWorld,
    attacker: EntityId,
    defender: EntityId,
) -> Result<Vec<GameEvent>, String> {
    collect_death_events_if_hp_depleted(world, defender, DeathCause::Combat { attacker })
}

pub fn collect_death_events_if_hp_depleted(
    world: &mut GameWorld,
    entity: EntityId,
    cause: DeathCause,
) -> Result<Vec<GameEvent>, String> {
    let Some(stats) = world.entities.actor_stats(entity).copied() else {
        return Ok(Vec::new());
    };
    let alive = world
        .entities
        .get(entity)
        .and_then(|entity| entity.actor().map(|(_, _, _, _, _, alive)| alive))
        .unwrap_or(false);
    if !alive || stats.hp > 0 {
        return Ok(Vec::new());
    }

    let location = world.entities.actor_location(entity);
    let kind = world.entities.get(entity).map(|entity| entity.kind());
    let corpse_jackal_data = world.corpse_jackal_data();
    let difficulty = world
        .entities
        .get(entity)
        .and_then(|entity| entity.monster_difficulty())
        .unwrap_or_default();
    if let (Some((level, pos)), Some(EntityKind::Monster(monster_kind))) = (location, kind) {
        if world.campaign.is_none() {
            if monster_kind == MonsterKind::Jackal {
                world
                    .state_mut()
                    .entities
                    .spawn_item_with_data(
                        ItemKind::CorpseJackal,
                        corpse_jackal_data,
                        EntityLocation::OnMap { level, pos },
                    )
                    .map_err(|error| error.to_string())?;
            }
        } else {
            let spec = aihack_content::monster_corpse_spec(monster_kind, world.content_registry());
            let roll = deterministic_corpse_roll(entity, level, pos);
            if roll < spec.chance {
                let (drop_kind, drop_data) = if monster_kind == MonsterKind::Jackal {
                    (ItemKind::CorpseJackal, corpse_jackal_data)
                } else {
                    (ItemKind::Corpse, world.corpse_generic_data())
                };
                let corpse_id = world
                    .state_mut()
                    .entities
                    .spawn_item_with_data(
                        drop_kind,
                        drop_data,
                        EntityLocation::OnMap { level, pos },
                    )
                    .map_err(|error| error.to_string())?;
                let entities = &mut world.state_mut().entities;
                entities.set_item_corpse_monster(corpse_id, Some(monster_kind));
                entities.set_item_corpse_age(corpse_id, 0);
            }
        }
    }
    world.state_mut().entities.set_alive(entity, false);
    if entity == world.player_id {
        world.state_mut().last_death_cause = Some(cause);
    } else {
        let gold_gain = if world.campaign.is_some() {
            u32::from(difficulty).saturating_mul(25)
        } else {
            u32::from(difficulty)
        };
        let state = world.state_mut();
        state.kill_count = state.kill_count.saturating_add(1);
        state.gold = state.gold.saturating_add(gold_gain);
    }
    let mut events = vec![GameEvent::EntityDied { entity, cause }];
    if entity != world.player_id
        && matches!(cause, DeathCause::Combat { attacker } if attacker == world.player_id)
    {
        events.extend(crate::campaign::award_xp(world, difficulty));
    }
    Ok(events)
}

#[cfg(feature = "testing")]
pub fn state_after_deaths(world: &GameWorld) -> RunState {
    aihack_core::death::state_after_death_check(world.player_alive(), world.last_death_cause)
}

pub fn state_after_deaths_at(world: &GameWorld, turn: u64) -> RunState {
    if world.player_alive() {
        RunState::Playing
    } else {
        RunState::GameOver {
            cause: world.last_death_cause.unwrap_or(DeathCause::Combat {
                attacker: EntityId(0),
            }),
            final_score: score::death_score(world, turn),
        }
    }
}

fn deterministic_corpse_roll(
    entity: EntityId,
    level: aihack_core::ids::LevelId,
    pos: aihack_core::position::Pos,
) -> u8 {
    let mut z = u64::from(entity.0)
        ^ ((u64::from(level.depth as u16) << 32)
            | (u64::from(pos.x as u16) << 16)
            | u64::from(pos.y as u16));
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((z ^ (z >> 31)) % 100) as u8
}
