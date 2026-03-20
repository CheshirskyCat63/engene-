//! Core subsystem - COMPATIBILITY HUB
//!
//! This module provides re-exports from canonical crates for backward compatibility.
//! Components have moved to dedicated crates:
//! - ECS → engine_ecs
//! - Runtime/system orchestration → engine_runtime  
//! - Core policies/config → engine_core
//! - Tools/debug → engine_tools
//! - SDK → sdk_app
//! - World → engine_world
//! - Content → engine_content

// RE-EXPORTS from canonical crates
pub use engine_ecs::access;
pub use engine_ecs::commands;
pub use engine_ecs::parallel_validation;
pub use engine_ecs::persistent_id;
pub use engine_ecs::sparse_set;
pub use engine_ecs::system_descriptor;

pub use engine_core::data_policy;
pub use engine_core::determinism_policy;
pub use engine_core::deterministic_merge;
pub use engine_core::failure_taxonomy;
pub use engine_core::integration_matrix;
pub use engine_core::metrics_registry;
pub use engine_core::runtime_config;
pub use engine_core::time;

pub use engine_startup::BuildManifest;
