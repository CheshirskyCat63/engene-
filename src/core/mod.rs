//! Core subsystem - TEMPORARY COMPATIBILITY LAYER
//! 
//! This module is being dismantled. Components have moved to dedicated crates:
//! - ECS → engine_ecs
//! - Runtime/system orchestration → engine_runtime  
//! - Core policies/config → engine_core
//! - Tools/debug → engine_tools
//! - SDK → sdk_app
//! - World → engine_world
//! - Content → engine_content

// RE-EXPORTS from canonical crates
pub use engine_ecs::access;
pub use engine_ecs::sparse_set;
pub use engine_ecs::persistent_id;
pub use engine_ecs::system_descriptor;
pub use engine_ecs::commands;
pub use engine_ecs::parallel_validation;

pub use engine_core::data_policy;
pub use engine_core::determinism_policy;
pub use engine_core::deterministic_merge;
pub use engine_core::failure_taxonomy;
pub use engine_core::integration_matrix;
pub use engine_core::metrics_registry;
pub use engine_core::runtime_config;
pub use engine_core::time;

pub use engine_startup::BuildManifest;

// REMOVED MODULES - moved to dedicated crates
// ✓ ai_emotions, ai_memory, ai_plan → engine_world
// ✓ budget_registry, config, game_config, plugin, quality_governor → engine_core
// ✓ component_registry → engine_ecs
// ✓ content_validation → engine_content
// ✓ job_graph, async_services, jobs, perf → engine_runtime
// ✓ job_topology_report, crash_telemetry, hot_reload, debug → engine_tools
// ✓ determinism_audit, mutation_policy, serialization, dirty_set, replay → engine_core
// ✓ material_truth, world_state_authority, events → engine_world
// ✓ sdk → sdk_app

// DEPRECATED MODULES - to be deleted
pub mod ecs;              // DELETE - use engine_ecs
pub mod ecs_internal;     // DELETE - use engine_ecs
pub mod engine;           // DELETE - split between engine_core/engine_runtime
pub mod query;            // DELETE - use engine_ecs
pub mod scheduler;        // DELETE - use engine_runtime (deprecated)
pub mod system;           // DELETE - use engine_runtime (deprecated)
pub mod systems;          // DELETE - use engine_runtime
