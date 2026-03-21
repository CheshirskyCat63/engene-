//! World Tick Wiring - TRANSITIONAL
//! Phase 3: Now re-exports from engine_runtime
//!
//! This module is transitional - the actual WorldTickSystem logic
//! has been moved to engine_runtime. This file provides backward
//! compatibility for root consumers.

pub use engine_runtime::simulation_core::systems::EngineSystem;
pub use engine_runtime::simulation_core::systems::Scheduler;
pub use engine_runtime::simulation_core::systems::WorldTickSystem;

// Re-export context types for transitional compatibility
pub use engine_core::system::FixedTickContext as WorldTickContext;

// Note: The full implementation with WorldGrid, ResourceGrid, biome types
// remains in root because those types are still in root (world::components).
// When world types move to engine_world, this wiring can be fully replaced.
//
// For now, root maintains game-specific world tick logic in:
// - ecosystem::environment
// - ecosystem::migration
// - background module
