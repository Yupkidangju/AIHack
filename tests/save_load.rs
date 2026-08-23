use std::{env, fs};

use aihack::{
    core::{
        save::ArtifactStore, CommandIntent, Direction, EntityId, GameSession, GameSnapshot,
        SaveDataV1,
    },
    domain::inventory::InventoryLetter,
};

fn temp_store(name: &str) -> (std::path::PathBuf, ArtifactStore) {
    let root = env::temp_dir().join(format!("aihack-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    (root, store)
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
    let (root, store) = temp_store("save-hash");
    let path = std::path::Path::new("save.json");
    let mut session = GameSession::new_for_playing(42);
    assert!(session.submit(CommandIntent::Wait).accepted);
    let before = session.snapshot().stable_hash();
    store.save_session(&session, path).unwrap();
    let loaded = store.load_session(path).unwrap();
    assert_eq!(before, loaded.snapshot().stable_hash());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn loaded_session_matches_direct_continuation() {
    let (root, store) = temp_store("save-continue");
    let path = std::path::Path::new("save.json");
    let mut direct = GameSession::new_for_playing(42);
    assert!(direct.submit(CommandIntent::Wait).accepted);
    store.save_session(&direct, path).unwrap();
    let mut loaded = store.load_session(path).unwrap();

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

    assert_eq!(direct.turn(), loaded.turn());
    assert_eq!(
        direct.snapshot().stable_hash(),
        loaded.snapshot().stable_hash()
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn phase8_state_roundtrip_is_complete() {
    let (root, store) = temp_store("save-state");
    let path = std::path::Path::new("save.json");
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_status(aihack::domain::status::Status {
            nutrition: 777,
            luck: 2,
            prayer_cooldown: 5,
            paralysis_turns: 1,
            hallucinating: true,
        });
        world.set_gold(123);
        world.set_kill_count(4);
        world.identify_item_kind(aihack::domain::item::ItemKind::Dagger);
        assert!(world
            .saved()
            .entities
            .set_item_charges(EntityId(7), Some(2)));
        let saved = world.saved();
        let player = saved.player_id;
        let armor = EntityId(10);
        let armor_letter = saved
            .inventory
            .add_existing_with_next_letter(armor)
            .expect("fixture armor must receive an inventory letter");
        assert!(saved.entities.set_item_location(
            armor,
            aihack::domain::entity::EntityLocation::Inventory { owner: player }
        ));
        assert!(saved.entities.set_item_letter(armor, armor_letter));
        saved.inventory.equipped_body = Some(armor);
        let armor_bonus = saved.entities.item_data(armor).unwrap().ac_bonus;
        saved.entities.actor_stats_mut(player).unwrap().ac -= armor_bonus;
        saved.inventory.entries[0].letter = InventoryLetter('z');
        assert!(saved
            .entities
            .set_item_letter(EntityId(5), InventoryLetter('z')));
    });
    store.save_session(&session, path).unwrap();
    let loaded = store.load_session(path).unwrap();
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
    assert_eq!(loaded.world().entities().item_charges(EntityId(7)), Some(2));
    assert_eq!(loaded.world().inventory().equipped_body, Some(EntityId(10)));
    assert_eq!(
        loaded.world().inventory().entries[0].letter,
        InventoryLetter('z')
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_save_schema_is_rejected() {
    let (root, store) = temp_store("save-invalid");
    let path = std::path::Path::new("save.json");
    store
        .write_atomic(
            path,
            r#"{\"schema_version\":999,\"seed\":42,\"turn\":0,\"run_state\":\"Playing\",\"rng_state\":{\"seed\":42,\"draws\":0},\"world\":{},\"event_log\":[]}"#
                .as_bytes(),
        )
        .unwrap();
    assert!(store.load_session(path).is_err());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
