use std::sync::{Arc, Mutex};
use zkcg_common::state::ProtocolState;

#[derive(Clone)]
pub struct StateStore {
    inner: Arc<Mutex<ProtocolState>>,
}

impl StateStore {
    pub fn new(state: ProtocolState) -> Self {
        Self {
            inner: Arc::new(Mutex::new(state)),
        }
    }

    pub fn load(&self) -> ProtocolState {
        self.inner.lock().unwrap().clone()
    }

    pub fn save(&self, state: ProtocolState) {
        *self.inner.lock().unwrap() = state;
    }
}
