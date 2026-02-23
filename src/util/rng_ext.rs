// ============================================================================
// [v2.27.0 R15-4] RNG 확장 (rng_ext.rs)
// 원본: NetHack 3.6.7 rnd.c (260줄) 확장
// 다이스 표현식, 가중 랜덤, 행운 보정 RNG
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 다이스 시스템 (원본: rnd.c d, rn2)
// =============================================================================

/// [v2.27.0 R15-4] 다이스 표현식 (NdS+M)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceExpr {
    /// 다이스 개수
    pub count: i32,
    /// 다이스 면수
    pub sides: i32,
    /// 고정 보정
    pub modifier: i32,
}

impl DiceExpr {
    pub fn new(count: i32, sides: i32, modifier: i32) -> Self {
        Self {
            count: count.max(0),
            sides: sides.max(1),
            modifier,
        }
    }

    /// 최소값
    pub fn min(&self) -> i32 {
        self.count + self.modifier
    }
    /// 최대값
    pub fn max(&self) -> i32 {
        self.count * self.sides + self.modifier
    }
    /// 기대값
    pub fn expected(&self) -> f64 {
        (self.count as f64) * ((self.sides as f64 + 1.0) / 2.0) + self.modifier as f64
    }

    /// 굴리기
    pub fn roll(&self, rng: &mut NetHackRng) -> i32 {
        let mut total = self.modifier;
        for _ in 0..self.count {
            total += rng.rn1(self.sides, 1);
        }
        total
    }
}

/// [v2.27.0 R15-4] 다이스 표현식 파싱 (예: "2d6+3")
pub fn parse_dice(expr: &str) -> Option<DiceExpr> {
    let expr = expr.trim().to_lowercase();
    // NdS+M 또는 NdS-M 또는 NdS
    let d_pos = expr.find('d')?;
    let count: i32 = expr[..d_pos].parse().ok()?;

    let rest = &expr[d_pos + 1..];
    if let Some(plus) = rest.find('+') {
        let sides: i32 = rest[..plus].parse().ok()?;
        let modifier: i32 = rest[plus + 1..].parse().ok()?;
        Some(DiceExpr::new(count, sides, modifier))
    } else if let Some(minus) = rest.find('-') {
        let sides: i32 = rest[..minus].parse().ok()?;
        let modifier: i32 = -(rest[minus + 1..].parse::<i32>().ok()?);
        Some(DiceExpr::new(count, sides, modifier))
    } else {
        let sides: i32 = rest.parse().ok()?;
        Some(DiceExpr::new(count, sides, 0))
    }
}

// =============================================================================
// [2] 가중 랜덤 (원본: rnd.c rn2_on_rng 확장)
// =============================================================================

/// [v2.27.0 R15-4] 가중 랜덤 선택
pub fn weighted_choice(weights: &[i32], rng: &mut NetHackRng) -> usize {
    let total: i32 = weights.iter().sum();
    if total <= 0 {
        return 0;
    }
    let mut roll = rng.rn2(total);
    for (i, &w) in weights.iter().enumerate() {
        roll -= w;
        if roll < 0 {
            return i;
        }
    }
    weights.len() - 1
}

// =============================================================================
// [3] 행운 보정 RNG (원본: rnd.c rnl)
// =============================================================================

/// [v2.27.0 R15-4] 행운 보정 랜덤 (원본: rnl)
/// 행운이 양수면 낮은 값 쪽으로 편향 (좋은 결과)
pub fn rnl(max: i32, luck: i32, rng: &mut NetHackRng) -> i32 {
    let mut result = rng.rn2(max.max(1));
    let adj = luck.clamp(-5, 5);
    result -= adj;
    result.clamp(0, max - 1)
}

/// [v2.27.0 R15-4] 백분율 판정 (행운 보정)
pub fn luck_check(base_chance: i32, luck: i32, rng: &mut NetHackRng) -> bool {
    let adjusted = (base_chance + luck * 3).clamp(1, 99);
    rng.rn2(100) < adjusted
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_basic() {
        let d = DiceExpr::new(2, 6, 0);
        assert_eq!(d.min(), 2);
        assert_eq!(d.max(), 12);
        assert!((d.expected() - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_dice_modifier() {
        let d = DiceExpr::new(1, 8, 3);
        assert_eq!(d.min(), 4);
        assert_eq!(d.max(), 11);
    }

    #[test]
    fn test_dice_roll() {
        let d = DiceExpr::new(3, 6, 0);
        let mut rng = NetHackRng::new(42);
        let r = d.roll(&mut rng);
        assert!(r >= 3 && r <= 18);
    }

    #[test]
    fn test_parse_dice() {
        assert_eq!(parse_dice("2d6"), Some(DiceExpr::new(2, 6, 0)));
        assert_eq!(parse_dice("1d8+3"), Some(DiceExpr::new(1, 8, 3)));
        assert_eq!(parse_dice("3d4-1"), Some(DiceExpr::new(3, 4, -1)));
    }

    #[test]
    fn test_weighted_choice() {
        let weights = vec![10, 30, 60];
        let mut rng = NetHackRng::new(42);
        let mut counts = [0; 3];
        for _ in 0..100 {
            counts[weighted_choice(&weights, &mut rng)] += 1;
        }
        // 가중치 60이 가장 많아야
        assert!(counts[2] > counts[0]);
    }

    #[test]
    fn test_rnl_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..50 {
            let v = rnl(20, 3, &mut rng);
            assert!(v >= 0 && v < 20);
        }
    }

    #[test]
    fn test_luck_check() {
        let mut rng = NetHackRng::new(42);
        let mut successes = 0;
        for s in 0..30 {
            let mut r = NetHackRng::new(s);
            if luck_check(50, 5, &mut r) {
                successes += 1;
            }
        }
        assert!(successes > 15); // 행운 +5로 65% 기대
    }
}
