/// 점수 계산에 필요한 world 읽기 전용 상태다.
pub trait DeathScoreView {
    fn gold_amount(&self) -> u32;
    fn kill_count(&self) -> u32;
    fn current_level_depth(&self) -> i16;
    fn inventory_value(&self) -> u32 {
        0
    }
}

/// 현재 death score 계산식이다.
pub fn death_score(world: &impl DeathScoreView, turn: u64) -> i32 {
    world.gold_amount() as i32
        + world.kill_count() as i32 * 10
        + world.current_level_depth() as i32 * 100
        + world.inventory_value() as i32
        - (turn / 10) as i32
}

pub fn apply_luck(base: i16, luck: i16) -> i16 {
    base + luck
}

/// 환각은 simulation state가 아니라 표시 문자열만 바꾼다.
pub fn hallucination_message(base: &str, hallucinating: bool) -> String {
    if hallucinating {
        format!("Hallucination: {base} shimmers in impossible colors.")
    } else {
        base.to_string()
    }
}
