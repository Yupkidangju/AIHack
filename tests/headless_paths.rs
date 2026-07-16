use std::{fs, path::Path};

use aihack::core::save::resolve_path_in_root;

#[test]
fn runtime_path_resolver_rejects_absolute_and_parent_traversal() {
    let root = std::env::temp_dir().join(format!("aihack-path-test-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();

    assert!(resolve_path_in_root(&root, Path::new("reports/run.json")).is_ok());
    assert!(resolve_path_in_root(&root, Path::new("../escape.json")).is_err());
    assert!(resolve_path_in_root(&root, Path::new("/tmp/escape.json")).is_err());

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn runtime_path_resolver_rejects_existing_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root =
        std::env::temp_dir().join(format!("aihack-path-symlink-test-{}", std::process::id()));
    let outside = std::env::temp_dir().join(format!("aihack-path-outside-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let escaped = root.join("escaped-report.json");
    let _ = fs::remove_file(&escaped);
    let outside_report = outside.join("report.json");
    fs::write(&outside_report, "outside").unwrap();
    symlink(&outside_report, &escaped).unwrap();

    assert!(resolve_path_in_root(&root, Path::new("escaped-report.json")).is_err());

    fs::remove_file(escaped).unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}
