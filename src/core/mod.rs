//! Core subsystem - TEMPORARY COMPATIBILITY LAYER
//! 
//! This module is being dismantled. Components are moving to dedicated crates:
//! - ECS → engine_ecs
//! - Runtime/system orchestration → engine_runtime  
//! - Core policies/config → engine_core
//! - Tools/debug → engine_tools
//! - SDK → sdk_app
//! - AI → engine_ai (future) or game_framework
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

// LEGACY MODULES - being moved/deleted
// TODO: Move each module to its appropriate crate

pub mod ai_emotions;     // → engine_ai or game_framework
pub mod ai_memory;       // → engine_ai or game_framework  
pub mod ai_plan;         // → engine_ai or game_framework

pub mod budget_registry;      // → engine_core
pub mod component_registry;   // → engine_ecs
pub mod config;               // → engine_core
pub mod content_validation;   // → engine_content

pub mod ecs;              // DELETE - use engine_ecs
pub mod ecs_internal;     // DELETE - use engine_ecs
pub mod engine;           // DELETE - split between engine_core/engine_runtime

pub mod events;           // → engine_core
pub mod game_config;      // → engine_core
pub mod job_graph;        // → engine_runtime
pub mod job_topology_report; // → engine_tools
pub mod material_truth;        // → engine_world
pub mod plugin;           // → engine_core
pub mod quality_governor;     // → engine_core

pub mod query;            // DELETE - use engine_ecs
pub mod scheduler;        // DELETE - use engine_runtime (deprecated)
pub mod system;           // DELETE - use engine_runtime (deprecated)
pub mod systems;          // DELETE - use engine_runtime

pub mod async_services;       // → engine_runtime
pub mod debug;                 // → engine_tools
pub mod determinism_audit;    // → engine_core
pub mod jobs;                 // → engine_runtime
pub mod mutation_policy;      // → engine_core
pub mod perf;                 // → engine_runtime
pub mod replay;               // → engine_core
pub mod sdk;                  // → sdk_app
pub mod serialization;        // → engine_core
pub mod world_state_authority; // → engine_world

pub mod crash_telemetry;    // → engine_tools
pub mod dirty_set;          // → engine_core
pub mod hot_reload;         // → engine_tools
