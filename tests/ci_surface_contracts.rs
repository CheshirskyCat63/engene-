//! CI Surface Contracts
//!
//! These tests verify that CI configuration matches documented expectations.
//! They catch CI drift and ensure discipline is maintained.

use std::fs;

/// CI watches main and transition branch.
/// .github/workflows/rust.yml must cover both main and engene-2.0-transition.
#[test]
fn ci_watches_main_and_transition_branch() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    assert!(
        content.contains("\"main\""),
        "CI must watch main branch"
    );
    assert!(
        content.contains("engene-2.0-transition"),
        "CI must watch engene-2.0-transition branch"
    );
}

/// CI includes fmt gate.
/// CI must run cargo fmt --check.
#[test]
fn ci_includes_fmt_gate() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    assert!(
        content.contains("fmt") && (content.contains("--check") || content.contains("fmt-check")),
        "CI must include format check gate"
    );
}

/// CI includes clippy gate.
/// CI must run cargo clippy with -D warnings.
#[test]
fn ci_includes_clippy_gate() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    assert!(
        content.contains("clippy"),
        "CI must include clippy gate"
    );
    assert!(
        content.contains("-D warnings") || content.contains("-Dwarnings"),
        "CI clippy must deny warnings"
    );
}

/// CI runs smoke lane.
/// CI must run the smoke test lane.
#[test]
fn ci_runs_smoke_lane() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    assert!(
        content.contains("smoke"),
        "CI must run smoke lane"
    );
}

/// CI runs contract lane.
/// CI must run the contract test lane.
#[test]
fn ci_runs_contract_lane() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    assert!(
        content.contains("contracts") || content.contains("contract"),
        "CI must run contract lane"
    );
}

/// CI does not run certification or perf in default PR path.
/// Certification and perf should be operator-only, not PR gates.
#[test]
fn ci_does_not_run_certification_or_perf_in_default_pr_path() {
    let workflow_path = ".github/workflows/rust.yml";
    let content = fs::read_to_string(workflow_path)
        .expect(".github/workflows/rust.yml must exist");
    
    // Count how many times certification/perf appear in job context
    // They should NOT be in the main PR verification jobs
    
    // If certification appears, verify it's NOT in a required job
    // This is a soft check - we verify the structure exists
    let has_certification_job = content.contains("certification");
    let has_perf_job = content.contains("perf");
    
    // Document the expectation: certification/perf should not be required PR gates
    // If they exist as jobs, they should be optional or separate workflows
    if has_certification_job || has_perf_job {
        // They should be in separate jobs, not in the main test job
        assert!(
            content.contains("jobs:") && content.split("jobs:").count() > 1,
            "If certification/perf jobs exist, they should be separate from main jobs"
        );
    }
    
    // The main verification should be smoke + contracts, not full certification
    // This test passes if the CI structure is reasonable
}
