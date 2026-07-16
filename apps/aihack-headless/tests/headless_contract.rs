use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_runtime::GameSession;

#[test]
fn headless_package_runs_through_the_game_client_contract() {
    let mut session = GameSession::new_for_playing(42);

    let report = run_to_turn(&mut session, 1, HeadlessPolicy::wait_v1()).unwrap();

    assert_eq!(report.accepted_turns, 1);
    assert_eq!(report.final_hash.0, "54e43384cefa2590");
}
