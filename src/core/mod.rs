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

pub mod budget_registry;
pub mod failure_taxonomy;
pub mod component_registry;
pub mod config;
pub mod content_validation;
pub mod ecs;
pub mod engine;
pub mod persistent_id;
pub mod events;
pub mod game_config;
pub mod job_graph;
pub mod job_topology_report;
pub mod material_truth;
pub mod metrics_registry;
pub mod plugin;
pub mod profiler;
pub mod quality_governor;
pub mod query;
pub mod registry;
pub mod scheduler;
pub mod sparse_set;
pub mod integration_matrix;
pub mod integration_systems;
pub mod system;
pub mod systems;
pub mod time;

pub mod access;
pub mod async_services;
pub mod commands;
pub mod data_policy;
pub mod debug;
pub mod determinism_audit;
pub mod jobs;
pub mod mutation_policy;
pub mod ownership_map;
pub mod perf;
pub mod replay;
pub mod runtime_config;
pub mod runtime_manifest;
pub mod parallel_validation;
pub mod sdk;
pub mod serialization;
pub mod system_descriptor;
pub mod world_state_authority;
pub mod build_manifest;
pub mod crash_telemetry;
pub mod determinism_policy;
pub mod deterministic_merge;
pub mod dirty_set;
pub mod hot_reload;
