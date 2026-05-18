use std::{env, fs};

use aihack::{
    core::{save, CommandIntent, Direction, EntityId, GameSession, GameSnapshot, SaveDataV1},
    domain::inventory::InventoryLetter,
};

fn temp_path(name: &str) -> std::path::PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("aihack-{name}-{}.json", std::process::id()));
    path
}

#[test]
fn schema_roundtrip() {
    let session = GameSession::new_for_playing(42);
    let save = session.to_save_data();
    let json = serde_json::to_string(&save).unwrap();
    let decoded: SaveDataV1 = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.schema_version, 1);
    let restored = GameSession::from_save_data(decoded).unwrap();
    assert_eq!(
        session.snapshot().stable_hash(),
        restored.snapshot().stable_hash()
    );
}

#[test]
fn rng_state_restores_continuation() {
    let mut rng = aihack::core::GameRng::new(42);
    let _ = rng.next_u64();
    let _ = rng.next_u64();
    let state = rng.snapshot_state();
    let next_original = rng.next_u64();
    let mut restored = aihack::core::GameRng::from_state(state);
    assert_eq!(next_original, restored.next_u64());
}

#[test]
fn save_load_preserves_snapshot_hash() {
    let path = temp_path("save-hash");
    let mut session = GameSession::new_for_playing(42);
    assert!(session.submit(CommandIntent::Wait).accepted);
    let before = session.snapshot().stable_hash();
    save::save_session_to_path(&session, &path).unwrap();
    let loaded = save::load_session_from_path(&path).unwrap();
    assert_eq!(before, loaded.snapshot().stable_hash());
    let _ = fs::remove_file(path);
}

#[test]
fn loaded_session_matches_direct_continuation() {
    let path = temp_path("save-continue");
    let mut direct = GameSession::new_for_playing(42);
    assert!(direct.submit(CommandIntent::Wait).accepted);
    save::save_session_to_path(&direct, &path).unwrap();
    let mut loaded = save::load_session_from_path(&path).unwrap();

    let commands = [
        CommandIntent::Search,
        CommandIntent::Move(Direction::East),
        CommandIntent::Wait,
    ];
    for command in commands {
        let a = direct.submit(command);
        let b = loaded.submit(command);
        assert_eq!(a.snapshot_hash, b.snapshot_hash);
        assert_eq!(a.next_state, b.next_state);
    }

    assert_eq!(direct.turn, loaded.turn);
    assert_eq!(
        direct.snapshot().stable_hash(),
        loaded.snapshot().stable_hash()
    );
    let _ = fs::remove_file(path);
}

#[test]
fn phase8_state_roundtrip_is_complete() {
    let path = temp_path("save-state");
    let mut session = GameSession::new_for_playing(42);
    session.world.nutrition = 777;
    session.world.luck = 2;
    session.world.prayer_cooldown = 5;
    session.world.paralysis_turns = 1;
    session.world.hallucinating = true;
    session.world.gold = 123;
    session.world.kill_count = 4;
    session
        .world
        .identify_item_kind(aihack::domain::item::ItemKind::Dagger);
    session
        .world
        .entities
        .set_item_charges(EntityId(7), Some(2));
    session.world.inventory.equipped_body = Some(EntityId(10));
    session.world.inventory.entries[0].letter = InventoryLetter('z');
    save::save_session_to_path(&session, &path).unwrap();
    let loaded = save::load_session_from_path(&path).unwrap();
    let loaded_snapshot: GameSnapshot = loaded.snapshot();
    assert_eq!(loaded_snapshot.nutrition, 777);
    assert_eq!(loaded_snapshot.luck, 2);
    assert_eq!(loaded_snapshot.prayer_cooldown, 5);
    assert_eq!(loaded_snapshot.paralysis_turns, 1);
    assert!(loaded_snapshot.hallucinating);
    assert_eq!(loaded_snapshot.gold, 123);
    assert_eq!(loaded_snapshot.kill_count, 4);
    assert!(loaded_snapshot
        .identified_items
        .contains(&aihack::domain::item::ItemKind::Dagger));
    assert_eq!(loaded.world.entities.item_charges(EntityId(7)), Some(2));
    assert_eq!(loaded.world.inventory.equipped_body, Some(EntityId(10)));
    assert_eq!(
        loaded.world.inventory.entries[0].letter,
        InventoryLetter('z')
    );
    let _ = fs::remove_file(path);
}

#[test]
fn invalid_save_schema_is_rejected() {
    let path = temp_path("save-invalid");
    fs::write(
        &path,
        r#"{\"schema_version\":999,\"seed\":42,\"turn\":0,\"run_state\":\"Playing\",\"rng_state\":{\"seed\":42,\"draws\":0},\"world\":{},\"event_log\":[]}"#,
    )
    .unwrap();
    assert!(save::load_session_from_path(&path).is_err());
    let _ = fs::remove_file(path);
}
