use std::{fs, path::Path};

use aihack_runtime::save::ArtifactStore;

#[test]
fn runtime_rejects_parent_escape_from_artifact_root() {
    let root = std::env::temp_dir().join(format!("aihack-runtime-save-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();

    let store = ArtifactStore::open(&root).unwrap();
    assert_eq!(
        store
            .validate_path(Path::new("./reports/run.json"))
            .unwrap(),
        Path::new("reports/run.json")
    );
    assert!(store.validate_path(Path::new("../escape.json")).is_err());

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_artifact_api_does_not_expose_the_ambient_path_resolver() {
    let source = include_str!("../src/save.rs");
    assert!(!source.contains("pub fn resolve_path_in_root"));
    assert!(source.contains("pub struct ArtifactStore"));
}
