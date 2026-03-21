pub mod phase;
pub mod assembly;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";
}

// ================================================================================
// PHASE API - Canonical phase execution contract
// ================================================================================
pub use phase::{
    Phase, PhaseContext, PhaseResult, PhaseTrait,
    validate_phase_order, run_all_phases,
};
pub use phase::tick::run_tick;
pub use phase::streaming::{run_streaming, StreamingInput, StreamingOutput};
pub use phase::persistence::{run_persistence, PersistenceInput, PersistenceOutput};
