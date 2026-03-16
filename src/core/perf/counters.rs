use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct PerfCounters {
    counters: HashMap<&'static str, AtomicU64>,
}

impl PerfCounters {
    pub fn new() -> Self {
        Self { counters: HashMap::new() }
    }

    pub fn increment(&self, name: &'static str) {
        if let Some(c) = self.counters.get(name) {
            c.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn add(&self, name: &'static str, value: u64) {
        if let Some(c) = self.counters.get(name) {
            c.fetch_add(value, Ordering::Relaxed);
        }
    }

    pub fn register(&mut self, name: &'static str) {
        self.counters.entry(name).or_insert_with(|| AtomicU64::new(0));
    }

    pub fn read(&self, name: &str) -> u64 {
        self.counters.get(name).map_or(0, |c| c.load(Ordering::Relaxed))
    }

    pub fn reset_all(&self) {
        for c in self.counters.values() {
            c.store(0, Ordering::Relaxed);
        }
    }
}

impl Default for PerfCounters {
    fn default() -> Self { Self::new() }
}
