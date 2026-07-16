use aihack_core::{
    death::state_after_death_check, domain::combat::DeathCause, ids::EntityId, run_state::RunState,
};

#[test]
fn death_state_uses_recorded_cause_or_the_stable_default() {
    assert_eq!(state_after_death_check(true, None), RunState::Playing);
    assert_eq!(
        state_after_death_check(
            false,
            Some(DeathCause::Combat {
                attacker: EntityId(4)
            })
        ),
        RunState::GameOver {
            cause: DeathCause::Combat {
                attacker: EntityId(4)
            },
            final_score: 0,
        }
    );
    assert_eq!(
        state_after_death_check(false, None),
        RunState::GameOver {
            cause: DeathCause::Combat {
                attacker: EntityId(0)
            },
            final_score: 0,
        }
    );
}
