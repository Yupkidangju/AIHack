use aihack::core::{
    policy::{run_to_turn, HeadlessPolicy},
    GameSession, RunState,
};

const SEEDS: [u64; 3] = [42, 7, 1234];
const TARGET_TURN: u64 = 1000;

#[test]
fn survival_policy_reaches_one_thousand_accepted_turns_for_required_seeds() {
    for seed in SEEDS {
        let mut session = GameSession::new_for_playing(seed);
        let report = run_to_turn(&mut session, TARGET_TURN, HeadlessPolicy::survival_v1()).unwrap();

        assert_eq!(report.accepted_turns, TARGET_TURN, "seed={seed}");
        assert_eq!(report.final_state, RunState::Playing, "seed={seed}");
        assert!((report.accepted_turns..=report.accepted_turns * 16)
            .contains(&report.submitted_commands));
    }
}

#[test]
fn survival_policy_hash_is_stable_across_three_runs_per_seed() {
    for seed in SEEDS {
        let hashes = (0..3)
            .map(|_| {
                let mut session = GameSession::new_for_playing(seed);
                run_to_turn(&mut session, TARGET_TURN, HeadlessPolicy::survival_v1())
                    .unwrap()
                    .final_hash
            })
            .collect::<Vec<_>>();

        assert!(
            hashes.windows(2).all(|pair| pair[0] == pair[1]),
            "seed={seed}"
        );
    }
}
