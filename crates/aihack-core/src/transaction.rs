/// 검증 전에는 원본을 바꾸지 않는 cloned working-copy transaction이다.
pub struct TurnTransaction<S> {
    working: S,
}

impl<S: Clone> TurnTransaction<S> {
    pub fn prepare(state: &S) -> Self {
        Self {
            working: state.clone(),
        }
    }
}

impl<S> TurnTransaction<S> {
    pub fn working(&self) -> &S {
        &self.working
    }

    pub fn working_mut(&mut self) -> &mut S {
        &mut self.working
    }

    pub fn commit(self) -> S {
        self.working
    }
}
