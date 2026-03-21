//! Entrypoint and Operator Truth Tests
//!
//! These tests verify that the `apps/*` entrypoint definitions and docs match reality.

use std::collections::HashSet;
use std::fs;

fn package_name_from_toml(path: &str) -> String {
    let cargo = fs::read_to_string(path).unwrap_or_else(|_| panic!("Cannot read {}", path));
    let mut in_package = false;
    for line in cargo.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            continue;
        }
        if in_package {
            if trimmed.starts_with('[') {
                break;
            }
            if let Some(name_value) = trimmed.strip_prefix("name =") {
                return name_value.trim().trim_matches('"').to_string();
            }
        }
    }
    panic!("Could not parse [package] name from {}", path);
}

#[test]
fn app_package_names_are_expected() {
    let apps = vec![
        "apps/engene_game/Cargo.toml",
        "apps/engene_sdk/Cargo.toml",
        "apps/engene_headless/Cargo.toml",
        "apps/engene_bootstrap/Cargo.toml",
    ];

    let package_names: HashSet<_> = apps.iter().map(|p| package_name_from_toml(p)).collect();

    let expected: HashSet<_> = [
        "app_engene_game".to_string(),
        "app_engene_sdk".to_string(),
        "app_engene_headless".to_string(),
        "engene_bootstrap".to_string(),
    ]
    .into_iter()
    .collect();

    assert_eq!(
        package_names, expected,
        "apps package names must exactly match expected set"
    );
}

#[test]
fn no_engene_tools_package_or_docs_references() {
    let tools_path = "apps/engene_tools/Cargo.toml";
    assert!(
        !std::path::Path::new(tools_path).exists(),
        "apps/engene_tools must not exist in current tree"
    );

    let docs = vec![
        "README_FIRST_RUN.md",
        "docs/canonical/ENTRYPOINT_TRUTH.md",
        "docs/canonical/CURRENT_BRANCH_STATE.md",
    ];

    for doc in docs {
        let content = fs::read_to_string(doc).unwrap_or_else(|_| panic!("Cannot open {}", doc));
        assert!(
            !content.contains("engene_tools"),
            "{} must not mention engene_tools",
            doc
        );
        assert!(
            !content.contains("apps/engene_tools"),
            "{} must not mention apps/engene_tools",
            doc
        );
    }
}

#[test]
fn docs_reference_real_app_package_names() {
    let expected_packages = [
        "app_engene_game",
        "app_engene_sdk",
        "app_engene_headless",
        "engene_bootstrap",
    ];

    let docs = vec![
        "README_FIRST_RUN.md",
        "docs/canonical/ENTRYPOINT_TRUTH.md",
        "docs/canonical/CURRENT_BRANCH_STATE.md",
        "GAME_QUICKSTART.md",
        "SDK_QUICKSTART.md",
    ];

    for doc in docs {
        let content = fs::read_to_string(doc).unwrap_or_else(|_| panic!("Cannot open {}", doc));

        for pkg in &expected_packages {
            assert!(content.contains(pkg), "{} must mention {}", doc, pkg);
        }

        if doc == "GAME_QUICKSTART.md" {
            assert!(
                content.contains("cargo run -p app_engene_game"),
                "{} must use cargo run -p app_engene_game",
                doc
            );
        }
        if doc == "SDK_QUICKSTART.md" {
            assert!(
                content.contains("cargo run -p app_engene_sdk"),
                "{} must use cargo run -p app_engene_sdk",
                doc
            );
        }
    }
}

