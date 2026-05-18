use serde_json::Value;

use aihack::core::{save, ActionIntent, ActionSpace, CommandIntent, GameSession, Observation};

#[test]
fn observation_fixture_roundtrip() {
    let session = GameSession::new_for_playing(42);
    let observation = session.observation();
    let fixture = include_str!("fixtures/observation_v1.json");
    let fixture_value: Value = serde_json::from_str(fixture).unwrap();
    let fixture_object = fixture_value.as_object().unwrap();
    let decoded: Observation =
        serde_json::from_str(&serde_json::to_string(&fixture_value).unwrap()).unwrap();
    assert_eq!(decoded.schema_version, observation.schema_version);

    let value: Value = serde_json::to_value(&observation).unwrap();
    let object = value.as_object().unwrap();
    for key in [
        "schema_version",
        "seed",
        "turn",
        "current_level",
        "run_state",
        "player",
        "visible_tiles",
        "visible_entities",
        "inventory",
        "last_events",
        "action_space",
        "legal_actions",
    ] {
        assert!(object.contains_key(key), "missing observation key {key}");
        assert!(
            fixture_object.contains_key(key),
            "missing fixture key {key}"
        );
    }
    assert_eq!(
        fixture_object.get("schema_version"),
        object.get("schema_version")
    );
    assert_eq!(fixture_object.get("seed"), object.get("seed"));
    assert_eq!(fixture_object.get("turn"), object.get("turn"));
}

#[test]
fn action_space_fixture_roundtrip() {
    let fixture = include_str!("fixtures/action_space_v1.json");
    let decoded: ActionSpace = serde_json::from_str(fixture).unwrap();
    let expected = ActionSpace {
        commands: vec![
            ActionIntent::Command(CommandIntent::Wait),
            ActionIntent::Command(CommandIntent::Search),
            ActionIntent::Noop,
        ],
    };
    assert_eq!(decoded, expected);
    let encoded = serde_json::to_string(&expected).unwrap();
    let fixture_value: Value = serde_json::from_str(fixture).unwrap();
    let encoded_value: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(fixture_value, encoded_value);
}

#[test]
fn save_load_preserves_ai_api_shape() {
    let path = std::env::temp_dir().join(format!("aihack-ai-api-{}.json", std::process::id()));
    let mut session = GameSession::new_for_playing(42);
    assert!(session.submit(CommandIntent::Wait).accepted);
    let before = session.observation();
    save::save_session_to_path(&session, &path).unwrap();
    let loaded = save::load_session_from_path(&path).unwrap();
    let after = loaded.observation();
    assert_eq!(before.schema_version, after.schema_version);
    assert_eq!(before.action_space, after.action_space);
    assert_eq!(before.player, after.player);
    assert_eq!(before.current_level, after.current_level);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn observation_does_not_expose_runtime_containers() {
    let observation = GameSession::new_for_playing(42).observation();
    let encoded = serde_json::to_string(&observation).unwrap();
    assert!(!encoded.contains("EntityStore"));
    assert!(!encoded.contains("GameWorld"));
    assert!(!encoded.contains("InventoryEntry"));
}
