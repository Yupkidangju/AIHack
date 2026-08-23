use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use aihack::core::{
    error::GameError,
    event::MessagePriority,
    save::{
        ArtifactStore, MAX_PERSISTED_TEXT_BYTES, MAX_RNG_DRAWS, MAX_SAVE_BYTES, MAX_SAVE_ENTITIES,
        MAX_SAVE_EVENTS,
    },
    CommandIntent, Direction, EntityId, GameEvent, GameSession,
};
use aihack::domain::{entity::EntityLocation, item::ItemKind};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

fn malformed_save(mutator: impl FnOnce(&mut serde_json::Value)) -> aihack::core::SaveDataV1 {
    let mut value = serde_json::to_value(GameSession::new_for_playing(42).to_save_data()).unwrap();
    mutator(&mut value);
    serde_json::from_value(value).unwrap()
}

fn assert_invalid_save(save: aihack::core::SaveDataV1) {
    assert!(matches!(
        GameSession::from_save_data(save),
        Err(GameError::InvalidSave(_))
    ));
}

fn assert_typed_invalid_without_panic(save: aihack::core::SaveDataV1) {
    let result = std::panic::catch_unwind(|| GameSession::from_save_data(save));
    assert!(result.is_ok(), "malformed save validation must not panic");
    assert!(matches!(result.unwrap(), Err(GameError::InvalidSave(_))));
}

