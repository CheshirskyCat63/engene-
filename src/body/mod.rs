//! Body simulation subsystem.
//!
//! # Status: experimental
//! # Integration: conditional (feature: body_sim)
//! # Tests: unit
//!
//! This module is gated behind `#[cfg(feature = "body_sim")]`.
//! For v1.0, body simulation is optional and can be disabled for performance.

#[cfg(feature = "body_sim")]
pub mod anatomy;
#[cfg(feature = "body_sim")]
pub mod blood;
#[cfg(feature = "body_sim")]
pub mod body_damage;
#[cfg(feature = "body_sim")]
pub mod body_response;
#[cfg(feature = "body_sim")]
pub mod body_store;
#[cfg(feature = "body_sim")]
pub mod body_system;
#[cfg(feature = "body_sim")]
pub mod death_pipeline;
#[cfg(feature = "body_sim")]
pub mod dismemberment;
#[cfg(feature = "body_sim")]
pub mod gore;

/// Stub for non-body_sim builds.
#[cfg(not(feature = "body_sim"))]
pub struct BodySimStub;

#[cfg(not(feature = "body_sim"))]
impl BodySimStub {
    pub fn new() -> Self {
        Self
    }
}
