use super::EventBus;

pub struct RenderBus {
    pub bus: EventBus,
}

impl RenderBus {
    pub fn new() -> Self {
        Self { bus: EventBus::with_capacity(1024) }
    }
}

impl Default for RenderBus {
    fn default() -> Self { Self::new() }
}
