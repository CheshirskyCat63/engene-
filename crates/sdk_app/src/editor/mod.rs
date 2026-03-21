//! Editor module - SDK/editor ownership.
//!
//! OWNER: sdk_app
//! This module contains all editor-only logic that must NOT leak into game runtime.
//!
//! ## What belongs here
//! - Editor shell state and update
//! - Inspector mutation paths
//! - Dashboard updates
//! - Editor UI surfaces
//!
//! ## What does NOT belong here
//! - Simulation truth (engine_world)
//! - Phase ordering (engine_runtime)
//! - Render implementation (engine_render)
//! - Audio implementation (engine_audio)

pub mod editor_shell;
pub mod doctor;

// Re-export for external use
pub use editor_shell::{DoctorReport, EditorShell};

/// Update editor state - called every frame in editor mode.
/// OWNER: sdk_app (this is the first extraction from sdk_runner)
/// 
/// This replaces the old call: `engene::app::sdk_runner::sdk_runner_phases::editor::run(self)`
pub fn update_editor(editor_shell: &mut EditorShell, _engine: &mut doctor::Engine) {
    // Update dashboards
    editor_shell.update_dashboards();
    
    // Apply pending inspector edits would go here
    // (requires proper ECS reference)
}

/// Initialize editor with doctor report from startup.
/// This is called once at editor startup.
pub fn init_editor_with_doctor_report(editor_shell: &mut EditorShell, report: DoctorReport) {
    editor_shell.last_doctor_report = Some(report);
}
