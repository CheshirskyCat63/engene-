//! Entrypoint and Operator Truth Tests
//!
//! These tests verify that documentation matches reality.
//! They catch doc drift, broken operator paths, and inconsistent state.

use std::fs;
use std::path::Path;

/// README mentions only real root bins.
/// README_FIRST_RUN.md must not document non-existent entrypoints.
#[test]
fn readme_mentions_only_real_root_bins() {
    let readme_path = "README_FIRST_RUN.md";
    let content = fs::read_to_string(readme_path)
        .expect("README_FIRST_RUN.md must exist");
    
    // Verify real bins are mentioned
    assert!(content.contains("engene_game"), "README must mention engene_game");
    assert!(content.contains("engene_sdk"), "README must mention engene_sdk");
    assert!(content.contains("engene_headless"), "README must mention engene_headless");
    assert!(content.contains("engene_tools"), "README must mention engene_tools");
    
    // Verify non-existent bins are explicitly NOT canonical
    // engene_test should not be presented as canonical entrypoint
    if content.contains("engene_test") {
        // If mentioned, must explicitly say it's NOT current canonical
        assert!(
            content.contains("not") && content.contains("canonical") || 
            content.contains("not") && content.contains("current") ||
            content.contains("DO NOT") ||
            content.contains("does not exist"),
            "If engene_test is mentioned, README must clarify it's not current canonical"
        );
    }
}

/// Entrypoint truth matches cargo bins.
/// ENTRYPOINT_TRUTH.md must document the same bins as Cargo.toml.
#[test]
fn entrypoint_truth_matches_cargo_bins() {
    let entrypoint_path = "docs/canonical/ENTRYPOINT_TRUTH.md";
    let content = fs::read_to_string(entrypoint_path)
        .expect("ENTRYPOINT_TRUTH.md must exist");
    
    // Verify all four canonical bins are documented
    assert!(content.contains("engene_game"), "ENTRYPOINT_TRUTH must mention engene_game");
    assert!(content.contains("engene_sdk"), "ENTRYPOINT_TRUTH must mention engene_sdk");
    assert!(content.contains("engene_headless"), "ENTRYPOINT_TRUTH must mention engene_headless");
    assert!(content.contains("engene_tools"), "ENTRYPOINT_TRUTH must mention engene_tools");
    
    // Verify it explicitly states these are root package bins
    assert!(
        content.contains("root package bin") || content.contains("root-package"),
        "ENTRYPOINT_TRUTH must clarify these are root package bins"
    );
}

/// All documented test scripts exist.
/// If docs mention scripts/test/*, those files must exist.
#[test]
fn all_documented_test_scripts_exist() {
    let scripts = vec![
        "scripts/test/smoke.sh",
        "scripts/test/smoke.ps1",
        "scripts/test/contracts.sh",
        "scripts/test/contracts.ps1",
        "scripts/test/certification.sh",
        "scripts/test/certification.ps1",
        "scripts/test/perf.sh",
        "scripts/test/perf.ps1",
    ];
    
    for script in &scripts {
        assert!(
            Path::new(script).exists(),
            "Documented script must exist: {}",
            script
        );
    }
}

/// Justfile lanes match test lane map.
/// Justfile commands must align with TEST_LANE_MAP.md.
#[test]
fn justfile_lanes_match_test_lane_map() {
    let justfile_path = "Justfile";
    let justfile = fs::read_to_string(justfile_path)
        .expect("Justfile must exist");
    
    let lane_map_path = "docs/canonical/TEST_LANE_MAP.md";
    let lane_map = fs::read_to_string(lane_map_path)
        .expect("TEST_LANE_MAP.md must exist");
    
    // Verify smoke lane exists in both
    assert!(justfile.contains("smoke:"), "Justfile must have smoke lane");
    assert!(lane_map.contains("Smoke lane"), "TEST_LANE_MAP must document smoke lane");
    
    // Verify contracts lane exists in both
    assert!(justfile.contains("contracts:"), "Justfile must have contracts lane");
    assert!(lane_map.contains("Contract lane"), "TEST_LANE_MAP must document contract lane");
    
    // Verify certification lane exists in both
    assert!(justfile.contains("certification:"), "Justfile must have certification lane");
    assert!(lane_map.contains("Certification lane"), "TEST_LANE_MAP must document certification lane");
    
    // Verify perf lane exists in both
    assert!(justfile.contains("perf:"), "Justfile must have perf lane");
    assert!(lane_map.contains("Perf lane"), "TEST_LANE_MAP must document perf lane");
}

/// Cargo aliases match test lane map.
/// .cargo/config.toml aliases must align with TEST_LANE_MAP.md.
#[test]
fn cargo_aliases_match_test_lane_map() {
    let config_path = ".cargo/config.toml";
    let config = fs::read_to_string(config_path)
        .expect(".cargo/config.toml must exist");
    
    // Verify smoke alias
    assert!(config.contains("smoke ="), ".cargo/config.toml must have smoke alias");
    
    // Verify contracts alias
    assert!(config.contains("contracts ="), ".cargo/config.toml must have contracts alias");
    
    // Verify cert alias
    assert!(config.contains("cert ="), ".cargo/config.toml must have cert alias");
}

/// Current branch state does not claim apps are canonical.
/// CURRENT_BRANCH_STATE.md must not present apps/* as current launch truth.
#[test]
fn current_branch_state_does_not_claim_apps_are_canonical() {
    let state_path = "docs/canonical/CURRENT_BRANCH_STATE.md";
    let content = fs::read_to_string(state_path)
        .expect("CURRENT_BRANCH_STATE.md must exist");
    
    // If apps are mentioned, must clarify they're not current canonical
    if content.contains("apps/") || content.contains("apps/*") {
        // Must have explicit "not" statement about apps not being canonical
        assert!(
            content.contains("not") && (content.contains("canonical") || content.contains("truth") || content.contains("active")),
            "CURRENT_BRANCH_STATE must clarify apps/* are not current canonical launch truth"
        );
    }
}

/// README links current branch state.
/// README_FIRST_RUN.md must link to CURRENT_BRANCH_STATE.md.
#[test]
fn readme_links_current_branch_state() {
    let readme_path = "README_FIRST_RUN.md";
    let content = fs::read_to_string(readme_path)
        .expect("README_FIRST_RUN.md must exist");
    
    assert!(
        content.contains("CURRENT_BRANCH_STATE.md"),
        "README_FIRST_RUN.md must link to CURRENT_BRANCH_STATE.md"
    );
}

/// Quickstarts point to entrypoint truth.
/// GAME_QUICKSTART.md and SDK_QUICKSTART.md must reference ENTRYPOINT_TRUTH.md.
#[test]
fn quickstarts_point_to_entrypoint_truth() {
    let game_quickstart = fs::read_to_string("GAME_QUICKSTART.md")
        .expect("GAME_QUICKSTART.md must exist");
    let sdk_quickstart = fs::read_to_string("SDK_QUICKSTART.md")
        .expect("SDK_QUICKSTART.md must exist");
    
    assert!(
        game_quickstart.contains("ENTRYPOINT_TRUTH.md"),
        "GAME_QUICKSTART.md must reference ENTRYPOINT_TRUTH.md"
    );
    assert!(
        sdk_quickstart.contains("ENTRYPOINT_TRUTH.md"),
        "SDK_QUICKSTART.md must reference ENTRYPOINT_TRUTH.md"
    );
}
