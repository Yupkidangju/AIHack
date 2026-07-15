use aihack::core::{GameSession, RunState};

/// Integration test가 production mutation API를 우회하지 않고 필요한 run state를 구성한다.
pub struct SessionBuilder {
    seed: u64,
    run_state: RunState,
}

impl SessionBuilder {
    pub fn playing(seed: u64) -> Self {
        Self {
            seed,
            run_state: RunState::Playing,
        }
    }

    pub fn run_state(mut self, run_state: RunState) -> Self {
        self.run_state = run_state;
        self
    }

    pub fn build(self) -> GameSession {
        let session = GameSession::new_for_playing(self.seed);
        let mut save = session.to_save_data();
        save.run_state = self.run_state;
        GameSession::from_save_data(save).expect("test fixture save data must be valid")
    }
}
