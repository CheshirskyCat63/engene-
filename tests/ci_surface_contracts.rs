//! CI Surface Contracts
//!
//! These tests verify CI configuration matches documented expectations exactly.
//! They lock the workflow structure and catch CI drift.

use std::fs;

const WORKFLOW_PATH: &str = ".github/workflows/rust.yml";

/// Parse workflow to extract branch list from on.push.branches
fn extract_push_branches(workflow: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut in_push_section = false;
    let mut in_branches_section = false;
    
    for line in workflow.lines() {
        let trimmed = line.trim();
        
        if trimmed == "push:" {
            in_push_section = true;
        } else if trimmed.starts_with("branches:") && in_push_section {
            in_branches_section = true;
            // Check if it's a single line array
            if trimmed.contains('[') {
                let start = trimmed.find('[').unwrap();
                let content = &trimmed[start..].trim_start_matches('[').trim_end_matches(']');
                for part in content.split(',') {
                    let branch = part.trim().trim_matches('"').to_string();
                    if !branch.is_empty() {
                        branches.push(branch);
                    }
                }
                break;
            }
        } else if trimmed.starts_with('"') && in_branches_section {
            // Multi-line array
            let branch = trimmed.trim_matches(',').trim_matches('"').to_string();
            if !branch.is_empty() {
                branches.push(branch);
            }
        } else if trimmed.starts_with('-') && in_branches_section {
            // YAML list format
            let branch = trimmed.trim_start_matches('-').trim().trim_matches('"').to_string();
            if !branch.is_empty() {
                branches.push(branch);
            }
        } else if in_push_section && !trimmed.starts_with('"') && !trimmed.starts_with('-') && !trimmed.is_empty() && !trimmed.starts_with("branches") {
            if !trimmed.starts_with('#') {
                break;
            }
        }
    }
    
    branches
}

/// Parse workflow to extract job names
fn extract_job_names(workflow: &str) -> Vec<String> {
    let mut jobs = Vec::new();
    let mut in_jobs_section = false;
    
    for line in workflow.lines() {
        let trimmed = line.trim();
        
        if trimmed == "jobs:" {
            in_jobs_section = true;
        } else if in_jobs_section && trimmed.ends_with(':') && !trimmed.starts_with('-') {
            let job_name = trimmed.trim_end_matches(':').to_string();
            if !job_name.is_empty() && job_name != "jobs" {
                jobs.push(job_name);
            }
        }
    }
    
    jobs
}

/// CI watches main and transition branch exactly.
#[test]
fn ci_watches_main_and_transition_branch() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let branches = extract_push_branches(&workflow);
    
    assert!(branches.contains(&"main".to_string()),
        "CI must watch 'main' branch, found: {:?}", branches);
    assert!(branches.contains(&"engene-2.0-transition".to_string()),
        "CI must watch 'engene-2.0-transition' branch, found: {:?}", branches);
}

/// CI includes fmt gate.
#[test]
fn ci_includes_fmt_gate() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let jobs = extract_job_names(&workflow);
    
    assert!(jobs.contains(&"fmt".to_string()),
        "CI must have 'fmt' job, found: {:?}", jobs);

    // Verify fmt job runs cargo fmt --check
    let fmt_section = workflow.split("fmt:").nth(1).unwrap_or("");
    assert!(fmt_section.contains("cargo fmt") && fmt_section.contains("--check"),
        "fmt job must run 'cargo fmt --check'");
}

/// CI includes clippy gate with deny warnings.
#[test]
fn ci_includes_clippy_gate() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let jobs = extract_job_names(&workflow);
    
    assert!(jobs.contains(&"clippy".to_string()),
        "CI must have 'clippy' job, found: {:?}", jobs);

    // Verify clippy job runs with -D warnings
    let clippy_section = workflow.split("clippy:").nth(1).unwrap_or("");
    assert!(clippy_section.contains("clippy"),
        "clippy job must run clippy");
    assert!(clippy_section.contains("-D warnings") || clippy_section.contains("-Dwarnings"),
        "clippy job must deny warnings with -D warnings");
}

/// CI runs smoke lane.
#[test]
fn ci_runs_smoke_lane() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let jobs = extract_job_names(&workflow);
    
    assert!(jobs.contains(&"smoke".to_string()),
        "CI must have 'smoke' job, found: {:?}", jobs);

    // Verify smoke job uses nextest
    let smoke_section = workflow.split("smoke:").nth(1).unwrap_or("");
    assert!(smoke_section.contains("nextest"),
        "smoke job must use nextest");
    assert!(smoke_section.contains("engine_contracts") && smoke_section.contains("production_candidate"),
        "smoke job must run engine_contracts and production_candidate tests");
}

/// CI runs contract lane.
#[test]
fn ci_runs_contract_lane() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let jobs = extract_job_names(&workflow);
    
    assert!(jobs.contains(&"contracts".to_string()),
        "CI must have 'contracts' job, found: {:?}", jobs);

    // Verify contracts job uses nextest
    let contracts_section = workflow.split("contracts:").nth(1).unwrap_or("");
    assert!(contracts_section.contains("nextest"),
        "contracts job must use nextest");
}

/// CI does not run certification or perf in default PR path.
#[test]
fn ci_does_not_run_certification_or_perf_in_default_pr_path() {
    let workflow = fs::read_to_string(WORKFLOW_PATH)
        .expect(".github/workflows/rust.yml must exist");
    
    let jobs = extract_job_names(&workflow);
    
    // Certification and perf should NOT be in the main job list
    // They can exist as separate workflows or optional jobs
    let has_certification = jobs.contains(&"certification".to_string());
    let has_perf = jobs.contains(&"perf".to_string());
    
    // If they exist as jobs, they should be optional (not required for PR)
    // Check they're not in the needs chain of build
    if has_certification || has_perf {
        let build_section = workflow.split("build:").nth(1).unwrap_or("");
        // Build should not wait for certification/perf
        assert!(!build_section.contains("certification") && !build_section.contains("perf"),
            "build job should not depend on certification/perf");
    }
    
    // Verify the main required jobs are fmt, clippy, build, smoke, contracts
    let required_jobs = ["fmt", "clippy", "build", "smoke", "contracts"];
    for job in &required_jobs {
        assert!(jobs.contains(&job.to_string()),
            "CI must have '{}' job as required gate", job);
    }
}
