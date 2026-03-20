//! Transitional game role facade.
//! Current canonical operator entrypoint is still the root `engene_game` bin.
//! This crate exists as the future Game role owner, but is not that owner yet.
pub mod api {
pub const CRATE: &str = "game_framework";
pub const STATUS: &str = "transitional_role_facade";
pub const CURRENT_OPERATOR_OWNER: &str = "root bin engene_game";
pub const TARGET_OWNER: &str = "game_framework";
}
