use std::{fs, path::Path};

use aihack_runtime::save::resolve_path_in_root;

#[test]
fn runtime_rejects_parent_escape_from_artifact_root() {
    let root = std::env::temp_dir().join(format!("aihack-runtime-save-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();

    assert!(resolve_path_in_root(&root, Path::new("reports/run.json")).is_ok());
    assert!(resolve_path_in_root(&root, Path::new("../escape.json")).is_err());

    fs::remove_dir_all(root).unwrap();
}
