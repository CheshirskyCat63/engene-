//! Physics bootstrap contracts - defines how physics connects to the platform.

/// Bootstrap mode for physics capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsBootstrapMode {
    /// Physics is not enabled for this runtime.
    Disabled,
    /// Physics is enabled with minimal configuration.
    Enabled,
}

/// Failure modes during physics bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsBootstrapFailure {
    /// Runtime state resource is missing when physics is enabled.
    MissingRuntimeState,
    /// World handle resource is missing when physics is enabled.
    MissingWorldHandle,
    /// Physics system registration is missing when physics is enabled.
    InvalidRegistration,
}

impl PhysicsBootstrapFailure {
    /// Returns a human-readable description of the failure.
    pub fn description(&self) -> &'static str {
        match self {
            Self::MissingRuntimeState => "PhysicsRuntimeState resource is missing",
            Self::MissingWorldHandle => "PhysicsWorldHandle resource is missing",
            Self::InvalidRegistration => "Physics systems were not registered",
        }
    }
}
