use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use aihack::core::{save::ArtifactStore, CommandIntent, GameSession, ReplayLineV1};

static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

fn temp_test_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "aihack-{name}-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn runtime_path_resolver_rejects_absolute_and_parent_traversal() {
    let root = temp_test_dir("path-test");
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();

    assert!(store.validate_path(Path::new("reports/run.json")).is_ok());
    assert!(store.validate_path(Path::new("../escape.json")).is_err());
    assert!(store.validate_path(Path::new("/tmp/escape.json")).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn save_does_not_touch_preplaced_predictable_temp_hard_link() {
    let root = temp_test_dir("save-temp-hard-link");
    let outside = temp_test_dir("save-temp-hard-link-victim");
    fs::create_dir_all(root.join("saves")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.txt");
    fs::write(&victim, "victim-must-not-change").unwrap();
    fs::hard_link(&victim, root.join("saves/run.tmp")).unwrap();

    let store = ArtifactStore::open(&root).unwrap();
    let session = GameSession::new_for_playing(42);
    store
        .save_session(&session, Path::new("saves/run.json"))
        .unwrap();

    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        "victim-must-not-change"
    );
    assert!(store.load_session(Path::new("saves/run.json")).is_ok());

    drop(store);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn save_rejects_existing_destination_hard_link_without_changing_victim() {
    let root = temp_test_dir("save-destination-hard-link");
    let outside = temp_test_dir("save-destination-hard-link-victim");
    fs::create_dir_all(root.join("saves")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.txt");
    fs::write(&victim, "victim-must-not-change").unwrap();
    fs::hard_link(&victim, root.join("saves/run.json")).unwrap();

    let store = ArtifactStore::open(&root).unwrap();
    let session = GameSession::new_for_playing(42);
    assert!(store
        .save_session(&session, Path::new("saves/run.json"))
        .is_err());
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        "victim-must-not-change"
    );

    drop(store);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn save_atomically_replaces_existing_regular_file() {
    let root = temp_test_dir("save-replace");
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();

    store
        .save_session(
            &GameSession::new_for_playing(42),
            Path::new("saves/run.json"),
        )
        .unwrap();
    store
        .save_session(
            &GameSession::new_for_playing(7),
            Path::new("saves/run.json"),
        )
        .unwrap();

    assert_eq!(
        store
            .load_session(Path::new("saves/run.json"))
            .unwrap()
            .seed(),
        7
    );

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn unix_save_file_uses_mode_0600() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_test_dir("save-unix-mode");
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    store
        .save_session(
            &GameSession::new_for_playing(42),
            Path::new("saves/run.json"),
        )
        .unwrap();

    let mode = fs::metadata(root.join("saves/run.json"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_save_file_is_writable_under_the_parent_acl_boundary() {
    let root = temp_test_dir("save-windows-acl");
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    let path = Path::new("saves/run.json");

    store
        .save_session(&GameSession::new_for_playing(42), path)
        .unwrap();
    assert!(!fs::metadata(root.join(path))
        .unwrap()
        .permissions()
        .readonly());
    store
        .save_session(&GameSession::new_for_playing(7), path)
        .unwrap();
    assert_eq!(store.load_session(path).unwrap().seed(), 7);

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replay_append_rejects_hard_link_without_changing_victim() {
    let root = temp_test_dir("replay-hard-link");
    let outside = temp_test_dir("replay-hard-link-victim");
    fs::create_dir_all(root.join("replays")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let victim = outside.join("victim.txt");
    fs::write(&victim, "victim-must-not-change").unwrap();
    fs::hard_link(&victim, root.join("replays/run.jsonl")).unwrap();

    let mut session = GameSession::new_for_playing(42);
    let turn_before = session.turn();
    let command = CommandIntent::Wait;
    let outcome = session.submit(command);
    let line = ReplayLineV1 {
        turn_before,
        command,
        snapshot_hash_after: outcome.snapshot_hash.clone(),
        outcome,
    };
    let store = ArtifactStore::open(&root).unwrap();

    assert!(store
        .append_replay_line(Path::new("replays/run.jsonl"), &line)
        .is_err());
    assert_eq!(
        fs::read_to_string(&victim).unwrap(),
        "victim-must-not-change"
    );

    drop(store);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
#[test]
fn runtime_path_resolver_rejects_existing_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = temp_test_dir("path-symlink-test");
    let outside = temp_test_dir("path-symlink-outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let escaped = root.join("escaped-report.json");
    let _ = fs::remove_file(&escaped);
    let outside_report = outside.join("report.json");
    fs::write(&outside_report, "outside").unwrap();
    symlink(&outside_report, &escaped).unwrap();

    let store = ArtifactStore::open(&root).unwrap();
    assert!(store
        .validate_path(Path::new("escaped-report.json"))
        .is_err());

    fs::remove_file(escaped).unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}
