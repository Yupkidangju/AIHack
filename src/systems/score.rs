use crate::core::world::GameWorld;

/// [v0.1.0] Phase 8 최소 death score 계산식이다.
pub fn death_score(world: &GameWorld, turn: u64) -> i32 {
    world.gold as i32 + world.kill_count as i32 * 10 + world.current_level().depth as i32 * 100
        - (turn / 10) as i32
}

/// [v0.1.0] Phase 8 luck 보정 helper다.
pub fn apply_luck(base: i16, luck: i16) -> i16 {
    base + luck
}

/// [v0.1.0] hallucination은 메시지만 바꾸고 core state는 바꾸지 않는다.
pub fn hallucination_message(base: &str, hallucinating: bool) -> String {
    if hallucinating {
        format!("환각: {base} 가 무지개처럼 보인다")
    } else {
        base.to_string()
    }
}
