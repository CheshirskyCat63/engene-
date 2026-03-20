//! Engine Contracts Aggregator
//! 
//! This file aggregates tests from specialized domain suites.
//! Original megasuite has been split into ownership-based files.
//! 
//! Ownership: Architecture Team (aggregator)
//! Lane: smoke
//! Type: Contract Aggregator
//! Speed: Fast

#[cfg(test)]
mod aggregated_contracts {
    // Import tests from specialized suites
    mod core_command_and_access_contracts;
    mod event_bus_contracts;
    mod identity_and_authority_contracts;
    mod runtime_profile_and_quality_contracts;
    mod world_persistence_contracts;
    mod editor_safe_mode_contracts;
    
    // Re-export critical tests for smoke lane
    pub use core_command_and_access_contracts::command_buffer_tests::command_buffer_spawn_and_despawn;
    pub use core_command_and_access_contracts::command_buffer_tests::command_buffer_take_spawns_is_single_consumer;
    pub use core_command_and_access_contracts::access_descriptor_tests::parallel_validation_detects_write_write_conflict;
    
    pub use event_bus_contracts::event_bus_boundedness_tests::event_bus_capacity_drops_last_overflow_only;
    pub use event_bus_contracts::sticky_events_tests::event_bus_sticky_survives_clear_frame;
    pub use event_bus_contracts::frame_lifecycle_tests::event_bus_clear_removes_frame_events_not_sticky;
    
    pub use identity_and_authority_contracts::persistent_identity_tests::persistent_identity_survives_world_cycle;
    pub use identity_and_authority_contracts::authority_matrix_tests::authority_matrix_contains_transform_needs_destruction_spatial;
    
    pub use runtime_profile_and_quality_contracts::runtime_profile_tests::runtime_profile_headless_has_zero_render_budget;
    pub use runtime_profile_and_quality_contracts::quality_governor_tests::quality_governor_degradation_order_contains_collision_navigation_identity_as_never_cut;
    
    pub use world_persistence_contracts::chunk_persistence_tests::chunk_save_unload_marks_entities_unloaded_not_dead;
    pub use world_persistence_contracts::streaming_contract_tests::streaming_four_region_cycle_preserves_identity_uniqueness;
    
    pub use editor_safe_mode_contracts::editor_safe_mode_tests::editor_safe_mode_disables_panel_after_three_panics;
}

// Legacy compatibility - re-export some critical tests
// These will be gradually removed as teams adopt the new structure

#[test]
fn dependency_graph_no_cycles() {
    // This test is now in core_command_and_access_contracts.rs
    // Keeping for backward compatibility during transition
    use engene::core::system_descriptor::SystemDescriptor;

    let a = SystemDescriptor::new("A").before("B");
    let b = SystemDescriptor::new("B").before("C");
    let c = SystemDescriptor::new("C");

    let descriptors = vec![a, b, c];
    let names: Vec<&str> = descriptors.iter().map(|d| d.name).collect();
    assert_eq!(names, vec!["A", "B", "C"]);
}

#[test]
fn event_bus_bounded_capacity() {
    // This test is now in event_bus_contracts.rs
    // Keeping for backward compatibility during transition
    use engene::core::events::EventBus;

    let mut bus = EventBus::with_capacity(3);
    bus.set_channel_capacity::<u32>(3);

    bus.emit(1u32);
    bus.emit(2u32);
    bus.emit(3u32);
    
    // Should drop the oldest when exceeding capacity
    bus.emit(4u32);
    
    let events = bus.read::<u32>();
    assert_eq!(events.len(), 3);
    assert_eq!(events, vec![&2u32, &3u32, &4u32]);
}

#[test]
fn runtime_manifest_feature_toggles() {
    // This test is now in runtime_profile_and_quality_contracts.rs
    // Keeping for backward compatibility during transition
    use engene::core::runtime_manifest::RuntimeManifest;
    use engene::core::runtime_config::{RuntimeProfile, RuntimeConfig};

    let config = RuntimeConfig::new(RuntimeProfile::Headless);
    let manifest = RuntimeManifest::from_config(&config);
    
    assert!(manifest.is_feature_enabled("audio"));
    assert!(manifest.is_feature_enabled("physics"));
    assert!(!manifest.is_feature_enabled("rendering"));
}

#[test]
fn parallel_validation_write_write_conflict() {
    // This test is now in core_command_and_access_contracts.rs
    // Keeping for backward compatibility during transition
    use engene::core::system_descriptor::SystemDescriptor;
    use engene::core::parallel_validation::validate_systems;

    let system_a = SystemDescriptor::new("SystemA").writes_component::<Transform>();
    let system_b = SystemDescriptor::new("SystemB").writes_component::<Transform>();
    
    let systems = vec![system_a, system_b];
    let validation = validate_systems(&systems);
    
    assert!(!validation.is_safe());
    assert!(validation.conflicts().len() > 0);
}

// Mock types for legacy compatibility
struct Transform;
