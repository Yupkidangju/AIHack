use aihack::core::world::GameWorld;

#[test]
fn default_fixture_uses_the_validated_content_bootstrap() {
    let expected = GameWorld::try_fixture_phase5().expect("embedded content must validate");
    let actual = GameWorld::fixture_phase5();

    assert_eq!(actual, expected);
}
