//! Transitional SDK role facade.
//! Current canonical operator entrypoint is still the root `engene_sdk` bin.
//! This crate exists as the future SDK role owner, but is not that owner yet.
pub mod api {
pub const CRATE: &str = "sdk_app";
pub const STATUS: &str = "transitional_role_facade";
pub const CURRENT_OPERATOR_OWNER: &str = "root bin engene_sdk";
pub const TARGET_OWNER: &str = "sdk_app";
}
