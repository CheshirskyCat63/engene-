//! Physics resources - runtime state owned by the physics capability.

/// Runtime state for physics capability.
#[derive(Debug, Clone, Default)]
pub struct PhysicsRuntimeState {
    /// Whether physics is enabled.
    pub enabled: bool,
}

/// Handle to the physics world/simulation.
#[derive(Debug, Clone, Default)]
pub struct PhysicsWorldHandle {
    /// Whether the physics world has been initialized.
    pub initialized: bool,
}

/// Minimal state that represents whether physics systems were registered.
#[derive(Debug, Clone, Default)]
pub struct PhysicsRegistrationState {
    /// Whether physics systems were registered into the engine.
    pub systems_registered: bool,
}

/// Configuration for physics simulation.
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Fixed timestep for physics simulation.
    pub timestep: f32,
    /// Gravity vector.
    pub gravity: [f32; 3],
    /// Maximum number of substeps per frame.
    pub max_substeps: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            timestep: 1.0 / 20.0,
            gravity: [0.0, -9.81, 0.0],
            max_substeps: 4,
        }
    }
}
