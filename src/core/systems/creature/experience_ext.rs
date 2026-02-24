// ============================================================================
// [v2.32.0 R20-4] 경험치/레벨 (experience_ext.rs)
// 원본: NetHack 3.6.7 exper.c
// XP 계산, 레벨업/다운, 최대 레벨
// ============================================================================

/// [v2.32.0 R20-4] 레벨 업 임계값 (원본: newuexp)
pub fn xp_threshold(level: i32) -> i64 {
    match level {
        1 => 0,
        2 => 20,
        3 => 40,
        4 => 80,
        5 => 160,
        6 => 320,
        7 => 640,
        8 => 1280,
        9 => 2560,
        10 => 5120,
        11 => 10000,
        12 => 20000,
        13 => 40000,
        14 => 80000,
        15 => 160000,
        16 => 320000,
        17 => 640000,
        18 => 1280000,
        19 => 2560000,
        20 => 5120000,
        21 => 10000000,
        22 => 20000000,
        23 => 40000000,
        24 => 80000000,
        25 => 160000000,
        26 => 320000000,
        27 => 640000000,
        28 => 1280000000,
        29 => 2560000000,
        30 => 5120000000,
        _ => i64::MAX,
    }
}

/// [v2.32.0 R20-4] 킬 XP (원본: experience)
pub fn kill_xp(monster_level: i32, monster_difficulty: i32) -> i64 {
    let base = (monster_level as i64 + 1) * (monster_difficulty as i64 + 1);
    base.max(1)
}

/// [v2.32.0 R20-4] 레벨 업 체크
pub fn check_level_up(current_level: i32, total_xp: i64) -> Option<i32> {
    let max_level = 30;
    let mut level = current_level;
    while level < max_level && total_xp >= xp_threshold(level + 1) {
        level += 1;
    }
    if level > current_level {
        Some(level)
    } else {
        None
    }
}

/// [v2.32.0 R20-4] 레벨 다운 체크 (드레인)
pub fn check_level_down(current_level: i32, total_xp: i64) -> Option<i32> {
    if current_level <= 1 {
        return None;
    }
    if total_xp < xp_threshold(current_level) {
        Some(current_level - 1)
    } else {
        None
    }
}

/// [v2.32.0 R20-4] 레벨 업 HP 보너스
pub fn level_up_hp_bonus(con: i32) -> i32 {
    let con_bonus = (con - 10) / 2;
    (1 + con_bonus).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds() {
        assert_eq!(xp_threshold(1), 0);
        assert_eq!(xp_threshold(10), 5120);
        assert_eq!(xp_threshold(30), 5120000000);
    }

    #[test]
    fn test_kill_xp() {
        assert_eq!(kill_xp(5, 3), 24);
    }

    #[test]
    fn test_level_up() {
        assert_eq!(check_level_up(1, 25), Some(2));
        assert_eq!(check_level_up(1, 100), Some(4));
    }

    #[test]
    fn test_no_level_up() {
        assert_eq!(check_level_up(5, 100), None);
    }

    #[test]
    fn test_level_down() {
        assert_eq!(check_level_down(5, 100), Some(4));
        assert_eq!(check_level_down(1, 0), None); // 최소 레벨
    }

    #[test]
    fn test_hp_bonus() {
        assert_eq!(level_up_hp_bonus(18), 5);
        assert_eq!(level_up_hp_bonus(8), 1); // 최소 1
    }
}
