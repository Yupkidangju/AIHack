use crate::{
    core::{event::GameEvent, ids::EntityId, session::RunState, world::GameWorld},
    domain::{combat::DeathCause, entity::EntityKind, item::ItemKind, monster::MonsterKind},
};

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
    world.entities.set_alive(entity, false);
    if entity == world.player_id {
        world.last_death_cause = Some(cause);
    } else {
        world.kill_count += 1;
    }
    let events = vec![GameEvent::EntityDied { entity, cause }];
    if let (Some((level, pos)), Some(EntityKind::Monster(MonsterKind::Jackal))) = (location, kind) {
        world.entities.spawn_item(
            ItemKind::CorpseJackal,
            crate::domain::entity::EntityLocation::OnMap { level, pos },
        );
    }
    events
}

/// [v0.2.0] Phase 16: player 사망 시 GameOver { cause, final_score }를 반환한다.
/// final_score는 0으로 초기화되며, session에서 최종 계산하여 갱신한다.
pub fn state_after_deaths(world: &GameWorld) -> RunState {
    if world.player_alive() {
        RunState::Playing
    } else {
        let cause = world.last_death_cause.unwrap_or(DeathCause::Combat {
            attacker: EntityId(0),
        });
        RunState::GameOver {
            cause,
            final_score: 0,
        }
    }
}
