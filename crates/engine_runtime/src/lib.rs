pub mod phase;
pub mod simulation_core;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";

    pub use crate::simulation_core;
}

// ================================================================================
// PHASE API - Canonical phase execution contract
// ================================================================================
pub use phase::{
    Phase, PhaseContext, PhaseResult, PhaseTrait,
    validate_phase_order, run_all_phases,
};
pub use phase::tick::run_tick;

// Re-export systems for transitional access from root
pub use simulation_core::systems::{EngineSystem, FixedTickContext, Scheduler, WorldTickSystem};

