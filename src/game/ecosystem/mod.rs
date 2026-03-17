//! Ecosystem subsystem - food chain, migration, and territory.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `food_chain`, `migration` - production, ecosystem dynamics
//! - `territory`, `environment` - production, world state

pub mod environment;
pub mod food_chain;
pub mod migration;
pub mod territory;
