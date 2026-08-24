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
    if let (Some((level, pos)), Some(EntityKind::Monster(MonsterKind::Jackal))) = (location, kind) {
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
    world.state_mut().entities.set_alive(entity, false);
    if entity == world.player_id {
        world.state_mut().last_death_cause = Some(cause);
    } else {
        let state = world.state_mut();
        state.kill_count = state.kill_count.saturating_add(1);
        state.gold = state.gold.saturating_add(u32::from(difficulty));
    }
    Ok(vec![GameEvent::EntityDied { entity, cause }])
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
