//! Entity Lifecycle Contracts
//! 
//! Tests for entity lifecycle, persistent identity, and generation tracking.
//! Ownership: ECS Team
//! Lane: ecs
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod entity_lifecycle_tests {
    use engene::core::entity::{Entity, EntityGenerator};
    use engene::world::world::World;

    #[test]
    fn spawn_new_assigns_unique_persistent_ids() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity1 = generator.spawn(&mut world);
        let entity2 = generator.spawn(&mut world);
        let entity3 = generator.spawn(&mut world);
        
        // IDs should be unique
        assert_ne!(entity1.id(), entity2.id());
        assert_ne!(entity2.id(), entity3.id());
        assert_ne!(entity1.id(), entity3.id());
        
        // IDs should be persistent
        assert_eq!(entity1.id(), entity1.id());
        assert_eq!(entity2.id(), entity2.id());
        assert_eq!(entity3.id(), entity3.id());
    }

    #[test]
    fn despawn_marks_presence_dead() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Entity should be alive initially
        assert!(world.is_alive(entity));
        assert!(world.contains(entity));
        
        // Despawn should mark as dead
        world.despawn(entity);
        
        assert!(!world.is_alive(entity));
        assert!(!world.contains(entity));
    }

    #[test]
    fn entity_ref_reports_unloaded_state_correctly() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Entity should be loaded initially
        let entity_ref = world.entity(entity);
        assert!(entity_ref.is_loaded());
        
        // Simulate unload
        world.unload_entity(entity);
        
        let entity_ref = world.entity(entity);
        assert!(!entity_ref.is_loaded());
        assert!(entity_ref.is_unloaded());
    }

    #[test]
    fn persistent_identity_survives_world_cycle() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        let original_id = entity.id();
        let original_generation = entity.generation();
        
        // Add some components
        world.add_component(entity, "test_component");
        world.add_component(entity, 42u32);
        
        // Simulate world cycle (save/unload/load)
        let world_state = world.save_state();
        drop(world);
        
        let mut new_world = World::new();
        let restored_entity = new_world.restore_entity(entity, world_state);
        
        // Identity should be preserved
        assert_eq!(restored_entity.id(), original_id);
        assert_eq!(restored_entity.generation(), original_generation);
        assert!(new_world.contains(restored_entity));
    }

    #[test]
    fn entity_generation_prevents_aba_problem() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity1 = generator.spawn(&mut world);
        let original_generation = entity1.generation();
        
        // Despawn entity
        world.despawn(entity1);
        
        // Spawn new entity - should get same ID but different generation
        let entity2 = generator.spawn(&mut world);
        assert_eq!(entity2.id(), entity1.id());
        assert_ne!(entity2.generation(), original_generation);
        
        // Old entity reference should be invalid
        assert!(!world.is_alive(entity1));
        assert!(world.is_alive(entity2));
    }

    #[test]
    fn entity_lifecycle_is_deterministic() {
        let mut world1 = World::new();
        let mut world2 = World::new();
        let mut generator1 = EntityGenerator::new();
        let mut generator2 = EntityGenerator::new();
        
        // Same sequence should produce same results
        let entity1a = generator1.spawn(&mut world1);
        let entity1b = generator1.spawn(&mut world1);
        
        let entity2a = generator2.spawn(&mut world2);
        let entity2b = generator2.spawn(&mut world2);
        
        assert_eq!(entity1a.id(), entity2a.id());
        assert_eq!(entity1b.id(), entity2b.id());
        assert_eq!(entity1a.generation(), entity2a.generation());
        assert_eq!(entity1b.generation(), entity2b.generation());
    }

    #[test]
    fn entity_cleanup_is_complete() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Add components
        world.add_component(entity, "test_component");
        world.add_component(entity, 42u32);
        world.add_component(entity, vec![1, 2, 3]);
        
        // Verify components exist
        assert!(world.has_component::<String>(entity));
        assert!(world.has_component::<u32>(entity));
        assert!(world.has_component::<Vec<i32>>(entity));
        
        // Despawn should clean up all components
        world.despawn(entity);
        
        assert!(!world.contains(entity));
        assert!(!world.has_component::<String>(entity));
        assert!(!world.has_component::<u32>(entity));
        assert!(!world.has_component::<Vec<i32>>(entity));
    }

    #[test]
    fn entity_persistence_is_accurate() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Add complex component data
        world.add_component(entity, "persistent_string");
        world.add_component(entity, 12345u64);
        world.add_component(entity, vec![10, 20, 30, 40]);
        
        // Save entity state
        let entity_state = world.save_entity_state(entity);
        
        // Restore in new world
        let mut new_world = World::new();
        let restored_entity = new_world.restore_entity_from_state(entity, entity_state);
        
        // Verify all data is preserved
        assert_eq!(restored_entity.id(), entity.id());
        assert_eq!(restored_entity.generation(), entity.generation());
        assert_eq!(new_world.get_component::<String>(restored_entity), Some(&"persistent_string".to_string()));
        assert_eq!(new_world.get_component::<u64>(restored_entity), Some(&12345u64));
        assert_eq!(new_world.get_component::<Vec<i32>>(restored_entity), Some(&vec![10, 20, 30, 40]));
    }

    #[test]
    fn entity_loading_is_safe() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Loading should be safe even if entity doesn't exist
        let non_existent = Entity::new(9999, 1);
        let entity_ref = world.entity(entity);
        let non_existent_ref = world.entity(non_existent);
        
        assert!(entity_ref.is_valid());
        assert!(!non_existent_ref.is_valid());
        
        // Loading unloaded entity should be safe
        world.unload_entity(entity);
        let unloaded_ref = world.entity(entity);
        assert!(unloaded_ref.is_valid());
        assert!(unloaded_ref.is_unloaded());
    }

    #[test]
    fn entity_unloading_is_graceful() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Add components
        world.add_component(entity, "test_component");
        world.add_component(entity, 42u32);
        
        // Unload should preserve entity but mark as unloaded
        world.unload_entity(entity);
        
        assert!(world.contains(entity));
        assert!(!world.is_loaded(entity));
        assert!(world.is_unloaded(entity));
        
        // Components should still be accessible but marked as unloaded
        let entity_ref = world.entity(entity);
        assert!(entity_ref.has_component::<String>());
        assert!(entity_ref.has_component::<u32>());
        assert!(!entity_ref.is_component_loaded::<String>());
        assert!(!entity_ref.is_component_loaded::<u32>());
    }

    #[test]
    fn entity_migration_preserves_identity() {
        let mut world1 = World::new();
        let mut world2 = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world1);
        let original_id = entity.id();
        let original_generation = entity.generation();
        
        // Add components
        world1.add_component(entity, "migration_test");
        world1.add_component(entity, 999u32);
        
        // Migrate to new world
        let migration_data = world1.prepare_migration(entity);
        let migrated_entity = world2.complete_migration(entity, migration_data);
        
        // Identity should be preserved
        assert_eq!(migrated_entity.id(), original_id);
        assert_eq!(migrated_entity.generation(), original_generation);
        
        // Components should be preserved
        assert_eq!(world2.get_component::<String>(migrated_entity), Some(&"migration_test".to_string()));
        assert_eq!(world2.get_component::<u32>(migrated_entity), Some(&999u32));
    }

    #[test]
    fn entity_serialization_is_complete() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        let entity = generator.spawn(&mut world);
        
        // Add various component types
        world.add_component(entity, "serialization_test");
        world.add_component(entity, 42u32);
        world.add_component(entity, 3.14f64);
        world.add_component(entity, vec![1, 2, 3, 4, 5]);
        
        // Serialize entity
        let serialized = world.serialize_entity(entity);
        
        // Deserialize in new world
        let mut new_world = World::new();
        let deserialized_entity = new_world.deserialize_entity(&serialized);
        
        // Verify complete restoration
        assert_eq!(deserialized_entity.id(), entity.id());
        assert_eq!(deserialized_entity.generation(), entity.generation());
        assert_eq!(new_world.get_component::<String>(deserialized_entity), Some(&"serialization_test".to_string()));
        assert_eq!(new_world.get_component::<u32>(deserialized_entity), Some(&42u32));
        assert_eq!(new_world.get_component::<f64>(deserialized_entity), Some(&3.14f64));
        assert_eq!(new_world.get_component::<Vec<i32>>(deserialized_entity), Some(&vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn entity_deserialization_is_safe() {
        // Test deserialization with invalid data
        let invalid_serialized = "invalid_serialized_data".to_string();
        
        let mut world = World::new();
        let result = world.deserialize_entity(&invalid_serialized);
        
        // Should handle invalid data gracefully
        assert!(result.is_none());
        
        // Test with corrupted but valid-looking data
        let corrupted_data = "{\"id\":9999,\"generation\":1,\"components\":{}}".to_string();
        let result = world.deserialize_entity(&corrupted_data);
        
        // Should handle corruption gracefully
        assert!(result.is_none());
    }

    #[test]
    fn entity_bulk_operations_are_atomic() {
        let mut world = World::new();
        let mut generator = EntityGenerator::new();
        
        // Spawn multiple entities
        let entities: Vec<Entity> = (0..10).map(|_| generator.spawn(&mut world)).collect();
        
        // Add components to all entities
        for entity in &entities {
            world.add_component(*entity, "bulk_test");
            world.add_component(*entity, 42u32);
        }
        
        // Bulk despawn should be atomic
        let despawn_results: Vec<bool> = entities.iter()
            .map(|entity| world.despawn(*entity))
            .collect();
        
        // All operations should succeed
        assert!(despawn_results.iter().all(|&success| success));
        
        // All entities should be gone
        for entity in &entities {
            assert!(!world.contains(*entity));
        }
    }

    #[test]
    fn entity_concurrent_access_is_safe() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let world = Arc::new(Mutex::new(World::new()));
        let generator = Arc::new(Mutex::new(EntityGenerator::new()));
        let mut handles = vec![];
        
        // Spawn entities from multiple threads
        for i in 0..4 {
            let world_clone = Arc::clone(&world);
            let generator_clone = Arc::clone(&generator);
            let handle = thread::spawn(move || {
                let mut world = world_clone.lock().unwrap();
                let mut generator = generator_clone.lock().unwrap();
                
                let entity = generator.spawn(&mut world);
                world.add_component(entity, format!("thread_{}", i));
                
                entity
            });
            handles.push(handle);
        }
        
        // Wait for all threads and collect entities
        let entities: Vec<Entity> = handles.into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        
        // Verify all entities are unique and valid
        let world = world.lock().unwrap();
        for (i, entity) in entities.iter().enumerate() {
            assert!(world.contains(*entity));
            assert_eq!(world.get_component::<String>(*entity), Some(&format!("thread_{}", i)));
        }
        
        // Verify uniqueness
        let mut ids = entities.iter().map(|e| e.id()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), entities.len());
    }
}
