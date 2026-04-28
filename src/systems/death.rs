use crate::{
    core::{event::GameEvent, ids::EntityId, session::RunState, world::GameWorld},
    domain::combat::DeathCause,
};

pub fn collect_death_events_after_attack(
    world: &mut GameWorld,
    attacker: EntityId,
    defender: EntityId,
) -> Vec<GameEvent> {
    let Some(stats) = world.entities.actor_stats(defender).copied() else {
        return Vec::new();
    };
    let alive = world
        .entities
        .get(defender)
        .and_then(|entity| entity.actor().map(|(_, _, _, _, _, alive)| alive))
        .unwrap_or(false);
    if !alive || stats.hp > 0 {
        return Vec::new();
    }

    world.entities.set_alive(defender, false);
    vec![GameEvent::EntityDied {
        entity: defender,
        cause: DeathCause::Combat { attacker },
    }]
}

pub fn state_after_deaths(world: &GameWorld) -> RunState {
    if world.player_alive() {
        RunState::Playing
    } else {
        RunState::GameOver
    }
}
