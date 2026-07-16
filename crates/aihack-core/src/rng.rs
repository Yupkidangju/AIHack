use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

/// 모든 난수 접근은 이 wrapper를 통과해 seed 기반 재현성을 공유한다.
#[derive(Debug, Clone)]
pub struct GameRng {
    seed: u64,
    draws: u64,
    inner: StdRng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngStateV1 {
    pub seed: u64,
    pub draws: u64,
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            draws: 0,
            inner: StdRng::seed_from_u64(seed),
        }
    }

    pub fn from_state(state: RngStateV1) -> Self {
        let mut rng = Self::new(state.seed);
        for _ in 0..state.draws {
            let _ = rng.next_u64();
        }
        rng
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn next_u64(&mut self) -> u64 {
        self.draws += 1;
        self.inner.next_u64()
    }

    pub fn snapshot_state(&self) -> RngStateV1 {
        RngStateV1 {
            seed: self.seed,
            draws: self.draws,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameRng;
    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = GameRng::new(42);
        let mut b = GameRng::new(42);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
