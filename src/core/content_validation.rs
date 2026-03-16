use crate::world::material_bridge::{MaterialBridge, ValidationMode};
use crate::world::surface_db::SurfaceDB;

pub struct ContentValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ContentValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_errors(&mut self, result: Result<(), Vec<String>>) {
        if let Err(errs) = result {
            self.errors.extend(errs);
        }
    }

    pub fn add_warnings(&mut self, warns: Vec<String>) {
        self.warnings.extend(warns);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn format(&self) -> String {
        let mut out = String::new();
        if !self.errors.is_empty() {
            out.push_str("=== CONTENT VALIDATION ERRORS ===\n");
            for e in &self.errors {
                out.push_str("  ERROR: ");
                out.push_str(e);
                out.push('\n');
            }
        }
        out
    }

    pub fn format_warnings(&self) -> String {
        let mut out = String::new();
        if !self.warnings.is_empty() {
            out.push_str("=== CONTENT VALIDATION WARNINGS ===\n");
            for w in &self.warnings {
                out.push_str("  WARN: ");
                out.push_str(w);
                out.push('\n');
            }
        }
        out
    }
}

pub fn validate_all_content(
    surface_db: &SurfaceDB,
    bridge: &MaterialBridge,
    mode: ValidationMode,
) -> Result<(), ContentValidationReport> {
    let mut report = ContentValidationReport::new();

    report.add_errors(crate::world::material_bridge::validate_material_bridge(
        surface_db, bridge,
    ));

    report.add_warnings(crate::world::material_bridge::validate_material_consistency(
        surface_db, bridge,
    ));

    match mode {
        ValidationMode::Strict => {
            if report.has_errors() {
                tracing::error!("{}", report.format());
                return Err(report);
            }
        }
        ValidationMode::ReportAll => {
            if report.has_errors() {
                tracing::error!("{}", report.format());
            }
        }
    }

    if !report.warnings.is_empty() {
        tracing::warn!("{}", report.format_warnings());
    }

    Ok(())
}
