//! Physics systems - minimal registration surface for physics.

/// Name of the physics tick system registered into the engine.
#[allow(non_upper_case_globals)]
pub const PHYSICS_TICK_SYSTEM_NAME: &str = "PhysicsTickSystem";

/// Placeholder for physics tick system.
pub struct PhysicsTickSystem;

impl PhysicsTickSystem {
    /// Create a new physics tick system.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PhysicsTickSystem {
    fn default() -> Self {
        Self::new()
    }
}
