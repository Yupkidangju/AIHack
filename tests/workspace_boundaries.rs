use std::{fs, path::Path};

#[test]
fn core_and_content_workspace_boundaries_remain_one_way() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_manifest = fs::read_to_string(root.join("crates/aihack-core/Cargo.toml")).unwrap();
    let content_manifest =
        fs::read_to_string(root.join("crates/aihack-content/Cargo.toml")).unwrap();
    let tui_manifest = fs::read_to_string(root.join("apps/aihack-tui/Cargo.toml")).unwrap();
    let headless_manifest =
        fs::read_to_string(root.join("apps/aihack-headless/Cargo.toml")).unwrap();
    let root_data = fs::read_to_string(root.join("src/data/mod.rs")).unwrap();
    let core_entity = fs::read_to_string(root.join("crates/aihack-core/src/domain/entity.rs"))
        .expect("core must own the entity storage implementation");

    for forbidden in ["ratatui", "crossterm", "reqwest", "ureq", "hyper"] {
        assert!(
            !core_manifest.contains(forbidden),
            "core must not depend on {forbidden}"
        );
    }
    assert!(content_manifest.contains("aihack-core"));
    assert!(!tui_manifest.contains("aihack-core"));
    assert!(!headless_manifest.contains("aihack-core"));
    assert!(tui_manifest.contains("aihack-runtime"));
    assert!(tui_manifest.contains("aihack-ai-contract"));
    assert!(headless_manifest.contains("aihack-runtime"));
    assert!(headless_manifest.contains("aihack-ai-contract"));
    assert!(!core_entity.contains("aihack_content"));
    assert!(!core_entity.contains("item::{item_data"));
    assert!(!core_entity.contains("monster::{monster_template"));
    assert!(root_data.contains("aihack_content"));
    assert!(root.join("crates/aihack-content/src/schema.rs").is_file());
    assert!(root
        .join("crates/aihack-content/src/data/items.toml")
        .is_file());
    assert!(root
        .join("crates/aihack-content/src/data/monsters.toml")
        .is_file());
    assert!(!root.join("src/data/schema.rs").exists());
}
