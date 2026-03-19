//! Engine Physics - Physics runtime capability for ENGENE.
//!
//! This crate provides the physics capability layer. It is a capability, not a runtime role.
//! Physics can be enabled in any runtime mode (game, SDK, headless, tools).

pub mod api;
pub mod bootstrap;
pub mod contracts;
pub mod resources;
pub mod systems;

// moved from root
pub mod physics;
pub mod animation;
