//! 호환성 record가 사용하는 opt-in 비원자 fixture 경계다.

use aihack_core::{event::GameEvent, ids::EntityId, run_state::RunState};

use crate::{systems::death, world::SavedWorldV1, GameError};

/// HP 0 precondition의 death event/state를 compatibility record용으로 투영한다.
///
/// 이 helper는 `testing` feature에서만 존재하며 production mutation API가 아니다.
pub fn resolve_depleted_death(
    saved: SavedWorldV1,
    attacker: EntityId,
    defender: EntityId,
) -> Result<(Vec<GameEvent>, RunState), GameError> {
    let mut world = crate::world::GameWorld::from_depleted_saved_world(saved)?;
    let events = death::collect_death_events_after_attack(&mut world, attacker, defender)
        .map_err(GameError::CommandRejected)?;
    let state = death::state_after_deaths(&world);
    Ok((events, state))
}
