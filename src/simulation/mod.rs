//! Simulation subsystem - world tick, LOD simulation, and background processes.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `simulation`, `world_tick` - production, core simulation
//! - `simulation_level` - production, LOD management
//! - `background_world`, `camp_simulation` - production, background sim

pub mod abstraction_invariants;
pub mod activation;
pub mod camp_simulation;
pub mod role_simulation;
pub mod simulation;
pub mod simulation_level;
pub mod social_propagation;
pub mod world_milestones;

pub mod time_events;
