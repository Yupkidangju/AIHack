use aihack::core::{error::GameError, CommandIntent, GameSession};

fn save_with_inventory_owner(owner: u32) -> aihack::core::SaveDataV1 {
    let save = GameSession::new_for_playing(42).to_save_data();
    let mut encoded = serde_json::to_value(save).unwrap();
    encoded["world"]["inventory"]["owner"] = serde_json::json!(owner);
    serde_json::from_value(encoded).unwrap()
}

#[test]
fn invalid_persisted_invariant_is_rejected_before_session_creation() {
    assert!(matches!(
        GameSession::from_save_data(save_with_inventory_owner(2)),
        Err(GameError::InvalidSave(_))
    ));
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
fn invalid_persisted_invariant_cannot_materialize_an_rng() {
    let result = GameSession::from_save_data(save_with_inventory_owner(2));
    assert!(matches!(result, Err(GameError::InvalidSave(_))));
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
