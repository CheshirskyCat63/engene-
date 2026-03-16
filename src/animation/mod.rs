//! Animation subsystem.
//!
//! # Status: partial
//! # Integration: enabled
//! # Tests: unit
//!
//! Core animation modules are always enabled.
//! Advanced animation (ragdoll, IK) is gated behind `#[cfg(feature = "advanced_anim")]`.

// Core animation - always enabled
pub mod animation;
pub mod animation_ladder;
pub mod clip_map;
pub mod injury_animation;
pub mod animation_integration;
pub mod locomotion;
pub mod micro_motion;
pub mod procedural;

// Advanced animation - feature-gated
#[cfg(feature = "advanced_anim")]
pub mod active_ragdoll;
#[cfg(feature = "advanced_anim")]
pub mod foot_ik;
#[cfg(feature = "advanced_anim")]
pub mod ragdoll;
