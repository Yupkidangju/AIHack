use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn headless_cli_supports_13_roles_race_alignment_and_rejects_invalid_build() {
    let bin = env!("CARGO_BIN_EXE_aihack-headless");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock monotonically after epoch")
        .as_nanos();
    let work_dir =
        std::env::temp_dir().join(format!("aihack-p10-cli-{}-{unique}", std::process::id()));
    fs::create_dir_all(&work_dir).expect("create temp test workdir");

    let ok = Command::new(bin)
        .current_dir(&work_dir)
        .args([
            "--role",
            "valkyrie",
            "--race",
            "dwarf",
            "--alignment",
            "neutral",
            "--turns",
            "3",
        ])
        .output()
        .expect("execute aihack-headless");
    assert!(
        ok.status.success(),
        "valid custom build must succeed: {}",
        String::from_utf8_lossy(&ok.stderr)
    );

    let bad = Command::new(bin)
        .current_dir(&work_dir)
        .args([
            "--role",
            "samurai",
            "--race",
            "dwarf",
            "--alignment",
            "lawful",
            "--turns",
            "3",
        ])
        .output()
        .expect("execute aihack-headless");
    assert!(
        !bad.status.success(),
        "invalid role/race/alignment build must exit non-zero"
    );

    let _ = fs::remove_dir_all(&work_dir);
}
