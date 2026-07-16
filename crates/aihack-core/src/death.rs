use crate::{domain::combat::DeathCause, ids::EntityId, run_state::RunState};

/// 사망 확인 뒤의 run state를 결정한다. entity mutation과 corpse 생성은 adapter가 맡는다.
pub fn state_after_death_check(player_alive: bool, cause: Option<DeathCause>) -> RunState {
    if player_alive {
        RunState::Playing
    } else {
        RunState::GameOver {
            cause: cause.unwrap_or(DeathCause::Combat {
                attacker: EntityId(0),
            }),
            final_score: 0,
        }
    }
}
