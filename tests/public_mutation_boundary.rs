use std::{fs, path::Path};

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn fallible_projectile_and_monster_mutation_is_transaction_managed_only() {
    let runtime_systems = project_file("crates/aihack-runtime/src/systems/mod.rs");
    assert!(runtime_systems.contains("pub(crate) mod monster_ai;"));
    assert!(runtime_systems.contains("pub(crate) mod projectiles;"));
    assert!(!runtime_systems.contains("pub mod monster_ai;"));
    assert!(!runtime_systems.contains("pub mod projectiles;"));

    let facade_systems = project_file("src/systems/mod.rs");
    assert!(!facade_systems.contains("pub mod monster_ai;"));
    assert!(!facade_systems.contains("pub mod projectiles;"));

    let specification = project_file("spec.md");
    assert!(specification
        .contains("외부 consumer의 fallible·atomic mutation 경계는 `GameSession::submit` 하나"));
}
