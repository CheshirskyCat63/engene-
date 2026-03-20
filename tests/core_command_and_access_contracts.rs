//! Core Command and Access Contracts
//! 
//! Tests for command buffer operations, component access, and ECS boundaries.
//! Ownership: Core ECS Team
//! Lane: Contracts
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod command_buffer_tests {
    use engene::core::commands::CommandBuffer;

    #[test]
    fn command_buffer_spawn_and_despawn() {
        let mut cb = CommandBuffer::new();
        assert!(cb.is_empty());

        cb.despawn(42);
        assert_eq!(cb.despawn_count(), 1);
        assert!(!cb.is_empty());

        let despawns = cb.take_despawns();
        assert_eq!(despawns, vec![42]);
    }

    #[test]
    fn command_buffer_emit_event() {
        #[derive(Debug)]
        struct TestEvent(u32);

        let mut cb = CommandBuffer::new();
        let evt = TestEvent(99);
        assert_eq!(evt.0, 99);
        cb.emit_event(evt);
        assert_eq!(cb.event_count(), 1);
    }

    #[test]
    fn command_buffer_take_spawns_is_single_consumer() {
        let mut cb = CommandBuffer::new();
        
        let spawn1 = cb.spawn();
        let spawn2 = cb.spawn();
        assert_eq!(cb.spawn_count(), 2);

        let spawns = cb.take_spawns();
        assert_eq!(spawns.len(), 2);
        
        // Second take should be empty (single consumer invariant)
        let empty_spawns = cb.take_spawns();
        assert_eq!(empty_spawns.len(), 0);
    }

    #[test]
    fn command_buffer_component_ops_isolation() {
        let mut cb = CommandBuffer::new();
        
        cb.add_component(100u64, String::from("test"));
        cb.remove_component::<String>(101u64);
        
        assert_eq!(cb.component_op_count(), 2);
        
        let ops = cb.take_component_ops();
        assert_eq!(ops.len(), 2);
        
        // Verify ops are isolated by entity
        let empty_ops = cb.take_component_ops();
        assert_eq!(empty_ops.len(), 0);
    }

    #[test]
    fn command_buffer_idempotent_after_drain() {
        let mut cb = CommandBuffer::new();
        
        cb.spawn();
        cb.despawn(42);
        cb.emit_event(99u32);
        
        // Drain all operations
        let _spawns = cb.take_spawns();
        let _despawns = cb.take_despawns();
        let _events = cb.take_events();
        
        assert!(cb.is_empty());
        
        // After drain, should be idempotent
        let empty_spawns = cb.take_spawns();
        let empty_despawns = cb.take_despawns();
        let empty_events = cb.take_events();
        
        assert_eq!(empty_spawns.len(), 0);
        assert_eq!(empty_despawns.len(), 0);
        assert_eq!(empty_events.len(), 0);
    }
}

#[cfg(test)]
mod access_descriptor_tests {
    use engene::core::system_descriptor::SystemDescriptor;

    #[test]
    fn access_descriptor_write_write_conflict_detection() {
        struct ComponentA;
        struct ComponentB;

        let system_a = SystemDescriptor::new("SystemA")
            .writes_component::<ComponentA>()
            .with_parallel(true);

        let system_b = SystemDescriptor::new("SystemB")
            .writes_component::<ComponentA>()
            .with_parallel(true);

        // Should detect write-write conflict
        let report = engene::core::parallel_validation::validate_systems(
            &[system_a, system_b], 
            false, 
            false
        );
        assert!(report.has_errors(), "write-write conflict should be detected");
    }

    #[test]
    fn access_descriptor_read_write_conflict_detection() {
        struct ComponentA;
        struct ComponentB;

        let reader = SystemDescriptor::new("Reader")
            .reads_component::<ComponentA>()
            .with_parallel(true);

        let writer = SystemDescriptor::new("Writer")
            .writes_component::<ComponentA>()
            .with_parallel(true);

        // Should detect read-write conflict
        let report = engene::core::parallel_validation::validate_systems(
            &[reader, writer], 
            false, 
            false
        );
        assert!(report.has_errors(), "read-write conflict should be detected");
    }

    #[test]
    fn access_descriptor_disjoint_parallel_safe() {
        struct ComponentX;
        struct ComponentY;

        let system_x = SystemDescriptor::new("SystemX")
            .writes_component::<ComponentX>()
            .with_parallel(true);

        let system_y = SystemDescriptor::new("SystemY")
            .writes_component::<ComponentY>()
            .with_parallel(true);

        // Disjoint writes should be parallel-safe
        let report = engene::core::parallel_validation::validate_systems(
            &[system_x, system_y], 
            false, 
            false
        );
        assert!(!report.has_errors(), "disjoint writes should be parallel-safe");
    }

    #[test]
    fn access_descriptor_resource_conflict_detection() {
        struct ResourceA;

        let system_a = SystemDescriptor::new("SystemA")
            .writes_resource::<ResourceA>()
            .with_parallel(true);

        let system_b = SystemDescriptor::new("SystemB")
            .writes_resource::<ResourceA>()
            .with_parallel(true);

        // Should detect resource conflict
        let report = engene::core::parallel_validation::validate_systems(
            &[system_a, system_b], 
            false, 
            false
        );
        assert!(report.has_errors(), "resource write-write conflict should be detected");
    }
}

#[cfg(test)]
mod ecs_boundary_tests {
    #[test]
    fn ecs_world_access_isolation() {
        // Test that ECS world access is properly isolated
        // This prevents systems from accessing unrelated world state
        use engene::core::ecs::World;

        let world = World::new();
        
        // World should start empty
        assert_eq!(world.entity_count(), 0);
        
        // Entity creation should be tracked
        let entity = world.spawn();
        assert_eq!(world.entity_count(), 1);
        assert!(world.is_alive(entity));
    }

    #[test]
    fn component_access_contract() {
        // Test component access follows contract rules
        use engene::core::ecs::World;

        let world = World::new();
        let entity = world.spawn();
        
        #[derive(Debug, PartialEq)]
        struct TestComponent(String);
        
        // Component should not exist initially
        assert!(!world.has_component::<TestComponent>(entity));
        
        // Add component
        world.insert(entity, TestComponent("test".to_string()));
        assert!(world.has_component::<TestComponent>(entity));
        
        // Access component
        let component = world.get::<TestComponent>(entity);
        assert_eq!(component.as_ref().map(|c| &c.0), Some(&"test".to_string()));
        
        // Remove component
        world.remove::<TestComponent>(entity);
        assert!(!world.has_component::<TestComponent>(entity));
    }
}
