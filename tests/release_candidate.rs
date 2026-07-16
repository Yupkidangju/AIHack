use aihack::core::{
    policy::{run_to_turn, HeadlessPolicy},
    GameSession, RunState,
};

#[test]
fn release_candidate_multiseed_survival_runs_reach_target_turn() {
    for seed in [42, 7, 1234] {
        let mut session = GameSession::new_for_playing(seed);
        let report = run_to_turn(&mut session, 1000, HeadlessPolicy::survival_v1()).unwrap();
        assert_eq!(report.accepted_turns, 1000, "seed {seed}");
        assert_eq!(report.final_state, RunState::Playing, "seed {seed}");
    }
}
