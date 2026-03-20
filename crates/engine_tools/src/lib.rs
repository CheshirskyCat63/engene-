//! Transitional tools role facade.
//! Current canonical operator entrypoint is still the root `engene_tools` bin.
//! This crate exists as the future tools role owner, but is not that owner yet.
pub mod api {
pub const CRATE: &str = "engine_tools";
pub const STATUS: &str = "transitional_role_facade";
pub const CURRENT_OPERATOR_OWNER: &str = "root bin engene_tools";
pub const TARGET_OWNER: &str = "engine_tools";
}
