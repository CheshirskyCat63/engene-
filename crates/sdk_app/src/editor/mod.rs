//! Editor module - SDK/editor ownership.
//!
//! OWNER: sdk_app
//! This module contains all editor-only logic that must NOT leak into game runtime.

pub mod editor_shell;
pub mod doctor;

pub use editor_shell::EditorShell;
