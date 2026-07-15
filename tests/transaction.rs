use aihack::core::{CommandIntent, GameSession, RunState};

fn save_with_inventory_owner(owner: u32) -> aihack::core::SaveDataV1 {
    let save = GameSession::new_for_playing(42).to_save_data();
    let mut encoded = serde_json::to_value(save).unwrap();
    encoded["world"]["inventory"]["owner"] = serde_json::json!(owner);
    serde_json::from_value(encoded).unwrap()
}

#[test]
fn invariant_failure_rejects_without_committing_world_or_turn() {
    let mut session = GameSession::from_save_data(save_with_inventory_owner(2)).unwrap();
    let before_hash = session.snapshot().stable_hash();
    let before_turn = session.turn();

    let outcome = session.submit(CommandIntent::Wait);

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.turn(), before_turn);
    assert_eq!(session.snapshot().stable_hash(), before_hash);
    assert_eq!(session.run_state(), RunState::Playing);
}

#[test]
fn rejected_command_preserves_the_following_deterministic_turn() {
    let mut baseline = GameSession::new_for_playing(42);
    let mut candidate = GameSession::new_for_playing(42);

    let rejected = candidate.submit(CommandIntent::Open(aihack::core::Direction::West));
    assert!(!rejected.accepted);

    let expected = baseline.submit(CommandIntent::Wait);
    let actual = candidate.submit(CommandIntent::Wait);

    assert_eq!(actual.snapshot_hash, expected.snapshot_hash);
}

#[test]
fn invariant_failure_discards_rng_draws_from_the_working_copy() {
    let mut session = GameSession::from_save_data(save_with_inventory_owner(2)).unwrap();
    let before_rng = session.to_save_data().rng_state;

    let outcome = session.submit(CommandIntent::Move(aihack::core::Direction::East));

    assert!(!outcome.accepted);
    assert_eq!(session.to_save_data().rng_state, before_rng);
}

#[test]
fn accepted_turns_leave_a_six_check_valid_invariant_report() {
    let mut session = GameSession::new_for_playing(42);

    let outcome = session.submit(CommandIntent::Wait);
    let report = session.world().validate_invariants();

    assert!(outcome.accepted);
    assert_eq!(report.checked, 6);
    assert!(report.is_valid());
}
