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

pub mod data_policy;
pub mod deterministic_merge;
pub mod failure_taxonomy;
pub mod time;

// Determinism policy - kept in kernel for runtime contract
pub mod determinism_policy;
