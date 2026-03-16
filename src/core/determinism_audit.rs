use super::system_descriptor::{DeterminismTier, SystemDescriptor};

#[derive(Clone, Debug)]
pub struct DeterminismAuditEntry {
    pub system_name: String,
    pub declared_tier: DeterminismTier,
    pub issues: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DeterminismAuditReport {
    pub entries: Vec<DeterminismAuditEntry>,
    pub hard_count: usize,
    pub soft_count: usize,
    pub non_count: usize,
}

impl DeterminismAuditReport {
    pub fn has_issues(&self) -> bool {
        self.entries.iter().any(|e| !e.issues.is_empty())
    }

    pub fn all_issues(&self) -> Vec<String> {
        self.entries.iter()
            .flat_map(|e| e.issues.iter().map(|i| format!("[{}] {}", e.system_name, i)))
            .collect()
    }
}

pub fn audit_determinism_tiers(descriptors: &[SystemDescriptor]) -> DeterminismAuditReport {
    let mut entries = Vec::new();
    let mut hard_count = 0;
    let mut soft_count = 0;
    let mut non_count = 0;

    for desc in descriptors {
        let mut issues = Vec::new();

        match desc.determinism {
            DeterminismTier::Hard => {
                hard_count += 1;
                if !desc.parallel_safe {
                    issues.push(
                        "Hard-deterministic systems should be parallel-safe for replay consistency".into()
                    );
                }
            }
            DeterminismTier::Soft => {
                soft_count += 1;
            }
            DeterminismTier::NonDeterministic => {
                non_count += 1;
            }
        }

        if desc.determinism == DeterminismTier::Hard && !desc.headless_compatible {
            issues.push("Hard-deterministic systems should be headless-compatible for server replay".into());
        }

        entries.push(DeterminismAuditEntry {
            system_name: desc.name.to_string(),
            declared_tier: desc.determinism,
            issues,
        });
    }

    DeterminismAuditReport {
        entries,
        hard_count,
        soft_count,
        non_count,
    }
}
