use super::EventBus;

pub struct DebugBus {
    pub bus: EventBus,
}

impl DebugBus {
    pub fn new() -> Self {
        Self {
            bus: EventBus::with_capacity(512),
        }
    }
}

impl Default for DebugBus {
    fn default() -> Self {
        Self::new()
    }
}
