//! Physics API surface - public types exposed to the platform.

pub use crate::bootstrap::PhysicsBootstrap;
pub use crate::contracts::{PhysicsBootstrapFailure, PhysicsBootstrapMode};
pub use crate::resources::{PhysicsRegistrationState, PhysicsRuntimeState, PhysicsWorldHandle};
pub use crate::systems::PHYSICS_TICK_SYSTEM_NAME;