/// Entrypoint truth matches cargo bins exactly.
#[test]
fn entrypoint_truth_matches_cargo_bins() {
    let entrypoint = fs::read_to_string("docs/canonical/ENTRYPOINT_TRUTH.md")
        .expect("ENTRYPOINT_TRUTH.md must exist");
    let cargo = fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");

    let bin_names = extract_bin_names(&cargo);

    // Each bin must be documented in ENTRYPOINT_TRUTH
    for bin in &bin_names {
        assert!(
            entrypoint.contains(bin),
            "ENTRYPOINT_TRUTH must document bin: {}",
            bin
        );
    }

    // Verify exact count - 4 bins
    assert_eq!(
        bin_names.len(),
        4,
        "Exactly 4 bins must be defined, found: {:?}",
        bin_names
    );
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
        assert!(Path::new(script).exists(), "Script must exist: {}", script);
    }
}

/// Justfile lanes match test lane map exactly.
#[test]
fn justfile_lanes_match_test_lane_map() {
    let justfile = fs::read_to_string("Justfile").expect("Justfile must exist");
    let lane_map =
        fs::read_to_string("docs/canonical/TEST_LANE_MAP.md").expect("TEST_LANE_MAP.md must exist");

    // Verify exact lane names exist in both
    let lanes = ["smoke", "contracts", "certification", "perf"];

    for lane in &lanes {
        // Justfile must have lane as target
        assert!(
            justfile.contains(&format!("{}:", lane)),
            "Justfile must have {} lane",
            lane
        );

        // TEST_LANE_MAP must document lane
        assert!(
            lane_map
                .to_lowercase()
                .contains(&format!("{} lane", lane.to_lowercase())),
            "TEST_LANE_MAP must document {} lane",
            lane
        );
    }
}

/// Cargo aliases match test lane map.
#[test]
fn cargo_aliases_match_test_lane_map() {
    let config = fs::read_to_string(".cargo/config.toml").expect(".cargo/config.toml must exist");
    let lane_map =
        fs::read_to_string("docs/canonical/TEST_LANE_MAP.md").expect("TEST_LANE_MAP.md must exist");

    // Verify smoke alias exists
    assert!(
        config.contains("smoke ="),
        ".cargo/config.toml must have smoke alias"
    );

    // Verify contracts alias exists
    assert!(
        config.contains("contracts ="),
        ".cargo/config.toml must have contracts alias"
    );

    // Verify cert alias exists
    assert!(
        config.contains("cert ="),
        ".cargo/config.toml must have cert alias"
    );

    // Verify these are documented in lane map
    assert!(
        lane_map.contains("cargo smoke"),
        "TEST_LANE_MAP must document 'cargo smoke'"
    );
    assert!(
        lane_map.contains("cargo contracts"),
        "TEST_LANE_MAP must document 'cargo contracts'"
    );
    assert!(
        lane_map.contains("cargo cert"),
        "TEST_LANE_MAP must document 'cargo cert'"
    );
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
            apps_section.contains("not")
                || state.contains("not current canonical")
                || state.contains("not yet operator truth"),
            "CURRENT_BRANCH_STATE must clarify apps/* are not current canonical launch truth"
        );
    }
}

/// README links current branch state.
#[test]
fn readme_links_current_branch_state() {
    let readme = fs::read_to_string("README_FIRST_RUN.md").expect("README_FIRST_RUN.md must exist");

    assert!(
        readme.contains("CURRENT_BRANCH_STATE.md"),
        "README must link to CURRENT_BRANCH_STATE.md"
    );
}

/// Quickstarts point to entrypoint truth.
#[test]
fn quickstarts_point_to_entrypoint_truth() {
    let game = fs::read_to_string("GAME_QUICKSTART.md").expect("GAME_QUICKSTART.md must exist");
    let sdk = fs::read_to_string("SDK_QUICKSTART.md").expect("SDK_QUICKSTART.md must exist");

    assert!(
        game.contains("ENTRYPOINT_TRUTH.md"),
        "GAME_QUICKSTART must reference ENTRYPOINT_TRUTH.md"
    );
    assert!(
        sdk.contains("ENTRYPOINT_TRUTH.md"),
        "SDK_QUICKSTART must reference ENTRYPOINT_TRUTH.md"
    );
}
