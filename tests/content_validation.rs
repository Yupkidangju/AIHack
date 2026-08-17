use aihack::{
    core::{ContentError, GameSession},
    data::{ContentRegistry, CONTENT_SCHEMA_VERSION},
};

const ITEMS: &str = r#"
[[item]]
id = "item.weapon.dagger"
kind = "weapon"
glyph = ")"
weight = 10
damage = "1d4"
"#;
const MONSTERS: &str = r#"
[[monster]]
id = "monster.jackal"
glyph = "d"
hp = 4
ac = 0
hit_bonus = 0
damage = "1d2"
ai = "wander"
speed = 12
difficulty = 1
"#;
const LEVEL_1: &str = r#"
level_id = "main:1"
branch = "Main"
depth = 1
width = 10
height = 10
player_start = [1, 1]
stairs_down = [8, 8]
[[monster]]
id = "monster.jackal"
pos = [2, 2]
"#;
const LEVEL_2: &str = r#"
level_id = "main:2"
branch = "Main"
depth = 2
width = 10
height = 10
player_start = [1, 1]
stairs_up = [1, 1]
"#;
const LEVEL_1_WITHOUT_CONTENT: &str = r#"
level_id = "main:1"
branch = "Main"
depth = 1
width = 10
height = 10
player_start = [1, 1]
stairs_down = [8, 8]
"#;
const EMPTY_MONSTERS: &str = "monster = []";

fn registry(
    items: &str,
    monsters: &str,
    levels: &[(&str, &str)],
) -> Result<ContentRegistry, ContentError> {
    ContentRegistry::from_toml_sources(CONTENT_SCHEMA_VERSION, items, monsters, levels)
}

#[test]
fn embedded_registry_has_schema_v1_and_stable_hash() {
    let hashes = (0..3)
        .map(|_| {
            ContentRegistry::from_embedded()
                .unwrap()
                .content_hash()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert!(hashes.iter().all(|hash| hash.len() == 16
        && hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())));
    assert!(hashes.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(
        ContentRegistry::from_embedded().unwrap().schema_version(),
        1
    );
}

#[test]
fn duplicate_id_is_a_typed_error() {
    let duplicate = format!("{ITEMS}\n{ITEMS}");
    assert!(matches!(
        registry(&duplicate, MONSTERS, &[("one", LEVEL_1), ("two", LEVEL_2)]),
        Err(ContentError::DuplicateId { .. })
    ));
}

#[test]
fn unknown_reference_is_a_typed_error() {
    let invalid_level = LEVEL_1.replace("monster.jackal", "monster.missing");
    assert!(matches!(
        registry(
            ITEMS,
            MONSTERS,
            &[("one", &invalid_level), ("two", LEVEL_2)]
        ),
        Err(ContentError::UnknownReference { .. })
    ));
}

#[test]
fn invalid_dice_and_coordinate_are_typed_errors_without_panicking() {
    let invalid_monsters = MONSTERS.replace("1d2", "2d0");
    assert!(matches!(
        registry(
            ITEMS,
            &invalid_monsters,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::InvalidDice { .. })
    ));
    let invalid_level = LEVEL_1.replace("[8, 8]", "[10, 8]");
    assert!(matches!(
        registry(
            ITEMS,
            MONSTERS,
            &[("one", &invalid_level), ("two", LEVEL_2)]
        ),
        Err(ContentError::InvalidCoordinate { .. })
    ));
}

#[test]
fn causal_numeric_content_rejects_invalid_ranges() {
    let negative_price = format!("{ITEMS}\nbase_price = -1");
    assert!(matches!(
        registry(
            &negative_price,
            MONSTERS,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::Parse { .. })
    ));
    let excessive_speed = MONSTERS.replace("speed = 12", "speed = 13");
    assert!(matches!(
        registry(
            ITEMS,
            &excessive_speed,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::Parse { .. })
    ));
    let zero_difficulty = MONSTERS.replace("difficulty = 1", "difficulty = 0");
    assert!(matches!(
        registry(
            ITEMS,
            &zero_difficulty,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::Parse { .. })
    ));
}

#[test]
fn unsupported_schema_and_unpaired_stairs_are_typed_errors() {
    assert!(matches!(
        ContentRegistry::from_toml_sources(
            2,
            ITEMS,
            MONSTERS,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::Parse { .. })
    ));
    assert!(matches!(
        registry(ITEMS, MONSTERS, &[("one", LEVEL_1)]),
        Err(ContentError::MissingStairsPair { .. })
    ));
}

#[test]
fn session_bootstrap_returns_content_error_when_main_level_is_missing() {
    let registry = registry(ITEMS, MONSTERS, &[]).unwrap();

    let result = GameSession::try_new_for_playing_with_registry(42, &registry);

    assert!(matches!(
        result,
        Err(ContentError::UnknownReference { owner, target })
            if owner == "world bootstrap" && target == "main:1"
    ));
}

#[test]
fn session_bootstrap_uses_the_injected_registry_for_starting_items() {
    let registry = registry(
        ITEMS,
        EMPTY_MONSTERS,
        &[("one", LEVEL_1_WITHOUT_CONTENT), ("two", LEVEL_2)],
    )
    .unwrap();

    let result = GameSession::try_new_for_playing_with_registry(42, &registry);

    assert!(matches!(
        result,
        Err(ContentError::UnknownReference { owner, target })
            if owner == "item factory" && target == "item.food.ration"
    ));
}
