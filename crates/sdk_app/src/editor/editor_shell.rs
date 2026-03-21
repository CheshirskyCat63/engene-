//! Editor shell - editor-only UI and tooling surface.
//!
//! OWNER: sdk_app
//! This is SDK/editor-only. Must NOT be used by game runtime.

use std::sync::Arc;

/// Editor shell state - contains all editor UI state.
/// This is purely SDK/editor functionality.
pub struct EditorShell {
    /// Last doctor report to display in editor
    pub last_doctor_report: Option<DoctorReport>,
    /// Current selection in scene hierarchy
    pub selected_entity: Option<u32>,
    /// Scene hierarchy visibility
    pub show_scene_hierarchy: bool,
    /// Inspector visibility
    pub show_inspector: bool,
    /// Console visibility
    pub show_console: bool,
    /// Profiler visibility
    pub show_profiler: bool,
    /// Debug overlay visibility
    pub show_debug_overlay: bool,
}

impl Default for EditorShell {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorShell {
    pub fn new() -> Self {
        Self {
            last_doctor_report: None,
            selected_entity: None,
            show_scene_hierarchy: true,
            show_inspector: true,
            show_console: false,
            show_profiler: false,
            show_debug_overlay: false,
        }
    }
    
    /// Update editor shell - called every frame in editor mode.
    pub fn update(&mut self) {
        // TODO: Update editor state
        // - Process pending inspector edits
        // - Update scene hierarchy
        // - Refresh dashboard data
    }
    
    /// Apply pending edits from inspector.
    pub fn apply_inspector_edits(&mut self, _ecs: &mut super::Ecs) {
        // TODO: Apply pending entity edits
    }
    
    /// Update dashboards with current state.
    /// OWNER: sdk_app::editor
    /// This is editor-only - does NOT mutate world truth.
    pub fn update_dashboards(&mut self) {
        // TODO: Update performance dashboard
        // TODO: Update entity count dashboard
        // TODO: Update memory dashboard
        // This is purely display state - no world mutation
    }
}

/// Doctor report for editor display.
pub struct DoctorReport {
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl DoctorReport {
    pub fn new() -> Self {
        Self {
            error_count: 0,
            warning_count: 0,
            diagnostics: Vec::new(),
        }
    }
    
    pub fn print(&self) {
        println!("=== DOCTOR REPORT ===");
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        for diag in &self.diagnostics {
            println!("  {:?}", diag);
        }
    }
}

impl Default for DoctorReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}
