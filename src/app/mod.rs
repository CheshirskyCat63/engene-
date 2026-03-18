//! Application layer - game/tools/headless runners and platform integration.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `game_runner` - game app lifecycle and event loop
//! - `headless_runner` - deterministic kernel/headless runtime
//! - `sdk_runner` - SDK workstation runtime
//! - `tools_runner` - tools diagnostics runtime

pub mod game_runner;
pub mod headless_runner;
pub mod sdk_runner;
pub mod tools_runner;
