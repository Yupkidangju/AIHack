use crate::{
    core::{
        event::GameEvent,
        ids::EntityId,
        position::{Direction, Pos},
        rng::GameRng,
        session::RunState,
        world::GameWorld,
    },
    domain::monster::MonsterAiKind,
    systems::{combat, death, movement, vision},
};

/// [v0.1.0] Phase 6 몬스터 턴에서 actor별 행동 계획을 분리해 결정론을 유지한다.
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
}

/// [v0.1.0] 수집 단계와 적용 단계를 분리하기 위한 최소 계획 단위다.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MonsterTurnPlan {
    pub intents: Vec<MonsterIntent>,
}

pub fn run_monster_turn(
    world: &mut GameWorld,
    rng: &mut GameRng,
    state: &mut RunState,
) -> Vec<GameEvent> {
    let plan = collect_monster_turn(world, rng);
    let events = apply_monster_turn(world, rng, &plan);
    *state = death::state_after_deaths(world);
    events
}

pub fn collect_monster_turn(world: &GameWorld, rng: &mut GameRng) -> MonsterTurnPlan {
    MonsterTurnPlan {
        intents: world
            .current_level_hostile_monsters()
            .into_iter()
            .map(|actor| decide_monster_intent(world, rng, actor))
            .collect(),
    }
}

pub fn apply_monster_turn(
    world: &mut GameWorld,
    rng: &mut GameRng,
    plan: &MonsterTurnPlan,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    for intent in &plan.intents {
        if !world.player_alive() {
            break;
        }

        match *intent {
            MonsterIntent::Wait => {}
            MonsterIntent::Move { entity, direction } => {
                let Some((_, from)) = world.entities.actor_location(entity) else {
                    continue;
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
                    continue;
                }
                let Some(resolution) = combat::resolve_attack(world, rng, attacker, defender)
                else {
                    continue;
                };
                events.push(combat::attack_event(&resolution));
                events.extend(death::collect_death_events_after_attack(
                    world, attacker, defender,
                ));
            }
        }
    }

    events
}

fn decide_monster_intent(world: &GameWorld, rng: &mut GameRng, actor: EntityId) -> MonsterIntent {
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
        MonsterAiKind::Stationary => MonsterIntent::Wait,
    }
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

    let current_distance = (
        actor_pos.chebyshev_distance(player_pos),
        manhattan_distance(actor_pos, player_pos),
    );
    for direction in Direction::ALL {
        if !movement::is_passable_for_actor(world, actor, direction) {
            continue;
        }
        let to = actor_pos.offset(direction.delta());
        let next_distance = (
            to.chebyshev_distance(player_pos),
            manhattan_distance(to, player_pos),
        );
        if next_distance < current_distance {
            return MonsterIntent::Move {
                entity: actor,
                direction,
            };
        }
    }
    MonsterIntent::Wait
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

fn manhattan_distance(from: Pos, to: Pos) -> i16 {
    let delta = from.delta_to(to);
    delta.dx.abs() + delta.dy.abs()
}
