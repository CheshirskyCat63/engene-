pub mod simulation_core;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";

    pub use crate::simulation_core;
}

// Re-export systems for transitional access from root
pub use simulation_core::systems::{EngineSystem, WorldTickSystem, Scheduler, FixedTickContext};
