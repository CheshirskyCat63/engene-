/// Classification of all bugs/failures in the engine.
/// Every issue reported by golden scenarios, long-run tests, or CI
/// should be tagged with one of these categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    /// Wrong simulation result (AI made impossible decision, physics violated)
    Correctness,
    /// Entity state lost or corrupted across L0/L2 transition or chunk boundary
    Continuity,
    /// Save/load loses or corrupts data
    Persistence,
    /// Two systems disagree on world truth (destruction says broken, nav says intact)
    AuthorityMismatch,
    /// Performance budget exceeded
    PerformanceRegression,
    /// Simulation dynamics diverge to extremes (economy singularity, population collapse)
    BalanceRunaway,
    /// Flickering, ghosting, temporal artifacts, visual corruption
    VisualInstability,
    /// Editor/validator produces wrong result or crashes
    ToolingFailure,
}

impl std::fmt::Display for FailureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Correctness => write!(f, "CORRECTNESS"),
            Self::Continuity => write!(f, "CONTINUITY"),
            Self::Persistence => write!(f, "PERSISTENCE"),
            Self::AuthorityMismatch => write!(f, "AUTHORITY_MISMATCH"),
            Self::PerformanceRegression => write!(f, "PERF_REGRESSION"),
            Self::BalanceRunaway => write!(f, "BALANCE_RUNAWAY"),
            Self::VisualInstability => write!(f, "VISUAL_INSTABILITY"),
            Self::ToolingFailure => write!(f, "TOOLING_FAILURE"),
        }
    }
}

/// A tagged failure report
#[derive(Clone, Debug)]
pub struct FailureReport {
    pub category: FailureCategory,
    pub phase: String,
    pub description: String,
    pub severity: FailureSeverity,
    pub reproducible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureSeverity {
    /// Must fix before phase completion
    Critical,
    /// Should fix, may block release
    Major,
    /// Fix when convenient
    Minor,
}

/// Stop signals -- conditions that require halting the roadmap and reassessing
pub fn check_stop_signals(reports: &[FailureReport]) -> Vec<String> {
    let mut signals = Vec::new();

    let critical_persistence = reports
        .iter()
        .filter(|r| {
            r.category == FailureCategory::Persistence
                && r.severity == FailureSeverity::Critical
        })
        .count();
    if critical_persistence > 0 {
        signals.push(format!(
            "{} critical persistence failures -- identity/save system compromised",
            critical_persistence
        ));
    }

    let authority_mismatches = reports
        .iter()
        .filter(|r| r.category == FailureCategory::AuthorityMismatch)
        .count();
    if authority_mismatches > 3 {
        signals.push(format!(
            "{} authority mismatches -- world state contract violated",
            authority_mismatches
        ));
    }

    let continuity_bugs = reports
        .iter()
        .filter(|r| {
            r.category == FailureCategory::Continuity
                && r.severity == FailureSeverity::Critical
        })
        .count();
    if continuity_bugs > 0 {
        signals.push(format!(
            "{} critical continuity bugs -- L0/L2 transition or chunk reload corrupts state",
            continuity_bugs
        ));
    }

    let runaway = reports
        .iter()
        .filter(|r| r.category == FailureCategory::BalanceRunaway)
        .count();
    if runaway > 2 {
        signals.push(format!(
            "{} balance runaway events -- systemic instability",
            runaway
        ));
    }

    signals
}

/// Staged performance thresholds
pub struct StagedThreshold {
    pub metric_name: &'static str,
    pub interim: f64,
    pub target: f64,
    pub release_gate: f64,
    pub unit: &'static str,
}

pub fn performance_thresholds() -> Vec<StagedThreshold> {
    vec![
        StagedThreshold {
            metric_name: "Frame budget (High, 60fps)",
            interim: 20.0,
            target: 16.0,
            release_gate: 16.7,
            unit: "ms",
        },
        StagedThreshold {
            metric_name: "Streaming hitch",
            interim: 5.0,
            target: 2.0,
            release_gate: 3.0,
            unit: "ms/frame",
        },
        StagedThreshold {
            metric_name: "Memory (4-region, High)",
            interim: 3072.0,
            target: 2048.0,
            release_gate: 2560.0,
            unit: "MB",
        },
        StagedThreshold {
            metric_name: "Chunk save/load",
            interim: 100.0,
            target: 50.0,
            release_gate: 70.0,
            unit: "ms",
        },
        StagedThreshold {
            metric_name: "AI think time (L0)",
            interim: 2.0,
            target: 1.0,
            release_gate: 1.5,
            unit: "ms/entity",
        },
        StagedThreshold {
            metric_name: "Chunk activation",
            interim: 30.0,
            target: 15.0,
            release_gate: 20.0,
            unit: "ms",
        },
        StagedThreshold {
            metric_name: "Background sim step (1000 L2)",
            interim: 1.0,
            target: 0.5,
            release_gate: 0.7,
            unit: "ms",
        },
    ]
}
