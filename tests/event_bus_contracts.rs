//! Event Bus Contracts
//! 
//! Tests for event bus boundedness, sticky semantics, frame lifecycle, and delivery.
//! Ownership: Core Events Team
//! Lane: Contracts  
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod event_bus_boundedness_tests {
    use engene::core::events::EventBus;

    #[test]
    fn event_bus_capacity_drops_last_overflow_only() {
        let mut bus = EventBus::with_capacity(3);
        bus.set_channel_capacity::<u32>(3);

        // Fill to capacity
        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32);
        
        // Overflow - should drop oldest
        bus.emit(4u32);

        assert_eq!(bus.count::<u32>(), 3);
        assert_eq!(bus.dropped_count::<u32>(), 1);
        
        // Verify correct items kept (2,3,4) and (1) dropped
        let events: Vec<u32> = bus.drain().collect();
        assert_eq!(events, vec![2, 3, 4]);
    }

    #[test]
    fn event_bus_bounded_channel_isolation() {
        let mut bus = EventBus::new();
        
        // Set different capacities for different event types
        bus.set_channel_capacity::<u32>(2);
        bus.set_channel_capacity::<String>(5);
        
        // Fill u32 channel beyond capacity
        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32); // Should drop 1
        
        // Fill String channel within capacity
        bus.emit("a".to_string());
        bus.emit("b".to_string());
        bus.emit("c".to_string());
        
        assert_eq!(bus.count::<u32>(), 2);
        assert_eq!(bus.dropped_count::<u32>(), 1);
        assert_eq!(bus.count::<String>(), 3);
        assert_eq!(bus.dropped_count::<String>(), 0);
    }

    #[test]
    fn event_bus_capacity_preserves_order() {
        let mut bus = EventBus::with_capacity(3);
        bus.set_channel_capacity::<u32>(3);

        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32);
        bus.emit(4u32); // Drops 1, keeps 2,3,4

        let events: Vec<u32> = bus.drain().collect();
        assert_eq!(events, vec![2, 3, 4], "order should be preserved after overflow");
    }
}

#[cfg(test)]
mod sticky_semantics_tests {
    use engene::core::events::EventBus;

    #[test]
    fn event_bus_sticky_survives_clear_frame() {
        let mut bus = EventBus::new();

        #[derive(Debug, PartialEq)]
        struct Config(u32);

        bus.set_sticky(Config(42));
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config(42)));

        bus.clear();
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config(42)), "sticky should survive clear");
    }

    #[test]
    fn event_bus_sticky_overwrite_behavior() {
        let mut bus = EventBus::new();

        #[derive(Debug, PartialEq)]
        struct Config(String);

        bus.set_sticky(Config("initial".to_string()));
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config("initial".to_string())));

        // Overwrite sticky
        bus.set_sticky(Config("updated".to_string()));
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config("updated".to_string())));
    }

    #[test]
    fn event_bus_multiple_sticky_types() {
        let mut bus = EventBus::new();

        #[derive(Debug, PartialEq)]
        struct ConfigA(u32);
        #[derive(Debug, PartialEq)]
        struct ConfigB(String);

        bus.set_sticky(ConfigA(100));
        bus.set_sticky(ConfigB("test".to_string()));

        assert_eq!(bus.get_sticky::<ConfigA>(), Some(&ConfigA(100)));
        assert_eq!(bus.get_sticky::<ConfigB>(), Some(&ConfigB("test".to_string())));

        bus.clear();

        assert_eq!(bus.get_sticky::<ConfigA>(), Some(&ConfigA(100)));
        assert_eq!(bus.get_sticky::<ConfigB>(), Some(&ConfigB("test".to_string())));
    }
}

#[cfg(test)]
mod clear_frame_lifecycle_tests {
    use engene::core::events::EventBus;

    #[test]
    fn event_bus_clear_removes_frame_events_not_sticky() {
        let mut bus = EventBus::new();
        
        // Add frame events
        bus.emit(100u32);
        bus.emit(200u32);
        
        // Add sticky event
        bus.set_sticky(String::from("persistent"));

        bus.clear();

        // Frame events should be gone
        assert_eq!(bus.count::<u32>(), 0);
        
        // Sticky should remain
        assert_eq!(
            bus.get_sticky::<String>(),
            Some(&String::from("persistent"))
        );
    }

