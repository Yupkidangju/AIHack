use aihack::{
    core::{
        policy::{
            run_replay_to_turn, run_to_turn, run_to_turn_with_trace, HeadlessPolicy,
            HeadlessRunError, ReplayMismatchField,
        },
        CommandIntent, Direction, GameSession, RunState, SnapshotHash,
    },
    testing::SessionBuilder,
};

#[test]
fn survival_policy_prefers_an_adjacent_hostile_bump_attack() {
    let session = GameSession::new_for_playing(42);

    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&session),
        vec![CommandIntent::Move(Direction::East)]
    );
}

#[test]
fn runner_uses_an_absolute_turn_target_and_reports_accepted_commands() {
    let mut session = GameSession::new_for_playing(42);

    let report = run_to_turn(&mut session, 1, HeadlessPolicy::wait_v1()).unwrap();

    assert_eq!(report.requested_turns, 1);
    assert_eq!(report.accepted_turns, 1);
    assert_eq!(report.submitted_commands, 1);
    assert_eq!(session.turn(), 1);
}

#[test]
fn runner_trace_contains_each_submitted_command_for_replay_output() {
    let mut session = GameSession::new_for_playing(42);

    let (_, trace) = run_to_turn_with_trace(&mut session, 1, HeadlessPolicy::wait_v1()).unwrap();

    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].turn_before, 0);
    assert_eq!(trace[0].command, CommandIntent::Wait);
}

#[test]
fn replay_runner_replays_recorded_commands_to_the_absolute_target() {
    let mut source = GameSession::new_for_playing(42);
    let (_, trace) = run_to_turn_with_trace(&mut source, 1, HeadlessPolicy::wait_v1()).unwrap();
    let mut replayed = GameSession::new_for_playing(42);

    let report = run_replay_to_turn(&mut replayed, 1, &trace).unwrap();

    assert_eq!(report.accepted_turns, 1);
    assert_eq!(
        replayed.snapshot().stable_hash(),
        source.snapshot().stable_hash()
    );
}

#[test]
fn replay_runner_rejects_each_tampered_integrity_field_without_partial_commit() {
    type ReplayMutation = Box<dyn Fn(&mut aihack::core::ReplayLineV1)>;
    type ReplayCase = (ReplayMismatchField, ReplayMutation);

    let mut source = GameSession::new_for_playing(42);
    let (_, trace) = run_to_turn_with_trace(&mut source, 1, HeadlessPolicy::wait_v1()).unwrap();
    let cases: Vec<ReplayCase> = vec![
        (
            ReplayMismatchField::TurnBefore,
            Box::new(|line| line.turn_before = 999),
        ),
        (
            ReplayMismatchField::Accepted,
            Box::new(|line| line.outcome.accepted = false),
        ),
        (
            ReplayMismatchField::TurnAdvanced,
            Box::new(|line| line.outcome.turn_advanced = false),
        ),
        (
            ReplayMismatchField::Events,
            Box::new(|line| line.outcome.events.clear()),
        ),
        (
            ReplayMismatchField::OutcomeSnapshotHash,
            Box::new(|line| line.outcome.snapshot_hash = SnapshotHash("forged-inner".into())),
        ),
        (
            ReplayMismatchField::NextState,
            Box::new(|line| line.outcome.next_state = RunState::Title),
        ),
        (
            ReplayMismatchField::SnapshotHashAfter,
            Box::new(|line| line.snapshot_hash_after = SnapshotHash("forged-outer".into())),
        ),
    ];

    for (expected_field, mutate) in cases {
        let mut tampered = trace.clone();
        mutate(&mut tampered[0]);
        let mut replayed = GameSession::new_for_playing(42);
        let before = replayed.snapshot().stable_hash();

        let error = run_replay_to_turn(&mut replayed, 1, &tampered).unwrap_err();

        assert!(matches!(
            error,
            HeadlessRunError::ReplayMismatch {
                line: 1,
                field,
                ..
            } if field == expected_field
        ));
        assert_eq!(replayed.turn(), 0);
        assert_eq!(replayed.snapshot().stable_hash(), before);
    }
}

#[test]
fn replay_exhaustion_does_not_commit_a_valid_prefix() {
    let mut source = GameSession::new_for_playing(42);
    let (_, trace) = run_to_turn_with_trace(&mut source, 1, HeadlessPolicy::wait_v1()).unwrap();
    let mut replayed = GameSession::new_for_playing(42);
    let before = replayed.snapshot().stable_hash();

    assert!(matches!(
        run_replay_to_turn(&mut replayed, 2, &trace),
        Err(HeadlessRunError::ReplayExhausted { .. })
    ));
    assert_eq!(replayed.turn(), 0);
    assert_eq!(replayed.snapshot().stable_hash(), before);
}

#[test]
fn survival_policy_quaffs_a_legal_healing_potion_when_health_is_low() {
    let mut session = SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.set_actor_location(
                world.player_id,
                aihack::core::LevelId::main(1),
                aihack::core::Pos { x: 8, y: 5 },
            );
        })
        .build();
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let potion = session
        .observation()
        .inventory
        .iter()
        .find(|item| item.kind == aihack::domain::item::ItemKind::PotionHealing)
        .unwrap()
        .item;
    SessionBuilder::mutate(&mut session, |world| {
        let saved = world.saved();
        saved.entities.actor_stats_mut(saved.player_id).unwrap().hp = 8;
    });

    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&session),
        vec![CommandIntent::Quaff { item: potion }]
    );
}
