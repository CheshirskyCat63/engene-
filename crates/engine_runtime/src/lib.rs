pub mod simulation_core;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";

    pub use crate::simulation_core;
}
