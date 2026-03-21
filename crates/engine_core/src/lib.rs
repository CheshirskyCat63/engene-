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

// REAL CORE MODULES ONLY
pub mod data_policy;
pub mod determinism_policy;
pub mod deterministic_merge;
pub mod failure_taxonomy;
pub mod time;

// REMOVED FAT MODULES:
// budget_registry, build_manifest, config, determinism_audit, dirty_set,
// game_config, integration_matrix, metrics_registry, mutation_policy,
// ownership_map, plugin, profiler, quality_governor, registry,
// replay, runtime_config, runtime_manifest, serialization
// 
// These belong in framework crates, not core.
