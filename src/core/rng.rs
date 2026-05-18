use rand::{rngs::StdRng, RngCore, SeedableRng};

/// [v0.1.0] 모든 난수 접근은 이 wrapper를 통과해야 한다.
/// 이렇게 해야 후속 시스템이 seed 기반 재현성을 공유하고, rand 직접 사용이 퍼지는 일을 막을 수 있다.
#[derive(Debug, Clone)]
pub struct GameRng {
    seed: u64,
    draws: u64,
    inner: StdRng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    fn 같은_seed는_같은_sequence를_만든다() {
        let mut a = GameRng::new(42);
        let mut b = GameRng::new(42);

        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn 다른_seed는_다른_sequence를_만든다() {
        let mut a = GameRng::new(42);
        let mut b = GameRng::new(43);
        let differs = (0..10).any(|_| a.next_u64() != b.next_u64());

        assert!(differs);
    }
}
