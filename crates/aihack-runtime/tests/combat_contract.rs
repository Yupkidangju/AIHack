use aihack_core::rng::GameRng;
use aihack_runtime::systems::combat;

#[test]
fn runtime_combat_keeps_seeded_dice_deterministic() {
    let mut first = GameRng::new(42);
    let mut second = GameRng::new(42);

    assert_eq!(
        combat::roll_die(&mut first, 20),
        combat::roll_die(&mut second, 20)
    );
}
