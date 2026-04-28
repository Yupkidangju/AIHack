use rand::{rngs::StdRng, RngCore, SeedableRng};

/// [v0.1.0] 모든 난수 접근은 이 wrapper를 통과해야 한다.
/// 이렇게 해야 후속 시스템이 seed 기반 재현성을 공유하고, rand 직접 사용이 퍼지는 일을 막을 수 있다.
#[derive(Debug, Clone)]
pub struct GameRng {
    seed: u64,
    inner: StdRng,
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            inner: StdRng::seed_from_u64(seed),
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
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
