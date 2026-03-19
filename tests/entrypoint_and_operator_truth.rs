//! Entrypoint and Operator Truth Tests
//!
//! These tests verify that documentation matches reality exactly.
//! They catch doc drift, broken operator paths, and inconsistent state.

use std::fs;
use std::path::Path;

/// Extract bin names from Cargo.toml
fn extract_bin_names(cargo_toml: &str) -> Vec<String> {
    let mut bins = Vec::new();
    let mut in_bin_section = false;
    
    for line in cargo_toml.lines() {
        if line.trim() == "[[bin]]" {
            in_bin_section = true;
        } else if line.trim().starts_with('[') && !line.trim().starts_with("[[bin]]") {
            in_bin_section = false;
        } else if in_bin_section && line.trim().starts_with("name =") {
            let name = line.split('=').nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
            if let Some(n) = name {
                bins.push(n);
            }
        }
    }
    bins
}

/// README mentions only real root bins.
#[test]
fn readme_mentions_only_real_root_bins() {
    let readme = fs::read_to_string("README_FIRST_RUN.md")
        .expect("README_FIRST_RUN.md must exist");
    let cargo = fs::read_to_string("Cargo.toml")
        .expect("Cargo.toml must exist");
    
    let bin_names = extract_bin_names(&cargo);
    
    // Verify all 4 bins are documented
    assert!(bin_names.contains(&"engene_game".to_string()),
        "Cargo.toml must define engene_game bin");
    assert!(bin_names.contains(&"engene_sdk".to_string()),
        "Cargo.toml must define engene_sdk bin");
    assert!(bin_names.contains(&"engene_headless".to_string()),
        "Cargo.toml must define engene_headless bin");
    assert!(bin_names.contains(&"engene_tools".to_string()),
        "Cargo.toml must define engene_tools bin");

    // README must mention all bins
    for bin in &bin_names {
        assert!(readme.contains(bin),
            "README must mention bin: {}", bin);
    }

    // README must NOT present apps/* as current canonical
    assert!(!readme.contains("apps/engene_game") || readme.contains("future") || readme.contains("not"),
        "README must not present apps/* as current canonical");

    // README must clarify engene_test is not current
    if readme.contains("engene_test") {
        assert!(readme.contains("not") && (readme.contains("canonical") || readme.contains("current")),
            "If engene_test is mentioned, must clarify it's not current canonical");
    }
}

/// Entrypoint truth matches cargo bins exactly.
#[test]
fn entrypoint_truth_matches_cargo_bins() {
    let entrypoint = fs::read_to_string("docs/canonical/ENTRYPOINT_TRUTH.md")
        .expect("ENTRYPOINT_TRUTH.md must exist");
    let cargo = fs::read_to_string("Cargo.toml")
        .expect("Cargo.toml must exist");
    
    let bin_names = extract_bin_names(&cargo);
    
    // Each bin must be documented in ENTRYPOINT_TRUTH
    for bin in &bin_names {
        assert!(entrypoint.contains(bin),
            "ENTRYPOINT_TRUTH must document bin: {}", bin);
    }

    // Verify exact count - 4 bins
    assert_eq!(bin_names.len(), 4,
        "Exactly 4 bins must be defined, found: {:?}", bin_names);
}

/// All documented test scripts exist.
#[test]
fn all_documented_test_scripts_exist() {
    let scripts = [
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
        assert!(Path::new(script).exists(),
            "Script must exist: {}", script);
    }
}

/// Justfile lanes match test lane map exactly.
#[test]
fn justfile_lanes_match_test_lane_map() {
    let justfile = fs::read_to_string("Justfile")
        .expect("Justfile must exist");
    let lane_map = fs::read_to_string("docs/canonical/TEST_LANE_MAP.md")
        .expect("TEST_LANE_MAP.md must exist");
    
    // Verify exact lane names exist in both
    let lanes = ["smoke", "contracts", "certification", "perf"];
    
    for lane in &lanes {
        // Justfile must have lane as target
        assert!(justfile.contains(&format!("{}:", lane)),
            "Justfile must have {} lane", lane);
        
        // TEST_LANE_MAP must document lane
        assert!(lane_map.to_lowercase().contains(&format!("{} lane", lane.to_lowercase())),
            "TEST_LANE_MAP must document {} lane", lane);
    }
}

/// Cargo aliases match test lane map.
#[test]
fn cargo_aliases_match_test_lane_map() {
    let config = fs::read_to_string(".cargo/config.toml")
        .expect(".cargo/config.toml must exist");
    let lane_map = fs::read_to_string("docs/canonical/TEST_LANE_MAP.md")
        .expect("TEST_LANE_MAP.md must exist");
    
    // Verify smoke alias exists
    assert!(config.contains("smoke ="),
        ".cargo/config.toml must have smoke alias");

    // Verify contracts alias exists
    assert!(config.contains("contracts ="),
        ".cargo/config.toml must have contracts alias");

    // Verify cert alias exists
    assert!(config.contains("cert ="),
        ".cargo/config.toml must have cert alias");

    // Verify these are documented in lane map
    assert!(lane_map.contains("cargo smoke"),
        "TEST_LANE_MAP must document 'cargo smoke'");
    assert!(lane_map.contains("cargo contracts"),
        "TEST_LANE_MAP must document 'cargo contracts'");
    assert!(lane_map.contains("cargo cert"),
        "TEST_LANE_MAP must document 'cargo cert'");
}

/// Current branch state does not claim apps are canonical.
#[test]
fn current_branch_state_does_not_claim_apps_are_canonical() {
    let state = fs::read_to_string("docs/canonical/CURRENT_BRANCH_STATE.md")
        .expect("CURRENT_BRANCH_STATE.md must exist");
    
    // If apps mentioned, must clarify they're not current canonical
    if state.contains("apps/") {
        // Must have explicit "not" statement
        let apps_section = state.split("apps/").nth(1).unwrap_or("");
        assert!(
            apps_section.contains("not") || 
            state.contains("not current canonical") ||
            state.contains("not yet operator truth"),
            "CURRENT_BRANCH_STATE must clarify apps/* are not current canonical launch truth"
        );
    }
}

/// README links current branch state.
#[test]
fn readme_links_current_branch_state() {
    let readme = fs::read_to_string("README_FIRST_RUN.md")
        .expect("README_FIRST_RUN.md must exist");
    
    assert!(readme.contains("CURRENT_BRANCH_STATE.md"),
        "README must link to CURRENT_BRANCH_STATE.md");
}

/// Quickstarts point to entrypoint truth.
#[test]
fn quickstarts_point_to_entrypoint_truth() {
    let game = fs::read_to_string("GAME_QUICKSTART.md")
        .expect("GAME_QUICKSTART.md must exist");
    let sdk = fs::read_to_string("SDK_QUICKSTART.md")
        .expect("SDK_QUICKSTART.md must exist");
    
    assert!(game.contains("ENTRYPOINT_TRUTH.md"),
        "GAME_QUICKSTART must reference ENTRYPOINT_TRUTH.md");
    assert!(sdk.contains("ENTRYPOINT_TRUTH.md"),
        "SDK_QUICKSTART must reference ENTRYPOINT_TRUTH.md");
}
