use aihack_content::{registry, CONTENT_SCHEMA_VERSION};

#[test]
fn content_crate_owns_the_embedded_registry_contract() {
    let registry = registry().expect("embedded content must validate");

    assert_eq!(registry.schema_version(), CONTENT_SCHEMA_VERSION);
    assert!(registry.item("item.weapon.dagger").is_some());
    assert!(registry.monster("monster.jackal").is_some());
    assert!(registry.level("main:1").is_some());
    assert_eq!(registry.content_hash(), "f106d044fee3e340");
}
