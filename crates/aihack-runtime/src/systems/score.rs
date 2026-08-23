use crate::world::GameWorld;

pub use aihack_core::score::{apply_luck, hallucination_message};

pub fn death_score(world: &GameWorld, turn: u64) -> i32 {
    aihack_core::score::death_score(world, turn)
}

/// 같은 world/turn에서 gold만 제거한 control도 production 점수 함수를 통과시킨다.
pub fn paired_gold_scores(world: &GameWorld, turn: u64) -> (i32, i32) {
    let with_gold = death_score(world, turn);
    let mut without_gold_world = world.clone();
    without_gold_world.state_mut().gold = 0;
    let without_gold = death_score(&without_gold_world, turn);
    (with_gold, without_gold)
}
