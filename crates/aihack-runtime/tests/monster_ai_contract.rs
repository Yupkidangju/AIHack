use aihack_core::rng::GameRng;
use aihack_runtime::{systems::monster_ai, world::GameWorld};

#[test]
fn runtime_monster_plan_is_seed_deterministic() {
    let world = GameWorld::fixture_phase5();
    let mut first = GameRng::new(42);
    let mut second = GameRng::new(42);

    assert_eq!(
        monster_ai::collect_monster_turn(&world, &mut first),
        monster_ai::collect_monster_turn(&world, &mut second)
    );
}
