use std::any::TypeId;

pub struct EventTrace {
    pub event_type: TypeId,
    pub event_name: &'static str,
    pub tick: u64,
    pub count: usize,
}

pub struct EventTracer {
    traces: Vec<EventTrace>,
    enabled: bool,
}

impl EventTracer {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn record(
        &mut self,
        event_type: TypeId,
        event_name: &'static str,
        tick: u64,
        count: usize,
    ) {
        if !self.enabled {
            return;
        }
        self.traces.push(EventTrace {
            event_type,
            event_name,
            tick,
            count,
        });
    }

    pub fn drain(&mut self) -> Vec<EventTrace> {
        std::mem::take(&mut self.traces)
    }
}

impl Default for EventTracer {
    fn default() -> Self {
        Self::new()
    }
}
