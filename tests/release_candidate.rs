use aihack::core::{CommandIntent, GameSession};

fn run_seed(seed: u64, turns: u64) -> (u64, String) {
    let mut session = GameSession::new_for_playing(seed);
    while session.turn < turns {
        let _ = session.submit(CommandIntent::Wait);
        if matches!(session.state, aihack::core::RunState::GameOver { .. }) {
            break;
        }
    }
    (session.turn, session.snapshot().stable_hash().0)
}

#[test]
fn release_candidate_multiseed_headless_baselines_are_stable() {
    assert_eq!(run_seed(42, 1000), (20, "569bc36895258349".to_string()));
    assert_eq!(run_seed(7, 1000), (28, "f1ee87dc33c32533".to_string()));
    assert_eq!(run_seed(1234, 1000), (18, "58762b2adea01615".to_string()));
}
