//! Networking subsystem - NOT PART OF v1.0
//!
//! # Status: planned
//! # Integration: disabled (feature-gated)
//! # Tests: none
//!
//! This module is explicitly excluded from v1.0 scope.
//! All networking code is gated behind `#[cfg(feature = "networking")]`.
//!
//! For post-1.0 roadmap, see docs/canonical/ENGENE_2_0_ROADMAP.md.

#[cfg(feature = "networking")]
pub mod client;
#[cfg(feature = "networking")]
pub mod interpolation;
#[cfg(feature = "networking")]
pub mod net_markers;
#[cfg(feature = "networking")]
pub mod network_system;
#[cfg(feature = "networking")]
pub mod protocol;
#[cfg(feature = "networking")]
pub mod server;

/// Stub type for non-networking builds.
#[cfg(not(feature = "networking"))]
pub struct NetworkStub;

#[cfg(not(feature = "networking"))]
impl NetworkStub {
    pub fn new() -> Self {
        Self
    }
}
