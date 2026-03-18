pub mod aggregation;
pub mod canonical;
pub mod debug_bus;
pub mod render_bus;
pub mod sim_bus;
pub mod sticky;
pub mod tracing_hooks;

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BusId {
    Sim,
    Render,
    Debug,
    Tooling,
}

struct TypedChannel {
    events: Vec<Box<dyn Any + Send + Sync>>,
    capacity: usize,
    dropped_count: u64,
    total_emitted: u64,
}

impl TypedChannel {
    fn new(capacity: usize) -> Self {
        Self {
            events: Vec::new(),
            capacity,
            dropped_count: 0,
            total_emitted: 0,
        }
    }

    fn push(&mut self, event: Box<dyn Any + Send + Sync>) {
        self.total_emitted += 1;
        if self.events.len() >= self.capacity {
            self.dropped_count += 1;
            return;
        }
        self.events.push(event);
    }
}

pub struct EventBus {
    channels: HashMap<TypeId, TypedChannel>,
    sticky: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    default_capacity: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            sticky: HashMap::new(),
            default_capacity: 4096,
        }
    }

    pub fn with_capacity(default_capacity: usize) -> Self {
        Self {
            channels: HashMap::new(),
            sticky: HashMap::new(),
            default_capacity,
        }
    }

    pub fn set_channel_capacity<E: 'static>(&mut self, capacity: usize) {
        let channel = self
            .channels
            .entry(TypeId::of::<E>())
            .or_insert_with(|| TypedChannel::new(self.default_capacity));
        channel.capacity = capacity;
    }

    pub fn emit<E: 'static + Send + Sync>(&mut self, event: E) {
        let cap = self.default_capacity;
        let channel = self
            .channels
            .entry(TypeId::of::<E>())
            .or_insert_with(|| TypedChannel::new(cap));
        channel.push(Box::new(event));
    }

    pub fn emit_boxed(&mut self, event: Box<dyn Any + Send + Sync>) {
        let type_id = (*event).type_id();
        let cap = self.default_capacity;
        let channel = self
            .channels
            .entry(type_id)
            .or_insert_with(|| TypedChannel::new(cap));
        channel.push(event);
    }

    pub fn read<E: 'static>(&self) -> Vec<&E> {
        self.channels
            .get(&TypeId::of::<E>())
            .map(|ch| {
                ch.events
                    .iter()
                    .filter_map(|e| e.downcast_ref::<E>())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn has<E: 'static>(&self) -> bool {
        self.channels
            .get(&TypeId::of::<E>())
            .map_or(false, |ch| !ch.events.is_empty())
    }

    pub fn count<E: 'static>(&self) -> usize {
        self.channels
            .get(&TypeId::of::<E>())
            .map_or(0, |ch| ch.events.len())
    }

    pub fn dropped_count<E: 'static>(&self) -> u64 {
        self.channels
            .get(&TypeId::of::<E>())
            .map_or(0, |ch| ch.dropped_count)
    }

    pub fn total_emitted<E: 'static>(&self) -> u64 {
        self.channels
            .get(&TypeId::of::<E>())
            .map_or(0, |ch| ch.total_emitted)
    }

    pub fn total_dropped(&self) -> u64 {
        self.channels.values().map(|ch| ch.dropped_count).sum()
    }

    pub fn set_sticky<E: 'static + Send + Sync>(&mut self, event: E) {
        self.sticky.insert(TypeId::of::<E>(), Box::new(event));
    }

    pub fn get_sticky<E: 'static>(&self) -> Option<&E> {
        self.sticky
            .get(&TypeId::of::<E>())
            .and_then(|e| e.downcast_ref::<E>())
    }

    pub fn clear_sticky<E: 'static>(&mut self) {
        self.sticky.remove(&TypeId::of::<E>());
    }

    pub fn clear(&mut self) {
        for channel in self.channels.values_mut() {
            channel.events.clear();
        }
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_and_read() {
        let mut bus = EventBus::new();

        bus.emit(42u32);
        bus.emit(100u32);
        bus.emit("hello".to_string());

        let u32_events = bus.read::<u32>();
        assert_eq!(u32_events.len(), 2);
        assert!(u32_events.contains(&&42));
        assert!(u32_events.contains(&&100));

        let str_events = bus.read::<String>();
        assert_eq!(str_events.len(), 1);
        assert_eq!(str_events[0], "hello");
    }

    #[test]
    fn test_count_and_has() {
        let mut bus = EventBus::new();

        assert_eq!(bus.count::<i32>(), 0);
        assert!(!bus.has::<i32>());

        bus.emit(1i32);
        bus.emit(2i32);

        assert_eq!(bus.count::<i32>(), 2);
        assert!(bus.has::<i32>());
    }

    #[test]
    fn test_capacity_overflow_drops_events() {
        let mut bus = EventBus::with_capacity(2);
        bus.set_channel_capacity::<u32>(2);

        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32); // Should be dropped
        bus.emit(4u32); // Should be dropped

        assert_eq!(bus.count::<u32>(), 2);
        assert_eq!(bus.dropped_count::<u32>(), 2);
        assert_eq!(bus.total_emitted::<u32>(), 4);
    }

    #[test]
    fn test_sticky_events() {
        let mut bus = EventBus::new();

        bus.set_sticky(999u64);

        assert_eq!(bus.get_sticky::<u64>(), Some(&999));
        assert_eq!(bus.get_sticky::<i32>(), None);

        bus.clear_sticky::<u64>();
        assert_eq!(bus.get_sticky::<u64>(), None);
    }

    #[test]
    fn test_clear() {
        let mut bus = EventBus::new();

        bus.emit(1u32);
        bus.emit(2u32);
        bus.set_sticky(999u64);

        bus.clear();

        assert_eq!(bus.count::<u32>(), 0);
        // Sticky should remain after clear()
        assert_eq!(bus.get_sticky::<u64>(), Some(&999));
    }

    #[test]
    fn test_multiple_event_types() {
        let mut bus = EventBus::new();

        bus.emit(1u8);
        bus.emit(2u16);
        bus.emit(3u32);
        bus.emit(4u64);

        assert_eq!(bus.count::<u8>(), 1);
        assert_eq!(bus.count::<u16>(), 1);
        assert_eq!(bus.count::<u32>(), 1);
        assert_eq!(bus.count::<u64>(), 1);
        assert_eq!(bus.channel_count(), 4);
    }
}
