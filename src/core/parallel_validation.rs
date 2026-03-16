use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};

#[derive(Debug)]
pub enum ValidationIssue {
    WriteWriteConflict { system_a: &'static str, system_b: &'static str, description: String },
    WriteReadHazard { writer: &'static str, reader: &'static str, description: String },
    HeadlessUnsafe { system: &'static str },
    DeterminismViolation { system: &'static str, description: String },
}

pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| matches!(i,
            ValidationIssue::WriteWriteConflict { .. } |
            ValidationIssue::DeterminismViolation { .. }
        ))
    }

    pub fn has_warnings(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn print_report(&self) {
        for issue in &self.issues {
            match issue {
                ValidationIssue::WriteWriteConflict { system_a, system_b, description } => {
                    println!("[VALIDATION ERROR] Write-write conflict: '{}' <-> '{}': {}", system_a, system_b, description);
                }
                ValidationIssue::WriteReadHazard { writer, reader, description } => {
                    println!("[VALIDATION WARN] Write-read hazard: '{}' writes, '{}' reads: {}", writer, reader, description);
                }
                ValidationIssue::HeadlessUnsafe { system } => {
                    println!("[VALIDATION WARN] System '{}' is not headless-compatible", system);
                }
                ValidationIssue::DeterminismViolation { system, description } => {
                    println!("[VALIDATION ERROR] Determinism violation in '{}': {}", system, description);
                }
            }
        }
    }
}

pub fn validate_systems(descriptors: &[SystemDescriptor], require_headless: bool, require_deterministic: bool) -> ValidationReport {
    let mut issues = Vec::new();

    for (i, a) in descriptors.iter().enumerate() {
        if require_headless && !a.headless_compatible {
            issues.push(ValidationIssue::HeadlessUnsafe { system: a.name });
        }

        if require_deterministic && a.determinism == DeterminismTier::NonDeterministic {
            issues.push(ValidationIssue::DeterminismViolation {
                system: a.name,
                description: "non-deterministic system in deterministic mode".to_string(),
            });
        }

        for b in descriptors.iter().skip(i + 1) {
            if !a.parallel_safe || !b.parallel_safe { continue; }

            for w in &a.writes_components {
                if b.writes_components.contains(w) {
                    issues.push(ValidationIssue::WriteWriteConflict {
                        system_a: a.name,
                        system_b: b.name,
                        description: format!("both write component {:?}", w),
                    });
                }
                if b.reads_components.contains(w) {
                    issues.push(ValidationIssue::WriteReadHazard {
                        writer: a.name,
                        reader: b.name,
                        description: format!("component {:?}", w),
                    });
                }
            }

            for w in &b.writes_components {
                if a.reads_components.contains(w) {
                    issues.push(ValidationIssue::WriteReadHazard {
                        writer: b.name,
                        reader: a.name,
                        description: format!("component {:?}", w),
                    });
                }
            }

            for w in &a.writes_resources {
                if b.writes_resources.contains(w) {
                    issues.push(ValidationIssue::WriteWriteConflict {
                        system_a: a.name,
                        system_b: b.name,
                        description: format!("both write resource {:?}", w),
                    });
                }
            }
        }
    }

    ValidationReport { issues }
}
