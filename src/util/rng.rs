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
}
