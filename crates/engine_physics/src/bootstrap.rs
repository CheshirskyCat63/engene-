//! Physics bootstrap - defines how physics connects to the platform.

use crate::contracts::{PhysicsBootstrapFailure, PhysicsBootstrapMode};
use crate::resources::{PhysicsRegistrationState, PhysicsRuntimeState, PhysicsWorldHandle};

/// Physics bootstrap context.
/// This is the entry point for connecting physics to a runtime.
#[derive(Debug, Clone)]
pub struct PhysicsBootstrap {
    /// Bootstrap mode.
    pub mode: PhysicsBootstrapMode,
    /// Runtime state resource.
    pub runtime_state: PhysicsRuntimeState,
    /// World handle resource.
    pub world_handle: PhysicsWorldHandle,
    /// Registration state (systems registered into engine).
    pub registration: PhysicsRegistrationState,
}

impl PhysicsBootstrap {
    /// Create a disabled physics bootstrap (no physics).
    pub fn disabled() -> Self {
        Self {
            mode: PhysicsBootstrapMode::Disabled,
            runtime_state: PhysicsRuntimeState { enabled: false },
            world_handle: PhysicsWorldHandle { initialized: false },
            registration: PhysicsRegistrationState {
                systems_registered: false,
            },
        }
    }

    /// Create an enabled minimal physics bootstrap.
    pub fn enabled_minimal() -> Self {
        Self {
            mode: PhysicsBootstrapMode::Enabled,
            runtime_state: PhysicsRuntimeState { enabled: true },
            world_handle: PhysicsWorldHandle { initialized: true },
            registration: PhysicsRegistrationState {
                systems_registered: true,
            },
        }
    }

    /// Validate the bootstrap configuration.
    /// Returns Ok if physics can be started, Err with failure reason otherwise.
    pub fn validate(&self) -> Result<(), PhysicsBootstrapFailure> {
        if self.mode == PhysicsBootstrapMode::Enabled && !self.runtime_state.enabled {
            return Err(PhysicsBootstrapFailure::MissingRuntimeState);
        }
        if self.mode == PhysicsBootstrapMode::Enabled && !self.world_handle.initialized {
            return Err(PhysicsBootstrapFailure::MissingWorldHandle);
        }
        if self.mode == PhysicsBootstrapMode::Enabled && !self.registration.systems_registered {
            return Err(PhysicsBootstrapFailure::InvalidRegistration);
        }
        Ok(())
    }

    /// Check if physics is enabled.
    pub fn is_enabled(&self) -> bool {
        self.mode == PhysicsBootstrapMode::Enabled
    }
}

impl Default for PhysicsBootstrap {
    fn default() -> Self {
        Self::disabled()
    }
}
