use super::EventBus;

pub struct SimBus {
    pub bus: EventBus,
}

impl SimBus {
    pub fn new() -> Self {
        Self { bus: EventBus::with_capacity(2048) }
    }
}

impl Default for SimBus {
    fn default() -> Self { Self::new() }
}
