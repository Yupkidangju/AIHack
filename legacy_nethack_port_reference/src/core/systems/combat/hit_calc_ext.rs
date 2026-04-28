// ============================================================================
// [v2.42.0 R30-1] 히트/미스 판정 (hit_calc_ext.rs)
// 원본: NetHack 3.6.7 uhitm.c/mhitm.c 명중 확장
// 명중 판정, AC 계산, 크리티컬, 회피
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.42.0 R30-1] 명중 판정
pub fn roll_to_hit(attack_bonus: i32, target_ac: i32, luck: i32, rng: &mut NetHackRng) -> bool {
    let roll = rng.rn2(20) + 1; // d20
    let needed = target_ac.max(0) + 1 - attack_bonus - luck;
    roll >= needed
}

/// [v2.42.0 R30-1] 크리티컬 히트
pub fn is_critical(roll: i32) -> bool {
    roll == 20
}

/// [v2.42.0 R30-1] 데미지 롤
pub fn roll_damage(dice: i32, sides: i32, bonus: i32, rng: &mut NetHackRng) -> i32 {
    let mut total = 0;
    for _ in 0..dice {
        total += rng.rn2(sides) + 1;
    }
    (total + bonus).max(1)
}

/// [v2.42.0 R30-1] 사거리 명중 감쇠
pub fn range_penalty(distance: i32) -> i32 {
    match distance {
        0..=3 => 0,
        4..=6 => -2,
        7..=10 => -4,
        _ => -7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_hit_low_ac() {
        let mut hits = 0;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if roll_to_hit(15, -5, 3, &mut rng) {
                hits += 1;
            }
        }
        assert!(hits > 40); // 거의 항상
    }

    #[test]
    fn test_damage() {
        let mut rng = NetHackRng::new(42);
        let d = roll_damage(2, 6, 3, &mut rng);
        assert!(d >= 5 && d <= 15);
    }

    #[test]
    fn test_critical() {
        assert!(is_critical(20));
        assert!(!is_critical(19));
    }

    #[test]
    fn test_range_penalty() {
        assert_eq!(range_penalty(2), 0);
        assert_eq!(range_penalty(8), -4);
    }
}
