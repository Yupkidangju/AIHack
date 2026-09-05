use aihack_ai_contract::CommandIntent;
use aihack_runtime::{
    save::{ArtifactStore, ReplayLineV1},
    GameSession,
};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

struct FixtureRoot(std::path::PathBuf);
impl FixtureRoot {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "aihack-target-exit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        // 이 테스트가 새로 만든 정확한 임시 root만 정리한다.
        if self.0.parent() == std::env::temp_dir().canonicalize().ok().as_deref()
            && self.0.canonicalize().ok().as_ref() == Some(&self.0)
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn run(root: &Path, policy: &str, target: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aihack-headless"));
    command.current_dir(root).args([
        "--load",
        "start.json",
        "--policy",
        policy,
        "--turns",
        target,
        "--report",
        "result.json",
        "--save",
        "output.json",
        "--replay-out",
        "output.jsonl",
    ]);
    if policy == "replay-file" {
        command.args(["--replay-in", "input.jsonl"]);
    }
    command.output().unwrap()
}

#[test]
fn actual_cli_lower_equal_higher_targets_preserve_inputs_and_map_exit_codes() {
    let root = FixtureRoot::new();
    let runtime = root.path().join("runtime");
    let store = ArtifactStore::open(&runtime).unwrap();
    let mut session = GameSession::new_for_playing(42);
    for _ in 0..2 {
        assert!(session.submit(CommandIntent::Wait).accepted);
    }
    store
        .save_session(&session, Path::new("start.json"))
        .unwrap();
    let initial_hash = session.snapshot().stable_hash().0;
    let outcome = session.submit(CommandIntent::Wait);
    let line = ReplayLineV1 {
        turn_before: 2,
        command: CommandIntent::Wait,
        snapshot_hash_after: outcome.snapshot_hash.clone(),
        outcome,
    };
    store
        .append_replay_lines(Path::new("input.jsonl"), &[line])
        .unwrap();
    let save_before = fs::read(runtime.join("start.json")).unwrap();
    let replay_before = fs::read(runtime.join("input.jsonl")).unwrap();

    for policy in ["wait-v1", "survival-v1", "replay-file"] {
        for target in ["1", "2", "3"] {
            fs::write(runtime.join("output.json"), b"save sentinel").unwrap();
            fs::write(runtime.join("output.jsonl"), b"").unwrap();
            let output = run(root.path(), policy, target);
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(runtime.join("result.json")).unwrap()).unwrap();
            assert_eq!(fs::read(runtime.join("start.json")).unwrap(), save_before);
            assert_eq!(
                fs::read(runtime.join("input.jsonl")).unwrap(),
                replay_before
            );
            assert_eq!(
                output.status.code(),
                Some(if target == "1" { 2 } else { 0 }),
                "{policy} target={target}, stderr={}",
                String::from_utf8_lossy(&output.stderr)
            );
            if target == "1" {
                assert!(output.stdout.is_empty());
                assert!(String::from_utf8_lossy(&output.stderr)
                    .contains("target turn 1 is before loaded turn 2"));
                assert_eq!(report["error"]["TargetBeforeCurrent"]["turn"], 2);
                assert_eq!(report["accepted_turns"], 0);
                assert_eq!(report["submitted_commands"], 0);
                assert_eq!(report["final_hash"], initial_hash);
                assert_eq!(
                    fs::read(runtime.join("output.json")).unwrap(),
                    b"save sentinel"
                );
                assert!(fs::read(runtime.join("output.jsonl")).unwrap().is_empty());
            } else {
                assert!(output.stderr.is_empty());
                assert_eq!(report["accepted_turns"], if target == "2" { 0 } else { 1 });
                assert_eq!(
                    report["submitted_commands"],
                    if target == "2" { 0 } else { 1 }
                );
                let saved = store.load_session(Path::new("output.json")).unwrap();
                assert_eq!(saved.turn(), target.parse::<u64>().unwrap());
                if target == "2" {
                    assert_eq!(report["final_hash"], initial_hash);
                }
            }
        }
    }
}

#[test]
fn actual_cli_replay_progress_failure_remains_exit_one() {
    let root = FixtureRoot::new();
    let runtime = root.path().join("runtime");
    let store = ArtifactStore::open(&runtime).unwrap();
    let mut session = GameSession::new_for_playing(42);
    for _ in 0..2 {
        session.submit(CommandIntent::Wait);
    }
    store
        .save_session(&session, Path::new("start.json"))
        .unwrap();
    store.write_atomic(Path::new("input.jsonl"), b"").unwrap();
    let output = run(root.path(), "replay-file", "3");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("replay ended before target"));
    assert!(!runtime.join("output.json").exists());
}
