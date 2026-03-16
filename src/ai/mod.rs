//! AI subsystem - goal-driven behavior for NPCs and monsters.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration
//!
//! ## Modules
//! - `ai`, `npc_ai`, `monster_ai` - production, core AI tick
//! - `desire`, `thresholds`, `emotions` - production, need processing
//! - `memory`, `plan` - production, learning/planning
//! - `ai_scheduler` - production, budget-aware tick staggering
//! - `decision`, `goals` - deprecated, replaced by scoring_ai (feature-gated)
//! - `offline_simulation` - experimental, L1/L2 simulation

pub mod ai;
pub mod ai_scheduler;
pub mod body;
pub mod observability;
pub mod combat;
pub mod combat_tactics;
pub mod desire;
pub mod emotions;
pub mod groups;
pub mod memory;
pub mod monster_ai;
pub mod needs;
pub mod npc_ai;
pub mod offline_simulation;
pub mod perception;
pub mod plan;
pub mod reproduction;
pub mod social;
pub mod thresholds;
pub mod traits;
pub mod witness;

// Deprecated: scoring_ai feature-gated alternative
#[cfg(feature = "scoring_ai")]
pub mod decision;
#[cfg(feature = "scoring_ai")]
pub mod goals;
