use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_runtime::GameSession;
use std::process::Command;

#[test]
fn headless_package_runs_through_the_game_client_contract() {
    let mut session = GameSession::new_for_playing(42);

    let report = run_to_turn(&mut session, 1, HeadlessPolicy::wait_v1()).unwrap();

    assert_eq!(report.accepted_turns, 1);
    assert_eq!(report.final_hash.0, "54e43384cefa2590");
}

#[test]
fn headless_help_uses_current_product_description() {
    let output = Command::new(env!("CARGO_BIN_EXE_aihack-headless"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("결정론적 headless runner"));
    assert!(!stdout.contains("v0.1.0"));
    assert!(!stdout.contains("Phase"));
}
