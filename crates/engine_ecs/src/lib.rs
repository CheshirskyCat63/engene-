pub mod api {
    /// Phase A facade placeholder for engine_ecs.
    pub const CRATE: &str = "engine_ecs";
    pub use crate::commands;
    pub use crate::parallel_validation;
    pub use crate::persistent_id;
    pub use crate::query_contract;
    pub use crate::system_descriptor;
}

pub mod access;
pub mod sparse_set;

pub mod persistent_id;
pub mod system_descriptor;

pub mod commands;
pub mod parallel_validation;

pub mod query_contract;
