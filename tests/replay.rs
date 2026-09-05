use std::{env, fs};

use aihack::core::{save::ArtifactStore, CommandIntent, ReplayLineV1};

fn temp_store(name: &str) -> (std::path::PathBuf, ArtifactStore) {
    let root = env::temp_dir().join(format!("aihack-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    (root, store)
}

#[test]
fn replay_jsonl_schema_is_stable() {
    let (root, store) = temp_store("replay-schema");
    let path = std::path::Path::new("trace.jsonl");
    let mut session = aihack::core::GameSession::new_for_playing(42);
    let turn_before = session.turn();
    let command = CommandIntent::Wait;
    let outcome = session.submit(command);
    let line = ReplayLineV1 {
        turn_before,
        command,
        snapshot_hash_after: outcome.snapshot_hash.clone(),
        outcome,
    };
    store.append_replay_line(path, &line).unwrap();
    let lines = store.read_replay_lines(path).unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], line);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replay_batch_append_writes_the_trace_with_one_atomic_store_operation() {
    let (root, store) = temp_store("replay-batch");
    let path = std::path::Path::new("trace.jsonl");
    let mut session = aihack::core::GameSession::new_for_playing(42);
    let mut lines = Vec::new();
    for _ in 0..2 {
        let turn_before = session.turn();
        let command = CommandIntent::Wait;
        let outcome = session.submit(command);
        lines.push(ReplayLineV1 {
            turn_before,
            command,
            snapshot_hash_after: outcome.snapshot_hash.clone(),
            outcome,
        });
    }

    store.append_replay_lines(path, &lines).unwrap();

    assert_eq!(store.read_replay_lines(path).unwrap(), lines);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn load_resume_replay_matches_direct_run() {
    let (root, store) = temp_store("replay-resume");
    let save_path = std::path::Path::new("save.json");
    let replay_a = std::path::Path::new("a.jsonl");
    let replay_b = std::path::Path::new("b.jsonl");

    let mut direct = aihack::core::GameSession::new_for_playing(42);
    assert!(direct.submit(CommandIntent::Wait).accepted);
    store.save_session(&direct, save_path).unwrap();
    let mut loaded = store.load_session(save_path).unwrap();

    for _ in 0..3 {
        let turn_a = direct.turn();
        let outcome_a = direct.submit(CommandIntent::Wait);
        store
            .append_replay_line(
                replay_a,
                &ReplayLineV1 {
                    turn_before: turn_a,
                    command: CommandIntent::Wait,
                    snapshot_hash_after: outcome_a.snapshot_hash.clone(),
                    outcome: outcome_a,
                },
            )
            .unwrap();

        let turn_b = loaded.turn();
        let outcome_b = loaded.submit(CommandIntent::Wait);
        store
            .append_replay_line(
                replay_b,
                &ReplayLineV1 {
                    turn_before: turn_b,
                    command: CommandIntent::Wait,
                    snapshot_hash_after: outcome_b.snapshot_hash.clone(),
                    outcome: outcome_b,
                },
            )
            .unwrap();
    }

    let lines_a = store.read_replay_lines(replay_a).unwrap();
    let lines_b = store.read_replay_lines(replay_b).unwrap();
    assert_eq!(lines_a.len(), lines_b.len());
    assert_eq!(
        lines_a.last().unwrap().snapshot_hash_after,
        lines_b.last().unwrap().snapshot_hash_after
    );
    assert_eq!(
        direct.snapshot().stable_hash(),
        loaded.snapshot().stable_hash()
    );

    drop(store);
    fs::remove_dir_all(root).unwrap();
}
