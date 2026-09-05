use aihack::core::{
    policy::{run_replay_to_turn, run_to_turn, run_to_turn_with_trace, HeadlessPolicy},
    CommandIntent, Direction, GameSession, SaveDataV1,
};

fn restored_with_status(paralysis_turns: u8, prayer_cooldown: u16) -> GameSession {
    let mut save = GameSession::new_for_playing(42).to_save_data();
    save.world.paralysis_turns = paralysis_turns;
    save.world.prayer_cooldown = prayer_cooldown;
    // 실제 저장 데이터 검증과 복원 경계를 거친 상태에서 관찰·실행을 비교한다.
    let json = serde_json::to_string(&save).unwrap();
    let decoded: SaveDataV1 = serde_json::from_str(&json).unwrap();
    GameSession::from_save_data(decoded).unwrap()
}

fn restored_at_turn_two() -> GameSession {
    let mut source = GameSession::new_for_playing(42);
    run_to_turn(&mut source, 2, HeadlessPolicy::wait_v1()).unwrap();
    GameSession::from_save_data(source.to_save_data()).unwrap()
}

#[test]
fn paralysis_action_space_contains_only_commands_accepted_by_submit() {
    let session = restored_with_status(1, 0);
    let observation = session.observation();
    assert!(observation.legal_actions.contains(&CommandIntent::Wait));
    for command in observation.legal_actions {
        let mut probe = session.clone();
        assert!(
            probe.submit(command).accepted,
            "advertised illegal command: {command:?}"
        );
    }
    for action in observation.action_space.commands {
        let aihack::core::ActionIntent::Command(command) = action else {
            continue;
        };
        let mut probe = session.clone();
        assert!(
            probe.submit(command).accepted,
            "action-space rejected: {command:?}"
        );
    }
}

#[test]
fn survival_runner_waits_through_restored_paralysis_and_reaches_target() {
    let mut session = restored_with_status(1, 0);
    let (report, trace) =
        run_to_turn_with_trace(&mut session, 1, HeadlessPolicy::survival_v1()).unwrap();
    assert_eq!(report.accepted_turns, 1);
    assert_eq!(session.turn(), 1);
    assert_eq!(session.observation().player.paralysis_turns, 0);
    assert!(trace
        .iter()
        .any(|line| line.command == CommandIntent::Wait && line.outcome.turn_advanced));
}

#[test]
fn survival_runner_falls_back_to_wait_after_a_stale_movement_candidate() {
    struct StaleObservationClient(GameSession);

    impl aihack_runtime::GameClient for StaleObservationClient {
        fn observation(&self) -> aihack_ai_contract::Observation {
            let mut observation = self.0.observation();
            // 관찰과 제출 사이에 상태가 바뀐 adapter를 재현한다. 실행은 실제 마비 guard를 거친다.
            observation
                .legal_actions
                .push(CommandIntent::Move(Direction::East));
            observation
        }

        fn revision(&self) -> aihack_ai_contract::ClientRevision {
            aihack_runtime::GameClient::revision(&self.0)
        }

        fn run_state(&self) -> aihack::core::RunState {
            self.0.run_state()
        }

        fn submit(&mut self, intent: CommandIntent) -> aihack::core::TurnOutcome {
            self.0.submit(intent)
        }
    }

    let mut client = StaleObservationClient(restored_with_status(1, 0));
    let (report, trace) =
        run_to_turn_with_trace(&mut client, 1, HeadlessPolicy::survival_v1()).unwrap();
    assert_eq!(report.accepted_turns, 1);
    assert_eq!(report.submitted_commands, 2);
    assert!(!trace[0].outcome.accepted);
    assert_eq!(trace[1].command, CommandIntent::Wait);
    assert!(trace[1].outcome.turn_advanced);
}