    #[test]
    fn event_bus_clear_resets_dropped_counters() {
        let mut bus = EventBus::with_capacity(2);
        bus.set_channel_capacity::<u32>(2);

        // Cause some drops
        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32); // drops 1
        bus.emit(4u32); // drops 2

        assert_eq!(bus.dropped_count::<u32>(), 2);

        bus.clear();

        // Dropped counters should reset
        assert_eq!(bus.dropped_count::<u32>(), 0);
    }

    #[test]
    fn event_bus_frame_events_isolation() {
        let mut bus = EventBus::new();
        
        // Add multiple types of frame events
        bus.emit(1u32);
        bus.emit("test".to_string());
        bus.emit(true);
        
        assert_eq!(bus.count::<u32>(), 1);
        assert_eq!(bus.count::<String>(), 1);
        assert_eq!(bus.count::<bool>(), 1);
        
        bus.clear();
        
        // All frame events should be cleared
        assert_eq!(bus.count::<u32>(), 0);
        assert_eq!(bus.count::<String>(), 0);
        assert_eq!(bus.count::<bool>(), 0);
    }
}

#[cfg(test)]
mod canonical_event_delivery_tests {
    use engene::core::events::EventBus;

    #[test]
    fn event_bus_canonical_delivery_order() {
        let mut bus = EventBus::new();
        
        // Emit events in specific order
        bus.emit(1u32);
        bus.emit(2u32);
        bus.emit(3u32);
        
        // Should deliver in same order
        let events: Vec<u32> = bus.drain().collect();
        assert_eq!(events, vec![1, 2, 3], "delivery order should match emit order");
    }

    #[test]
    fn event_bus_no_delivery_if_empty() {
        let mut bus = EventBus::new();
        
        // Empty bus should return empty iterator
        let events: Vec<u32> = bus.drain().collect();
        assert_eq!(events, vec![]);
        
        // Count should be zero
        assert_eq!(bus.count::<u32>(), 0);
    }

    #[test]
    fn event_bus_delivery_is_consumptive() {
        let mut bus = EventBus::new();
        
        bus.emit(1u32);
        bus.emit(2u32);
        
        // First delivery should consume
        let events1: Vec<u32> = bus.drain().collect();
        assert_eq!(events1, vec![1, 2]);
        
        // Second delivery should be empty
        let events2: Vec<u32> = bus.drain().collect();
        assert_eq!(events2, vec![]);
    }

    #[test]
    fn event_bus_mixed_event_types_delivery() {
        let mut bus = EventBus::new();
        
        bus.emit(42u32);
        bus.emit("test".to_string());
        bus.emit(true);
        
        // Each type should be delivered separately
        let ints: Vec<u32> = bus.drain().collect();
        let strings: Vec<String> = bus.drain().collect();
        let bools: Vec<bool> = bus.drain().collect();
        
        assert_eq!(ints, vec![42]);
        assert_eq!(strings, vec!["test"]);
        assert_eq!(bools, vec![true]);
    }
}

#[cfg(test)]
mod integration_tests {
    use engene::core::events::EventBus;

    #[test]
    fn event_bus_complex_lifecycle() {
        let mut bus = EventBus::with_capacity(3);
        bus.set_channel_capacity::<u32>(3);

        // Set sticky config
        #[derive(Debug, PartialEq)]
        struct Config(u32);
        bus.set_sticky(Config(999));

        // Add frame events
        bus.emit(1u32);
        bus.emit(2u32);
        
        // Overflow
        bus.emit(3u32);
        bus.emit(4u32); // drops 1

        assert_eq!(bus.count::<u32>(), 3);
        assert_eq!(bus.dropped_count::<u32>(), 1);
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config(999)));

        // Clear frame events
        bus.clear();
        
        assert_eq!(bus.count::<u32>(), 0);
        assert_eq!(bus.dropped_count::<u32>(), 0);
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config(999)));

        // Add new frame events
        bus.emit(5u32);
        bus.emit(6u32);
        
        let events: Vec<u32> = bus.drain().collect();
        assert_eq!(events, vec![5, 6]);
        assert_eq!(bus.get_sticky::<Config>(), Some(&Config(999)));
    }
}
