//! Engine core = tiny kernel.
//! No runtime assembly, no ECS container, no physics, no AI, no tooling.
//! Only: time, determinism, failure classification, data policy.

pub mod api {
    pub use crate::data_policy;
    pub use crate::determinism_policy;
    pub use crate::deterministic_merge;
    pub use crate::failure_taxonomy;
    pub use crate::time;
}

pub mod budget_registry;
pub mod build_manifest;
pub mod config;
pub mod data_policy;
pub mod deterministic_merge;
pub mod determinism_audit;
pub mod determinism_policy;
pub mod dirty_set;
pub mod failure_taxonomy;
pub mod game_config;
pub mod integration_matrix;
pub mod metrics_registry;
pub mod mutation_policy;
pub mod ownership_map;
pub mod plugin;
pub mod profiler;
pub mod quality_governor;
pub mod registry;
pub mod replay;
pub mod runtime_config;
pub mod runtime_manifest;
pub mod serialization;
pub mod time;