#[test]
fn prayer_cooldown_agrees_with_observation_and_recovers_after_wait() {
    let mut session = restored_with_status(0, 1);
    let before = session.to_save_data();
    assert!(!session.submit(CommandIntent::Pray).accepted);
    assert_eq!(session.to_save_data(), before);
    assert!(!session
        .observation()
        .legal_actions
        .contains(&CommandIntent::Pray));
    assert!(session.submit(CommandIntent::Wait).accepted);
    assert_eq!(session.observation().player.prayer_cooldown, 0);
    assert!(session
        .observation()
        .legal_actions
        .contains(&CommandIntent::Pray));
    assert!(session.submit(CommandIntent::Pray).accepted);
}

#[test]
fn doorless_kick_is_not_advertised_as_legal() {
    let session = GameSession::new_for_playing(42);
    for direction in Direction::ALL {
        let command = CommandIntent::Kick(direction);
        let mut probe = session.clone();
        let accepted = probe.submit(command).accepted;
        assert_eq!(
            session.observation().legal_actions.contains(&command),
            accepted,
            "kick legality differs from submit for {direction:?}"
        );
    }
}

#[test]
fn loaded_lower_target_is_rejected_without_mutation_or_submission() {
    for policy in [HeadlessPolicy::wait_v1(), HeadlessPolicy::survival_v1()] {
        let mut session = restored_at_turn_two();
        let before = session.to_save_data();
        let result = run_to_turn_with_trace(&mut session, 1, policy);
        assert_eq!(session.to_save_data(), before);
        let error = result.expect_err("a lower absolute target must be rejected");
        assert_eq!(error.submitted_commands(), 0);
    }
}

#[test]
fn loaded_equal_and_higher_targets_report_exact_progress() {
    let mut session = restored_at_turn_two();
    let before = session.to_save_data();
    let equal = run_to_turn(&mut session, 2, HeadlessPolicy::wait_v1()).unwrap();
    assert_eq!(equal.accepted_turns, 0);
    assert_eq!(equal.submitted_commands, 0);
    assert_eq!(session.to_save_data(), before);
    let higher = run_to_turn(&mut session, 3, HeadlessPolicy::wait_v1()).unwrap();
    assert_eq!(higher.accepted_turns, 1);
    assert_eq!(higher.submitted_commands, 1);
    assert_eq!(session.turn(), 3);
}

#[test]
fn replay_lower_target_is_rejected_without_mutating_session_or_input() {
    let mut source = restored_at_turn_two();
    let (_, trace) = run_to_turn_with_trace(&mut source, 3, HeadlessPolicy::wait_v1()).unwrap();
    let original_trace = trace.clone();
    let mut replayed = restored_at_turn_two();
    let before = replayed.to_save_data();
    let result = run_replay_to_turn(&mut replayed, 1, &trace);
    assert_eq!(replayed.to_save_data(), before);
    assert_eq!(trace, original_trace);
    assert_eq!(
        result
            .expect_err("replay lower target must be rejected")
            .submitted_commands(),
        0
    );
}

#[test]
fn replay_equal_and_higher_targets_preserve_input_and_match_source() {
    let mut source = restored_at_turn_two();
    let (_, trace) = run_to_turn_with_trace(&mut source, 3, HeadlessPolicy::wait_v1()).unwrap();
    let original_trace = trace.clone();
    let mut replayed = restored_at_turn_two();
    let before = replayed.to_save_data();
    let equal = run_replay_to_turn(&mut replayed, 2, &trace).unwrap();
    assert_eq!(equal.accepted_turns, 0);
    assert_eq!(equal.submitted_commands, 0);
    assert_eq!(replayed.to_save_data(), before);
    let higher = run_replay_to_turn(&mut replayed, 3, &trace).unwrap();
    assert_eq!(higher.accepted_turns, 1);
    assert_eq!(higher.submitted_commands, 1);
    assert_eq!(replayed.to_save_data(), source.to_save_data());
    assert_eq!(trace, original_trace);
}
