//! Doctor diagnostics - tools-only functionality.
//!
//! OWNER: sdk_app / engine_tools
//! This is purely tooling functionality. Not needed in game runtime.

use super::editor_shell::{Diagnostic, DiagnosticSeverity, DoctorReport};

/// Doctor mode for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorMode {
    /// Advisory mode - report issues but don't fail
    Advisory,
    /// Strict mode - fail on any issue
    Strict,
}

/// Run doctor diagnostics on the engine.
/// This is tools-only functionality.
pub fn run_doctor(_engine: &super::Engine, mode: DoctorMode) -> DoctorReport {
    let mut report = DoctorReport::new();
    
    // TODO: Run actual diagnostics
    // - Check for missing resources
    // - Validate ECS consistency
    // - Check spatial index integrity
    // - Validate persistence state
    
    match mode {
        DoctorMode::Strict => {
            if report.error_count > 0 {
                // In strict mode, errors would cause failure
            }
        }
        DoctorMode::Advisory => {
            // Just report
        }
    }
    
    report
}

/// Stub for engine reference in tools.
pub struct Engine;
