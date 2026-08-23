use aihack::{
    core::{CommandIntent, GameSession, Pos, RunState},
    testing::SessionBuilder,
};
use std::{fs, process::Command};

#[test]
fn session_exposes_read_only_runtime_state_through_accessors() {
    let mut session = GameSession::new_for_playing(42);

    assert_eq!(session.seed(), 42);
    assert_eq!(session.turn(), 0);
    assert_eq!(session.run_state(), RunState::Playing);
    assert!(session.event_log().is_empty());

    let outcome = session.submit(CommandIntent::Wait);

    assert!(outcome.accepted);
    assert_eq!(session.turn(), 1);
    assert_eq!(session.snapshot().turn, session.turn());
    assert_eq!(session.observation().turn, session.turn());
    assert_eq!(session.event_log().len(), outcome.events.len());
}

#[test]
fn fixture_builder_rebuilds_a_session_from_saved_world_configuration() {
    let session = SessionBuilder::playing(42)
        .configure_saved_world(|world| {
            world.entities.clear_monsters();
            assert!(world.entities.set_actor_location(
                world.player_id,
                world.current_level,
                Pos { x: 8, y: 5 },
            ));
        })
        .build();

    assert_eq!(session.world().player_pos(), Pos { x: 8, y: 5 });
    assert!(session.world().current_level_hostile_monsters().is_empty());
}

#[test]
fn external_consumer_cannot_borrow_session_world_or_entity_store_mutably() {
    struct TempProject(std::path::PathBuf);
    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = TempProject(
        std::env::temp_dir().join(format!("aihack-api-compile-fail-{}", std::process::id())),
    );
    let fixture = fixture_root.0.as_path();
    let source_dir = fixture.join("src");
    fs::create_dir_all(&source_dir).unwrap();
    let dependency_path = root.display().to_string().replace('\\', "/");
    fs::write(
        fixture.join("Cargo.toml"),
        format!(
            "[package]\nname = \"aihack-api-compile-fail\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\naihack = {{ path = \"{dependency_path}\" }}\n"
        ),
    )
    .unwrap();
    fs::write(
        source_dir.join("main.rs"),
        r#"use aihack::core::GameSession;
use aihack::domain::entity::EntityStore;

fn main() {
    let mut session = GameSession::new_for_playing(42);
    session.turn = 999;
    session.world.nutrition = 0;

    let mut world = aihack::core::GameWorld::fixture_phase5();
    world.gold = 999;

    let mut entities = EntityStore::new();
    let _ = entities.get_mut(aihack::core::EntityId(1));
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO"))
        .args(["check", "--offline", "--quiet"])
        .current_dir(fixture)
        .env("CARGO_TARGET_DIR", fixture.join("target"))
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "external mutable consumer unexpectedly compiled"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot assign") || stderr.contains("DerefMut"),
        "unexpected compiler diagnostic: {stderr}"
    );
}
