//! Core subsystem - ECS, engine, events, and system orchestration.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration
//!
//! ## Modules
//! - `ecs`, `engine`, `events` - production, core framework
//! - `sparse_set`, `commands` - production, data structures
//! - `scheduler`, `registry` - production, system management
//! - `game_config`, `config` - production, configuration
//! - `query` - production, ECS query layer
//! - `metrics_registry` - production, observability (Phase C.7)
//! - `replay`, `determinism_audit` - experimental, determinism
//! - `systems` - production, extracted game systems (Phase A.4)

pub mod ai_emotions;
pub mod ai_memory;
pub mod ai_plan;
pub mod budget_registry;
pub mod component_registry;
pub mod config;
pub mod content_validation;
pub mod ecs;
pub mod ecs_internal;
pub mod engine;
pub mod events;
pub mod game_config;
pub mod job_graph;
pub mod job_topology_report;
pub mod material_truth;
pub mod plugin;
pub mod quality_governor;
pub mod query;
pub mod scheduler;
pub mod system;
pub mod systems;

pub mod async_services;
pub mod debug;
pub mod determinism_audit;
pub mod jobs;
pub mod mutation_policy;
pub mod perf;
pub mod replay;
pub mod sdk;
pub mod serialization;
pub mod world_state_authority;

pub use engine_core::build_manifest;
pub use engine_core::data_policy;
pub use engine_core::determinism_policy;
pub use engine_core::deterministic_merge;
pub use engine_core::failure_taxonomy;
pub use engine_core::integration_matrix;
pub use engine_core::metrics_registry;
pub use engine_core::ownership_map;
pub use engine_core::profiler;
pub use engine_core::registry;
pub use engine_core::runtime_config;
pub use engine_core::runtime_manifest;
pub use engine_core::time;
pub mod crash_telemetry;
pub mod dirty_set;
pub mod hot_reload;

pub use engine_ecs::access;
pub use engine_ecs::sparse_set;

pub use engine_ecs::persistent_id;
pub use engine_ecs::system_descriptor;

pub use engine_ecs::commands;
pub use engine_ecs::parallel_validation;
