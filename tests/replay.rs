use std::{env, fs};

use aihack::core::{save, CommandIntent, ReplayLineV1};

fn temp_path(name: &str) -> std::path::PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("aihack-{name}-{}.jsonl", std::process::id()));
    path
}

#[test]
fn replay_jsonl_schema_is_stable() {
    let path = temp_path("replay-schema");
    let mut session = aihack::core::GameSession::new_for_playing(42);
    let turn_before = session.turn;
    let command = CommandIntent::Wait;
    let outcome = session.submit(command);
    let line = ReplayLineV1 {
        turn_before,
        command,
        snapshot_hash_after: outcome.snapshot_hash.clone(),
        outcome,
    };
    save::append_replay_line(&path, &line).unwrap();
    let lines = save::read_replay_lines(&path).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], line);
    let _ = fs::remove_file(path);
}

#[test]
fn load_resume_replay_matches_direct_run() {
    let save_path = temp_path("replay-save");
    let replay_a = temp_path("replay-a");
    let replay_b = temp_path("replay-b");

    let mut direct = aihack::core::GameSession::new_for_playing(42);
    assert!(direct.submit(CommandIntent::Wait).accepted);
    save::save_session_to_path(&direct, &save_path).unwrap();
    let mut loaded = save::load_session_from_path(&save_path).unwrap();

    for _ in 0..3 {
        let turn_a = direct.turn;
        let outcome_a = direct.submit(CommandIntent::Wait);
        save::append_replay_line(
            &replay_a,
            &ReplayLineV1 {
                turn_before: turn_a,
                command: CommandIntent::Wait,
                snapshot_hash_after: outcome_a.snapshot_hash.clone(),
                outcome: outcome_a,
            },
        )
        .unwrap();

        let turn_b = loaded.turn;
        let outcome_b = loaded.submit(CommandIntent::Wait);
        save::append_replay_line(
            &replay_b,
            &ReplayLineV1 {
                turn_before: turn_b,
                command: CommandIntent::Wait,
                snapshot_hash_after: outcome_b.snapshot_hash.clone(),
                outcome: outcome_b,
            },
        )
        .unwrap();
    }

    let lines_a = save::read_replay_lines(&replay_a).unwrap();
    let lines_b = save::read_replay_lines(&replay_b).unwrap();
    assert_eq!(lines_a.len(), lines_b.len());
    assert_eq!(
        lines_a.last().unwrap().snapshot_hash_after,
        lines_b.last().unwrap().snapshot_hash_after
    );
    assert_eq!(
        direct.snapshot().stable_hash(),
        loaded.snapshot().stable_hash()
    );

    let _ = fs::remove_file(save_path);
    let _ = fs::remove_file(replay_a);
    let _ = fs::remove_file(replay_b);
}
