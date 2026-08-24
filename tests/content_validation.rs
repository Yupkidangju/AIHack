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
slot = "melee"
hit_bonus = 0
damage = "1d4"
"#;
const EMBEDDED_ITEMS: &str = include_str!("../crates/aihack-content/src/data/items.toml");
const EMBEDDED_MONSTERS: &str = include_str!("../crates/aihack-content/src/data/monsters.toml");
const EMBEDDED_LEVEL_1: &str = include_str!("../crates/aihack-content/src/data/levels/main_1.toml");
const EMBEDDED_LEVEL_2: &str = include_str!("../crates/aihack-content/src/data/levels/main_2.toml");
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

    let irreversible_armor = format!(
        "{ITEMS}\n[[item]]\nid = \"item.armor.leather\"\nkind = \"armor\"\nglyph = \"[\"\nweight = 15\nslot = \"body\"\nac_bonus = {}\nbase_price = 8\n",
        i16::MIN
    );
    assert!(matches!(
        registry(
            &irreversible_armor,
            MONSTERS,
            &[("one", LEVEL_1), ("two", LEVEL_2)]
        ),
        Err(ContentError::Parse { .. })
    ));
}

#[test]
fn registry_rejects_non_live_monster_hp_and_forbidden_armor_projectile_fields() {
    for hp in [0, -1] {
        let monsters = MONSTERS.replace("hp = 4", &format!("hp = {hp}"));
        assert!(matches!(
            registry(ITEMS, &monsters, &[("one", LEVEL_1), ("two", LEVEL_2)]),
            Err(ContentError::Parse { .. })
        ));
    }

    for forbidden in ["damage=\"1d4\"", "hit_bonus=1"] {
        let items = EMBEDDED_ITEMS.replacen("ac_bonus=1", &format!("ac_bonus=1\n{forbidden}"), 1);
        assert!(matches!(
            registry(
                &items,
                EMBEDDED_MONSTERS,
                &[
                    ("main_1.toml", EMBEDDED_LEVEL_1),
                    ("main_2.toml", EMBEDDED_LEVEL_2)
                ]
            ),
            Err(ContentError::Parse { .. })
        ));
    }
}

#[test]
fn accepted_custom_registry_bootstrap_wait_and_save_round_trip_remain_valid() {
    let monsters = EMBEDDED_MONSTERS.replacen("hp=4", "hp=5", 1);
    let registry = ContentRegistry::from_toml_sources(
        CONTENT_SCHEMA_VERSION,
        EMBEDDED_ITEMS,
        &monsters,
        &[
            ("main_1.toml", EMBEDDED_LEVEL_1),
            ("main_2.toml", EMBEDDED_LEVEL_2),
        ],
    )
    .unwrap();
    let mut session = GameSession::try_new_for_playing_with_registry(42, &registry).unwrap();
    assert!(GameSession::from_save_data_with_registry(session.to_save_data(), &registry).is_ok());
    assert!(session.submit(aihack::core::CommandIntent::Wait).accepted);
    assert!(GameSession::from_save_data_with_registry(session.to_save_data(), &registry).is_ok());
}

#[test]
fn known_item_id_rejects_a_shape_valid_declared_class_override() {
    let dagger_as_armor = EMBEDDED_ITEMS.replacen(
        "kind=\"weapon\"\nglyph=\")\"\nweight=10\nslot=\"melee\"\nhit_bonus=1\ndamage=\"1d4\"",
        "kind=\"armor\"\nglyph=\"[\"\nweight=10\nslot=\"body\"\nac_bonus=1",
        1,
    );

    assert!(matches!(
        registry(
            &dagger_as_armor,
            EMBEDDED_MONSTERS,
            &[
                ("main_1.toml", EMBEDDED_LEVEL_1),
                ("main_2.toml", EMBEDDED_LEVEL_2)
            ]
        ),
        Err(ContentError::Parse { .. })
    ));
}

#[test]
fn embedded_known_item_ids_keep_the_canonical_declared_kind_table() {
    let registry = ContentRegistry::from_embedded().unwrap();
    for (id, expected_kind) in [
        ("item.weapon.dagger", "weapon"),
        ("item.food.ration", "food"),
        ("item.potion.healing", "potion"),
        ("item.wand.magic_missile", "wand"),
        ("item.scroll.identify", "scroll"),
        ("item.scroll.reveal", "scroll"),
        ("item.scroll.teleport", "scroll"),
        ("item.armor.leather", "armor"),
        ("item.weapon.rock", "weapon"),
        ("item.corpse.jackal", "corpse"),
    ] {
        assert_eq!(registry.item(id).unwrap().kind, expected_kind, "id={id}");
    }
}

#[test]
fn item_glyph_requires_exactly_one_unicode_scalar() {
    for invalid in ["", "AB", "e\u{301}"] {
        let items = EMBEDDED_ITEMS.replacen("glyph=\")\"", &format!("glyph=\"{invalid}\""), 1);
        assert!(
            matches!(
                registry(
                    &items,
                    EMBEDDED_MONSTERS,
                    &[
                        ("main_1.toml", EMBEDDED_LEVEL_1),
                        ("main_2.toml", EMBEDDED_LEVEL_2)
                    ]
                ),
                Err(ContentError::Parse { .. })
            ),
            "invalid glyph accepted: {invalid:?}"
        );
    }

    let unicode = EMBEDDED_ITEMS.replacen("glyph=\")\"", "glyph=\"🗡\"", 1);
    assert!(registry(
        &unicode,
        EMBEDDED_MONSTERS,
        &[
            ("main_1.toml", EMBEDDED_LEVEL_1),
            ("main_2.toml", EMBEDDED_LEVEL_2)
        ]
    )
    .is_ok());
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
