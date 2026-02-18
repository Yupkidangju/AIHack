/// NetHack-RS RNG System
///
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetHackRng {
    _seed: u64,
    //
    //
    state: u64,
}

impl NetHackRng {
    pub fn new(seed: u64) -> Self {
        Self {
            _seed: seed,
            state: seed,
        }
    }

    /// 0 <= RND(x) < x
    ///
    fn rnd_internal(&mut self, x: i32) -> i32 {
        if x <= 0 {
            return 0;
        }

        //
        //
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state % x as u64) as i32
    }

    /// 0 <= rn2(x) < x
    pub fn rn2(&mut self, x: i32) -> i32 {
        self.rnd_internal(x)
    }

    /// 1 <= rnd(x) <= x
    pub fn rnd(&mut self, x: i32) -> i32 {
        self.rnd_internal(x) + 1
    }

    /// y <= rn1(x, y) < x + y
    pub fn rn1(&mut self, x: i32, y: i32) -> i32 {
        self.rn2(x) + y
    }

    ///
    pub fn d(&mut self, n: i32, x: i32) -> i32 {
        let mut n = n;
        let mut tmp = n;
        if x < 0 || n < 0 || (x == 0 && n != 0) {
            return 1;
        }
        while n > 0 {
            tmp += self.rnd_internal(x);
            n -= 1;
        }
        tmp
    }

    /// 1 <= rne(x) <= max(ulevel/3, 5)
    ///
    pub fn rne(&mut self, x: i32, ulevel: i32) -> i32 {
        let utmp = if ulevel < 15 { 5 } else { ulevel / 3 };
        let mut tmp = 1;
        while tmp < utmp && self.rn2(x) == 0 {
            tmp += 1;
        }
        tmp
    }

    ///
    pub fn rnz(&mut self, i: i32, ulevel: i32) -> i32 {
        let mut tmp = 1000i64;
        tmp += self.rn2(1000) as i64;
        tmp *= self.rne(4, ulevel) as i64;

        let mut x = i as i64;
        if self.rn2(2) == 0 {
            x *= tmp;
            x /= 1000;
        } else {
            x *= 1000;
            x /= tmp;
        }
        x as i32
    }

    /// [v2.9.8] 0 <= rnl(x) < x (행운 조정) (원본: rnd.c L116-159)
    /// 행운이 좋으면 0 쪽으로, 나쁘면 (x-1) 쪽으로 결과 편향
    pub fn rnl(&mut self, x: i32, luck: i32) -> i32 {
        if x <= 0 {
            return 0;
        }
        // 작은 범위에서는 행운/3 적용 (0에서 멀어지는 방향으로 반올림)
        let adjustment = if x <= 15 {
            let abs_luck = luck.abs();
            let adj_abs = (abs_luck + 1) / 3;
            if luck >= 0 {
                adj_abs
            } else {
                -adj_abs
            }
        } else {
            luck
        };
        let mut i = self.rnd_internal(x);
        // 37 + |luck| 중 1의 확률로만 보정 비적용
        if adjustment != 0 && self.rn2(37 + adjustment.abs()) != 0 {
            i -= adjustment;
            if i < 0 {
                i = 0;
            } else if i >= x {
                i = x - 1;
            }
        }
        i
    }
}

/// 0 <= rn2_on_display_rng(x) < x
///
pub fn rn2_on_display_rng(x: i32, seed: &mut u32) -> i32 {
    if x <= 0 {
        return 0;
    }
    *seed = seed.wrapping_mul(2739110765);
    ((*seed >> 16) % x as u32) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = NetHackRng::new(42);
        let mut rng2 = NetHackRng::new(42);
        assert_eq!(rng1.rn2(100), rng2.rn2(100));
        assert_eq!(rng1.rnd(10), rng2.rnd(10));
        assert_eq!(rng1.d(3, 6), rng2.d(3, 6));
    }

    #[test]
    fn test_rn2_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..1000 {
            let v = rng.rn2(10);
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_rnd_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..1000 {
            let v = rng.rnd(6);
            assert!(v >= 1 && v <= 6);
        }
    }

    #[test]
    fn test_rn1_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..1000 {
            let v = rng.rn1(10, 5);
            assert!(v >= 5 && v < 15);
        }
    }

    #[test]
    fn test_d_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let v = rng.d(3, 6);
            assert!(v >= 3 && v <= 21); // 3d6: 3~21 + n(=3)
        }
    }

    #[test]
    fn test_d_edge_cases() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(rng.d(-1, 6), 1); // 음수 → fallback
        assert_eq!(rng.d(3, -1), 1); // 음수 x → fallback
    }

    #[test]
    fn test_rne_range() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let v = rng.rne(4, 10);
            assert!(v >= 1 && v <= 5); // ulevel<15 → max 5
        }
    }

    #[test]
    fn test_rne_high_level() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let v = rng.rne(4, 30);
            assert!(v >= 1 && v <= 10); // ulevel/3 = 10
        }
    }

    #[test]
    fn test_rnz_positive() {
        let mut rng = NetHackRng::new(42);
        let v = rng.rnz(100, 10);
        assert!(v > 0); // rnz는 항상 양수 (입력이 양수일 때)
    }

    #[test]
    fn test_rnl_no_luck() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let v = rng.rnl(10, 0);
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_rnl_good_luck() {
        // 행운이 좋으면 0 쪽으로 편향
        let mut sum = 0;
        for seed in 0..1000u64 {
            let mut rng = NetHackRng::new(seed);
            sum += rng.rnl(20, 10);
        }
        let avg = sum as f64 / 1000.0;
        // 행운 10 → avg가 9.5(중립)보다 낮아야 함
        assert!(avg < 9.5);
    }

    #[test]
    fn test_rnl_bad_luck() {
        let mut sum = 0;
        for seed in 0..1000u64 {
            let mut rng = NetHackRng::new(seed);
            sum += rng.rnl(20, -10);
        }
        let avg = sum as f64 / 1000.0;
        assert!(avg > 9.5); // 나쁜 행운 → 높은 값으로 편향
    }

    #[test]
    fn test_rnl_small_range() {
        let mut rng = NetHackRng::new(42);
        // 작은 범위 (x <= 15)에서 luck/3 조정
        for _ in 0..100 {
            let v = rng.rnl(10, 9); // adj = (9+1)/3 = 3
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_rn2_on_display_rng() {
        let mut seed = 1u32;
        for _ in 0..100 {
            let v = rn2_on_display_rng(10, &mut seed);
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_rn2_edge() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(rng.rn2(0), 0); // 0 입력 → 0
        assert_eq!(rng.rn2(-1), 0); // 음수 → 0
        assert_eq!(rng.rn2(1), 0); // x=1 → 항상 0
    }
}
