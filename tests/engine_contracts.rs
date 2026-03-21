//! Engine Test Aggregator
//! 
//! Thin aggregator that re-exports all specialized test suites.
//! Ownership: Architecture Team
//! Lane: smoke
//! Type: Aggregator
//! Speed: Fast

#[cfg(test)]
mod aggregated_contracts {
    // Import tests from specialized suites
    pub use core_command_and_access_contracts::*;
    pub use event_bus_contracts::*;
    pub use identity_and_authority_contracts::*;
    pub use runtime_profile_and_quality_contracts::*;
    pub use world_persistence_contracts::*;
    pub use editor_safe_mode_contracts::*;
    pub use ecs_lifecycle_contracts::*;
    pub use ecs_authority_contracts::*;
    pub use ecs_performance_contracts::*;
    pub use camera_contracts::*;
    pub use rendering_pipeline_contracts::*;
    pub use editor_console_contracts::*;
    pub use editor_inspector_contracts::*;
    pub use engine_lifecycle_contracts::*;
    pub use world_integration_contracts::*;
    pub use simulation_integration_contracts::*;
    pub use performance_governance_contracts::*;
    pub use multithreading_performance_contracts::*;
    pub use physics_chain_reaction_contracts::*;
    pub use navigation_integration_contracts::*;
    pub use architectural_gates::*;
    pub use architecture_validation::*;
    
    // Re-export critical tests for smoke lane
    pub use core_command_and_access_contracts::command_buffer_tests::command_buffer_spawn_and_despawn;
    pub use core_command_and_access_contracts::command_buffer_tests::command_buffer_take_spawns_is_single_consumer;
    pub use core_command_and_access_contracts::access_descriptor_tests::parallel_validation_detects_write_write_conflict;

    pub use event_bus_contracts::event_bus_boundedness_tests::event_bus_capacity_drops_last_overflow_only;
    pub use event_bus_contracts::sticky_events_tests::event_bus_sticky_survives_clear_frame;
    pub use event_bus_contracts::frame_lifecycle_tests::event_bus_clear_removes_frame_events_not_sticky;

    pub use ecs_lifecycle_contracts::entity_lifecycle_tests::entity_spawn_and_despawn;
    pub use ecs_authority_contracts::ownership_tests::single_writer_exclusive_access;

    pub use architectural_gates::ownership_enforcement_tests::all_test_files_have_canonical_headers;
    pub use architecture_validation::architecture_validation_tests::no_files_over_500_lines;
}

