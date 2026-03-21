//! Game role owner crate.
//! Headless runtime ownership has moved to runtime_headless.
//! This crate is reserved for future game-specific launch paths.

pub mod api {
    pub const CRATE: &str = "game_framework";
    pub const STATUS: &str = "game_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "game_framework";
    pub const TARGET_OWNER: &str = "game_framework";
}
