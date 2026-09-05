use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

struct CompileFixture(PathBuf);

impl CompileFixture {
    fn new(name: &str, source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "aihack-r30-public-{name}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        let runtime = project
            .join("crates/aihack-runtime")
            .display()
            .to_string()
            .replace('\\', "/");
        let core = project
            .join("crates/aihack-core")
            .display()
            .to_string()
            .replace('\\', "/");
        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"r30-{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\naihack-runtime = {{ path = \"{runtime}\", default-features = false }}\naihack-core = {{ path = \"{core}\" }}\n"
            ),
        )
        .unwrap();
        fs::write(root.join("src/main.rs"), source).unwrap();
        Self(root)
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet"])
            .current_dir(&self.0)
            .env("CARGO_TARGET_DIR", self.0.join("target"))
            .output()
            .unwrap()
    }
}

impl Drop for CompileFixture {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}

fn project_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn rust_sources_under(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources_under(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

#[test]
fn default_runtime_read_queries_compile_for_an_external_consumer() {
    let fixture = CompileFixture::new(
        "read-pass",
        r#"use aihack_runtime::{systems::{score, vision}, GameSession};

fn main() {
    let session = GameSession::new_for_playing(42);
    let world = session.world();
    let _ = world.player_pos();
    let _ = world.entities().get(world.player_id());
    let _ = score::death_score(world, session.turn());
    let _ = vision::visible_positions(world);
}
"#,
    );

    let output = fixture.check();
    assert!(
        output.status.success(),
        "read-only external consumer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn default_runtime_rejects_external_world_and_system_mutation() {
    for (name, source) in [
        (
            "world-mutation",
            r#"use aihack_runtime::{world::{GameWorld, PHASE3_JACKAL_START}, GameSession};

fn main() {
    let session = GameSession::new_for_playing(42);
    let mut world = GameWorld::from_saved_world(session.to_save_data().world).unwrap();
    world.set_player_pos(PHASE3_JACKAL_START);
}
"#,
        ),
        (
            "system-mutation",
            r#"use aihack_core::position::Direction;
use aihack_runtime::{systems::movement, world::GameWorld, GameSession};

fn main() {
    let session = GameSession::new_for_playing(42);
    let mut world = GameWorld::from_saved_world(session.to_save_data().world).unwrap();
    movement::move_player(&mut world, Direction::East).unwrap();
}
"#,
        ),
        (
            "testing-feature",
            r#"use aihack_runtime::testing;

fn main() {
    let _ = testing::resolve_depleted_death;
}
"#,
        ),
    ] {
        let fixture = CompileFixture::new(name, source);
        let output = fixture.check();
        assert!(
            !output.status.success(),
            "forbidden default public mutation compiled: {name}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("private")
                || stderr.contains("not found")
                || stderr.contains("no method named")
                || stderr.contains("unresolved import")
                || stderr.contains("could not find"),
            "unexpected compiler diagnostic for {name}: {stderr}"
        );
    }

    let systems = project_file("crates/aihack-runtime/src/systems/mod.rs");
    for module in [
        "combat",
        "death",
        "doors",
        "items",
        "monster_ai",
        "movement",
        "projectiles",
        "stairs",
        "traps",
    ] {
        assert!(
            systems.contains(&format!("pub(crate) mod {module};")),
            "mutating module must be crate-private: {module}"
        );
    }
    for module in ["score", "vision"] {
        assert!(
            systems.contains(&format!("pub mod {module};")),
            "read-only module must remain public: {module}"
        );
    }

    let facade_systems = project_file("src/systems/mod.rs");
    assert_eq!(
        facade_systems.lines().collect::<Vec<_>>(),
        ["pub mod score;", "pub mod vision;"]
    );

    for manifest in [
        "apps/aihack-tui/Cargo.toml",
        "apps/aihack-headless/Cargo.toml",
    ] {
        let content = project_file(manifest);
        assert!(!content.contains("features = [\"testing\"]"), "{manifest}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut shipped_sources = Vec::new();
    for app in ["apps/aihack-tui/src", "apps/aihack-headless/src"] {
        rust_sources_under(&root.join(app), &mut shipped_sources);
    }
    for source in shipped_sources {
        let content = fs::read_to_string(&source).unwrap();
        assert!(
            !content.contains("aihack_runtime::testing")
                && !content.contains("aihack_runtime::systems::combat")
                && !content.contains("aihack_runtime::systems::death")
                && !content.contains("aihack_runtime::systems::doors")
                && !content.contains("aihack_runtime::systems::items")
                && !content.contains("aihack_runtime::systems::movement")
                && !content.contains("aihack_runtime::systems::stairs")
                && !content.contains("aihack_runtime::systems::traps"),
            "shipped adapter imports a non-production mutation surface: {}",
            source.display()
        );
    }

    assert!(project_file("Cargo.toml").contains("features = [\"testing\"]"));
    assert!(project_file("crates/aihack-runtime/Cargo.toml").contains("testing = []"));
}
