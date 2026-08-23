use aihack_ai_contract::CommandIntent;
use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_runtime::{save::ReplayLineV1, GameSession};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);
type SaveMutator = Box<dyn FnOnce(&mut serde_json::Value)>;

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "aihack-headless-{label}-{}-{}",
        std::process::id(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

fn replay_fixture() -> String {
    let mut session = GameSession::new_for_playing(42);
    let command = CommandIntent::Wait;
    let outcome = session.submit(command);
    serde_json::to_string(&ReplayLineV1 {
        turn_before: 0,
        command,
        snapshot_hash_after: outcome.snapshot_hash.clone(),
        outcome,
    })
    .unwrap()
        + "\n"
}

fn run_headless_with_malformed_save(
    root: &std::path::Path,
    name: &str,
    mutator: impl FnOnce(&mut serde_json::Value),
) -> std::process::Output {
    fs::create_dir_all(root.join("runtime/saves")).unwrap();
    let mut value = serde_json::to_value(GameSession::new_for_playing(42).to_save_data()).unwrap();
    mutator(&mut value);
    fs::write(
        root.join("runtime/saves").join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_aihack-headless"))
        .args([
            "--load",
            &format!("saves/{name}"),
            "--turns",
            "1",
            "--policy",
            "wait-v1",
        ])
        .current_dir(root)
        .output()
        .unwrap()
}

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

#[test]
fn replay_input_and_curdir_output_alias_are_rejected_without_mutating_input() {
    let root = fixture_root("curdir-alias");
    fs::create_dir_all(root.join("runtime/replays")).unwrap();
    let replay = root.join("runtime/replays/run.jsonl");
    fs::write(&replay, replay_fixture()).unwrap();
    let before = fs::read(&replay).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_aihack-headless"))
        .args([
            "--seed",
            "42",
            "--turns",
            "1",
            "--policy",
            "replay-file",
            "--replay-in",
            "replays/run.jsonl",
            "--replay-out",
            "./replays/run.jsonl",
        ])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read(&replay).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn production_headless_rejects_consumer_unsafe_malformed_saves_before_running() {
    let root = fixture_root("malformed-consumer");
    let cases: [(&str, SaveMutator); 4] = [
        (
            "unequipped-ac.json",
            Box::new(|value| {
                value["world"]["entities"]["entities"][0]["payload"]["Actor"]["stats"]["ac"] =
                    serde_json::json!(-1);
            }),
        ),
        (
            "max-turn.json",
            Box::new(|value| value["turn"] = serde_json::json!(u64::MAX)),
        ),
        (
            "max-kills.json",
            Box::new(|value| value["world"]["kill_count"] = serde_json::json!(u32::MAX)),
        ),
        (
            "forged-item.json",
            Box::new(|value| {
                value["world"]["entities"]["entities"][4]["payload"]["Item"]["data"]
                    ["base_price"] = serde_json::json!(u32::MAX);
            }),
        ),
    ];

    for (name, mutator) in cases {
        let output = run_headless_with_malformed_save(&root, name, mutator);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{name} stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn replay_input_and_case_variant_output_are_rejected_without_mutating_input() {
    let root = fixture_root("case-alias");
    fs::create_dir_all(root.join("runtime/replays")).unwrap();
    let replay = root.join("runtime/replays/run.jsonl");
    fs::write(&replay, replay_fixture()).unwrap();
    let before = fs::read(&replay).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_aihack-headless"))
        .args([
            "--seed",
            "42",
            "--turns",
            "1",
            "--policy",
            "replay-file",
            "--replay-in",
            "replays/run.jsonl",
            "--replay-out",
            "REPLAYS/RUN.JSONL",
        ])
        .current_dir(&root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read(&replay).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn replay_windows_name_aliases_are_rejected_without_mutating_input() {
    for alias in [
        "replays/run.jsonl.",
        "replays/run.jsonl..",
        "replays/run.jsonl ",
        "replays/CON",
        "replays/con.trace",
        "replays/COM1.jsonl",
        "replays/LPT9",
        "replays/CONIN$",
    ] {
        let root = fixture_root("windows-name-alias");
        fs::create_dir_all(root.join("runtime/replays")).unwrap();
        let replay = root.join("runtime/replays/run.jsonl");
        fs::write(&replay, replay_fixture()).unwrap();
        let before = fs::read(&replay).unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_aihack-headless"))
            .args([
                "--seed",
                "42",
                "--turns",
                "1",
                "--policy",
                "replay-file",
                "--replay-in",
                "replays/run.jsonl",
                "--replay-out",
                alias,
            ])
            .current_dir(&root)
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(2),
            "alias={alias} stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(&replay).unwrap(), before, "alias={alias}");
        fs::remove_dir_all(root).unwrap();
    }
}
