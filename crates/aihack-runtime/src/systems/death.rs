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

use crate::{domain::item::item_data, systems::score, world::GameWorld};

pub fn collect_death_events_after_attack(
    world: &mut GameWorld,
    attacker: EntityId,
    defender: EntityId,
) -> Vec<GameEvent> {
    collect_death_events_if_hp_depleted(world, defender, DeathCause::Combat { attacker })
}

pub fn collect_death_events_if_hp_depleted(
    world: &mut GameWorld,
    entity: EntityId,
    cause: DeathCause,
) -> Vec<GameEvent> {
    let Some(stats) = world.entities.actor_stats(entity).copied() else {
        return Vec::new();
    };
    let alive = world
        .entities
        .get(entity)
        .and_then(|entity| entity.actor().map(|(_, _, _, _, _, alive)| alive))
        .unwrap_or(false);
    if !alive || stats.hp > 0 {
        return Vec::new();
    }

    let location = world.entities.actor_location(entity);
    let kind = world.entities.get(entity).map(|entity| entity.kind());
    let difficulty = world
        .entities
        .get(entity)
        .and_then(|entity| entity.monster_difficulty())
        .unwrap_or_default();
    world.entities.set_alive(entity, false);
    if entity == world.player_id {
        world.last_death_cause = Some(cause);
    } else {
        world.kill_count += 1;
        world.gold = world.gold.saturating_add(u32::from(difficulty));
    }
    let events = vec![GameEvent::EntityDied { entity, cause }];
    if let (Some((level, pos)), Some(EntityKind::Monster(MonsterKind::Jackal))) = (location, kind) {
        world.entities.spawn_item_with_data(
            ItemKind::CorpseJackal,
            item_data(ItemKind::CorpseJackal),
            EntityLocation::OnMap { level, pos },
        );
    }
    events
}

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
