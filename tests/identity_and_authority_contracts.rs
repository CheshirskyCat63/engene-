//! Identity and Authority Contracts
//! 
//! Tests for persistent identity, authority matrix, and ownership boundaries.
//! Ownership: Core Identity Team
//! Lane: Contracts
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod persistent_identity_tests {
    use engene::core::ecs::World;

    #[test]
    fn spawn_new_assigns_unique_persistent_ids() {
        let world = World::new();
        
        let mut ids = std::collections::HashSet::new();
        
        // Spawn 1000 entities and verify uniqueness
        for _ in 0..1000 {
            let entity = world.spawn();
            let id = entity.id();
            
            // ID should be unique
            assert!(!ids.contains(&id), "ID {} should be unique", id);
            ids.insert(id);
            
            // Entity should be alive
            assert!(world.is_alive(entity));
        }
        
        // Should have exactly 1000 unique IDs
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn despawn_marks_presence_dead() {
        let world = World::new();
        
        let entity = world.spawn();
        
        // Entity should be alive initially
        assert!(world.is_alive(entity));
        assert!(world.is_present(entity));
        
        // Despawn entity
        world.despawn(entity);
        
        // Entity should be dead but still present for cleanup
        assert!(!world.is_alive(entity));
        assert!(!world.is_present(entity), "despawned entity should not be present");
    }

    #[test]
    fn entity_ref_reports_unloaded_state_correctly() {
        let world = World::new();
        
        let entity = world.spawn();
        
        // Initially loaded
        assert!(world.is_loaded(entity));
        
        // Simulate unload (would normally happen in streaming scenarios)
        world.mark_unloaded(entity);
        
        // Should report unloaded state
        assert!(!world.is_loaded(entity));
        assert!(world.is_alive(entity), "unloaded entity should still be alive");
    }

    #[test]
    fn persistent_identity_survives_world_cycle() {
        let world = World::new();
        
        let entity = world.spawn();
        let original_id = entity.id();
        
        // Simulate world save/load cycle
        let saved_state = world.save_entity_state(entity);
        
        // Create new world and restore entity
        let new_world = World::new();
        let restored_entity = new_world.restore_entity_state(saved_state);
        
        // ID should be preserved
        assert_eq!(original_id, restored_entity.id());
        assert!(new_world.is_alive(restored_entity));
    }

    #[test]
    fn entity_generation_prevents_aba_problem() {
        let world = World::new();
        
        let entity1 = world.spawn();
        let id1 = entity1.id();
        
        // Despawn entity
        world.despawn(entity1);
        
        // Spawn new entity - should get different generation
        let entity2 = world.spawn();
        let id2 = entity2.id();
        
        // Same index but different generation should prevent ABA
        assert_eq!(id1.index(), id2.index());
        assert_ne!(id1.generation(), id2.generation());
        
        // Old entity reference should be invalid
        assert!(!world.is_alive(entity1));
        assert!(world.is_alive(entity2));
    }
}

#[cfg(test)]
mod authority_matrix_tests {
    use engene::core::world_state_authority;

    #[test]
    fn authority_matrix_contains_transform_needs_destruction_spatial() {
        let matrix = authority_matrix::authority_matrix();
        
        // Transform component should have write authority
        assert!(matrix.has_write_authority("Transform"));
        
        // Destruction system should need transform access
        assert!(matrix.system_needs_component("DestructionSystem", "Transform"));
        
        // Spatial system should need transform access
        assert!(matrix.system_needs_component("SpatialSystem", "Transform"));
    }

    #[test]
    fn authority_matrix_rejects_two_writers_same_resource() {
        let matrix = authority_matrix::authority_matrix();
        
        // Define two systems that both write to Transform
        let system_a = "PhysicsSystem";
        let system_b = "AnimationSystem";
        let resource = "Transform";
        
        // Grant write authority to first system
        matrix.grant_write_authority(system_a, resource);
        
        // Second system should be rejected
        assert!(!matrix.can_grant_write_authority(system_b, resource));
    }

    #[test]
    fn authority_matrix_allows_readers_with_writer() {
        let matrix = authority_matrix::authority_matrix();
        
        let writer = "PhysicsSystem";
        let reader = "RenderingSystem";
        let resource = "Transform";
        
        // Grant write authority
        matrix.grant_write_authority(writer, resource);
        
        // Reader should be allowed
        assert!(matrix.can_grant_read_authority(reader, resource));
    }

    #[test]
    fn authority_matrix_tracks_system_dependencies() {
        let matrix = authority_matrix::authority_matrix();
        
        // Set up dependency chain: A -> B -> C
        matrix.grant_write_authority("SystemA", "Resource1");
        matrix.grant_read_authority("SystemB", "Resource1");
        matrix.grant_write_authority("SystemB", "Resource2");
        matrix.grant_read_authority("SystemC", "Resource2");
        
        // Check dependency tracking
        let deps_a = matrix.get_system_dependencies("SystemA");
        let deps_b = matrix.get_system_dependencies("SystemB");
        let deps_c = matrix.get_system_dependencies("SystemC");
        
        assert!(deps_a.is_empty(), "SystemA should have no dependencies");
        assert_eq!(deps_b.len(), 1, "SystemB should depend on SystemA");
        assert!(deps_b.contains(&"SystemA".to_string()));
        assert_eq!(deps_c.len(), 1, "SystemC should depend on SystemB");
        assert!(deps_c.contains(&"SystemB".to_string()));
    }

    #[test]
    fn authority_matrix_validates_no_cycles() {
        let matrix = authority_matrix::authority_matrix();
        
        // Create a cycle: A -> B -> C -> A
        matrix.grant_write_authority("SystemA", "Resource1");
        matrix.grant_read_authority("SystemB", "Resource1");
        matrix.grant_write_authority("SystemB", "Resource2");
        matrix.grant_read_authority("SystemC", "Resource2");
        matrix.grant_write_authority("SystemC", "Resource3");
        
        // This should detect the cycle when SystemA tries to read Resource3
        assert!(!matrix.can_grant_read_authority("SystemA", "Resource3"));
    }
}

#[cfg(test)]
mod ownership_boundary_tests {
    #[test]
    fn derived_state_rebuild_passes_for_nav_cover_spatial() {
        // Test that derived state (navigation, cover, spatial) rebuilds correctly
        use engene::core::derived_state::DerivedStateManager;
        
        let manager = DerivedStateManager::new();
        
        // Initial state should be clean
        assert!(manager.is_nav_grid_clean());
        assert!(manager.is_cover_map_clean());
        assert!(manager.is_spatial_index_clean());
        
        // Mark spatial dirty
        manager.mark_spatial_dirty();
        
        // Should trigger rebuild chain
        assert!(!manager.is_spatial_index_clean());
        assert!(!manager.is_nav_grid_clean(), "nav should be dirty when spatial is dirty");
        assert!(!manager.is_cover_map_clean(), "cover should be dirty when spatial is dirty");
        
        // Rebuild spatial
        manager.rebuild_spatial_index();
        
        assert!(manager.is_spatial_index_clean());
        assert!(!manager.is_nav_grid_clean(), "nav should still need rebuild");
        assert!(!manager.is_cover_map_clean(), "cover should still need rebuild");
        
        // Rebuild derived systems
        manager.rebuild_nav_grid();
        manager.rebuild_cover_map();
        
        // All should be clean now
        assert!(manager.is_nav_grid_clean());
        assert!(manager.is_cover_map_clean());
        assert!(manager.is_spatial_index_clean());
    }

    #[test]
    fn resource_authority_prevents_unauthorized_access() {
        use engene::core::resource_authority::ResourceAuthority;
        
        let authority = ResourceAuthority::new();
        
        // Grant exclusive access to PhysicsSystem
        authority.grant_exclusive_access("PhysicsWorld", "PhysicsSystem");
        
        // PhysicsSystem should have access
        assert!(authority.has_access("PhysicsWorld", "PhysicsSystem"));
        
        // Other systems should be denied
        assert!(!authority.has_access("PhysicsWorld", "RenderingSystem"));
        assert!(!authority.has_access("PhysicsWorld", "AudioSystem"));
        
        // Release exclusive access
        authority.release_exclusive_access("PhysicsWorld", "PhysicsSystem");
        
        // Access should be denied after release
        assert!(!authority.has_access("PhysicsWorld", "PhysicsSystem"));
    }

    #[test]
    fn component_registry_enforces_ownership_rules() {
        use engene::core::component_registry::ComponentRegistry;
        
        let registry = ComponentRegistry::new();
        
        // Register components with owners
        registry.register::<Transform>("Transform", "PhysicsSystem");
        registry.register::<Velocity>("Velocity", "PhysicsSystem");
        registry.register::<Mesh>("Mesh", "RenderingSystem");
        
        // PhysicsSystem should access its components
        assert!(registry.can_access::<Transform>("PhysicsSystem"));
        assert!(registry.can_access::<Velocity>("PhysicsSystem"));
        assert!(!registry.can_access::<Mesh>("PhysicsSystem"));
        
        // RenderingSystem should access its components
        assert!(registry.can_access::<Mesh>("RenderingSystem"));
        assert!(!registry.can_access::<Transform>("RenderingSystem"));
        assert!(!registry.can_access::<Velocity>("RenderingSystem"));
    }
}

use engene::world::components::Transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Velocity {
    vector: (f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Mesh {
    id: u32,
}