fn fixture_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "aihack-{label}-{}-{}",
        std::process::id(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn semantic_validator_rejects_missing_non_player_duplicate_and_dangling_entities() {
    assert_invalid_save(malformed_save(|value| {
        value["world"]["player_id"] = serde_json::json!(999);
    }));
    assert_invalid_save(malformed_save(|value| {
        value["world"]["player_id"] = serde_json::json!(2);
    }));
    assert_invalid_save(malformed_save(|value| {
        let entities = value["world"]["entities"]["entities"]
            .as_array_mut()
            .unwrap();
        entities.push(entities[0].clone());
    }));
    assert_invalid_save(malformed_save(|value| {
        value["world"]["inventory"]["entries"][0]["item"] = serde_json::json!(999);
    }));
}

#[test]
fn semantic_validator_rejects_invalid_map_stats_equipment_and_rng_seed() {
    assert_invalid_save(malformed_save(|value| {
        value["world"]["levels"]["levels"][0]["map"]["width"] = serde_json::json!(0);
    }));
    assert_invalid_save(malformed_save(|value| {
        value["world"]["entities"]["entities"][0]["payload"]["Actor"]["stats"]["max_hp"] =
            serde_json::json!(0);
    }));
    assert_invalid_save(malformed_save(|value| {
        value["world"]["inventory"]["equipped_body"] = serde_json::json!(5);
    }));
    assert_invalid_save(malformed_save(|value| {
        value["rng_state"]["seed"] = serde_json::json!(7);
    }));
}

#[test]
fn semantic_validator_rejects_inverse_inventory_actor_and_armor_arithmetic_boundaries() {
    let mut orphan = GameSession::new_for_playing(42).to_save_data();
    let on_map_item = orphan
        .world
        .entities
        .entities()
        .iter()
        .find(|entity| {
            matches!(
                entity.item(),
                Some((_, _, EntityLocation::OnMap { .. }, _, _))
            )
        })
        .map(|entity| entity.id)
        .expect("fixture must contain an on-map item");
    assert!(orphan.world.entities.set_item_location(
        on_map_item,
        EntityLocation::Inventory {
            owner: EntityId(999),
        },
    ));
    assert_typed_invalid_without_panic(orphan);

    let mut hp_over_max = GameSession::new_for_playing(42).to_save_data();
    let player_id = hp_over_max.world.player_id;
    let stats = hp_over_max
        .world
        .entities
        .actor_stats_mut(player_id)
        .expect("player stats must exist");
    stats.hp = stats.max_hp + 1;
    assert_typed_invalid_without_panic(hp_over_max);

    let mut dead_with_positive_hp = GameSession::new_for_playing(42).to_save_data();
    let player_id = dead_with_positive_hp.world.player_id;
    assert!(dead_with_positive_hp
        .world
        .entities
        .set_alive(player_id, false));
    dead_with_positive_hp
        .world
        .entities
        .actor_stats_mut(player_id)
        .unwrap()
        .hp = 1;
    assert_typed_invalid_without_panic(dead_with_positive_hp);

    let mut worn = GameSession::new_for_playing(7);
    aihack::testing::SessionBuilder::mutate(&mut worn, |world| {
        world.saved().entities.clear_monsters();
    });
    assert!(worn.submit(CommandIntent::Move(Direction::East)).accepted);
    assert!(worn.submit(CommandIntent::Move(Direction::East)).accepted);
    assert!(worn.submit(CommandIntent::Pickup).accepted);
    let armor = worn
        .observation()
        .inventory
        .iter()
        .find(|item| item.kind == ItemKind::ArmorLeather)
        .map(|item| item.item)
        .expect("fixture must contain leather armor");
    assert!(worn.submit(CommandIntent::Wear { item: armor }).accepted);

    for bonus in [i16::MIN, i16::MAX] {
        let mut value = serde_json::to_value(worn.to_save_data()).unwrap();
        let entity = value["world"]["entities"]["entities"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entity| entity["id"] == armor.0)
            .expect("armor entity must exist");
        entity["payload"]["Item"]["data"]["ac_bonus"] = serde_json::json!(bonus);
        let save = serde_json::from_value(value).unwrap();
        assert_typed_invalid_without_panic(save);
    }
}

#[test]
fn semantic_validator_rejects_consumer_unsafe_scalars_unequipped_ac_and_forged_item_data() {
    let mut cases = Vec::new();

    cases.push((
        "unequipped player AC",
        malformed_save(|value| {
            value["world"]["entities"]["entities"][0]["payload"]["Actor"]["stats"]["ac"] =
                serde_json::json!(-1);
        }),
    ));
    cases.push((
        "turn increment overflow",
        malformed_save(|value| value["turn"] = serde_json::json!(u64::MAX)),
    ));
    cases.push((
        "kill-count score overflow",
        malformed_save(|value| {
            value["world"]["kill_count"] = serde_json::json!(u32::MAX);
        }),
    ));
    cases.push((
        "forged item base price",
        malformed_save(|value| {
            value["world"]["entities"]["entities"][4]["payload"]["Item"]["data"]["base_price"] =
                serde_json::json!(u32::MAX);
        }),
    ));

    let accepted = cases
        .into_iter()
        .filter_map(|(name, save)| {
            (!matches!(
                GameSession::from_save_data(save),
                Err(GameError::InvalidSave(_))
            ))
            .then_some(name)
        })
        .collect::<Vec<_>>();
    assert!(
        accepted.is_empty(),
        "consumer-unsafe malformed save가 수용됐습니다: {accepted:?}"
    );
}

#[test]
fn persisted_text_accepts_the_byte_limit_and_rejects_control_or_limit_plus_one() {
    let mut exact = GameSession::new_for_playing(42).to_save_data();
    exact.event_log = vec![GameEvent::Message {
        priority: MessagePriority::Info,
        text: "a".repeat(MAX_PERSISTED_TEXT_BYTES),
    }];
    assert!(GameSession::from_save_data(exact).is_ok());

    let mut too_long = GameSession::new_for_playing(42).to_save_data();
    too_long.event_log = vec![GameEvent::CommandRejected {
        reason: "a".repeat(MAX_PERSISTED_TEXT_BYTES + 1),
    }];
    assert_invalid_save(too_long);

    let mut control = GameSession::new_for_playing(42).to_save_data();
    control.event_log = vec![GameEvent::Message {
        priority: MessagePriority::Warning,
        text: "unsafe\u{1b}[31m".to_string(),
    }];
    assert_invalid_save(control);
}

#[test]
fn cardinality_and_rng_budgets_reject_limit_plus_one() {
    let mut events = GameSession::new_for_playing(42).to_save_data();
    events.event_log = vec![GameEvent::Waited { turn: 0 }; MAX_SAVE_EVENTS + 1];
    assert_invalid_save(events);

    let entities = malformed_save(|value| {
        let array = value["world"]["entities"]["entities"]
            .as_array_mut()
            .unwrap();
        let template = array.last().unwrap().clone();
        while array.len() <= MAX_SAVE_ENTITIES {
            let mut entity = template.clone();
            entity["id"] = serde_json::json!(array.len() as u32 + 1_000);
            array.push(entity);
        }
    });
    assert_invalid_save(entities);

    let mut exact_rng = GameSession::new_for_playing(42).to_save_data();
    exact_rng.rng_state.draws = MAX_RNG_DRAWS;
    assert!(GameSession::from_save_data(exact_rng).is_ok());

    let mut rng = GameSession::new_for_playing(42).to_save_data();
    rng.rng_state.draws = MAX_RNG_DRAWS + 1;
    assert_invalid_save(rng);
}

#[test]
fn artifact_store_rejects_save_bytes_limit_plus_one_before_json_decode() {
    let root = std::env::temp_dir().join(format!("aihack-save-budget-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    store
        .write_atomic(Path::new("oversize.json"), &vec![b' '; MAX_SAVE_BYTES + 1])
        .unwrap();

    assert!(matches!(
        store.load_session(Path::new("oversize.json")),
        Err(GameError::InvalidSave(_))
    ));

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn artifact_store_accepts_the_exact_save_byte_limit() {
    let root = std::env::temp_dir().join(format!("aihack-save-exact-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    let mut payload = serde_json::to_vec(&GameSession::new_for_playing(42).to_save_data()).unwrap();
    payload.resize(MAX_SAVE_BYTES, b' ');
    store
        .write_atomic(Path::new("exact.json"), &payload)
        .unwrap();

    let loaded = store.load_session(Path::new("exact.json"));
    assert!(loaded.is_ok(), "exact-byte save failed: {loaded:?}");

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_writer_rejects_a_self_unloadable_payload_without_clobbering_destination() {
    let root = fixture_root("save-writer-budget");
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    let path = Path::new("current.json");
    let original = b"preserve-existing-save";
    store.write_atomic(path, original).unwrap();

    let mut save = GameSession::new_for_playing(42).to_save_data();
    save.event_log = vec![
        GameEvent::Message {
            priority: MessagePriority::Info,
            text: "x".repeat(MAX_PERSISTED_TEXT_BYTES),
        };
        40_000
    ];
    let session = GameSession::from_save_data(save).unwrap();
    let result = store.save_session(&session, path);

    assert!(matches!(result, Err(GameError::InvalidSave(_))));
    assert_eq!(fs::read(root.join(path)).unwrap(), original);

    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn valid_schema_mismatch_reaches_the_typed_version_error() {
    let mut save = GameSession::new_for_playing(42).to_save_data();
    save.schema_version = 999;

    assert!(matches!(
        GameSession::from_save_data(save),
        Err(GameError::SaveSchemaVersionMismatch {
            expected: 1,
            actual: 999,
        })
    ));
}

#[test]
fn entity_id_zero_is_never_accepted_as_a_persisted_reference() {
    assert_invalid_save(malformed_save(|value| {
        value["world"]["inventory"]["owner"] = serde_json::json!(EntityId(0).0);
    }));
}
