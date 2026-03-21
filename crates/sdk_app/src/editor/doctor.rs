//! Doctor diagnostics - tools-only functionality.
//!
//! ## OWNER STATUS: UNRESOLVED - TEMPORARY DEBT
//! Current: dual owner candidate (sdk_app / engine_tools)
//! Target: engine_tools (doctor is diagnostic tool, not editor UI)
//! This file is TEMPORARY until owner decision is finalized.
//! 
//! This is purely tooling functionality. Not needed in game runtime.

use super::editor_shell::DoctorReport;
// TODO: Use Diagnostic and DiagnosticSeverity when doctor implementation is complete
// use super::editor_shell::{Diagnostic, DiagnosticSeverity};

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
/// Note: Engine reference would be proper type in real implementation
pub fn run_doctor(_engine: &Engine, mode: DoctorMode) -> DoctorReport {
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
