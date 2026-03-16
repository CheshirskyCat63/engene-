//! Economy subsystem - trading, jobs, and resource management.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration
//!
//! ## Modules
//! - `economy`, `trading` - production, core economy
//! - `jobs`, `resource_flow` - production, job system

pub mod economy;
pub mod item_registry;
pub mod jobs;
pub mod resource_flow;
pub mod trader_economy;
pub mod trading;
