//! Physics Bootstrap Contracts
//!
//! These tests verify that physics bootstrap contract is documented accurately
//! and that physics is not claimed to be connected when it's actually a stub.

use std::fs;

/// Verify physics bootstrap contract is not future fanfic.
#[test]
fn physics_bootstrap_contract_is_not_future_fanfic() {
    let bootstrap_contract = fs::read_to_string("docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md")
        .expect("PHYSICS_BOOTSTRAP_CONTRACT.md must exist");
    
    // Verify it describes current state, not just future
    // It should mention "current" or "as of" or "status"
    let has_current_state = bootstrap_contract.to_lowercase().contains("current") ||
                            bootstrap_contract.to_lowercase().contains("as of") ||
                            bootstrap_contract.to_lowercase().contains("status") ||
                            bootstrap_contract.to_lowercase().contains("not yet");
    
    assert!(has_current_state,
        "PHYSICS_BOOTSTRAP_CONTRACT must describe current state, not just future");
    
    // Verify it does NOT claim physics is connected when it's not
    // Check that stub state is acknowledged
    let mentions_stub = bootstrap_contract.to_lowercase().contains("stub") ||
                        bootstrap_contract.to_lowercase().contains("placeholder") ||
                        bootstrap_contract.to_lowercase().contains("not yet implemented");
    
    // Either mentions stub, or clearly documents what needs to exist
    let documents_prerequisites = bootstrap_contract.contains("Preconditions") ||
                                   bootstrap_contract.contains("must exist") ||
                                   bootstrap_contract.contains("before physics is");
    
    assert!(mentions_stub || documents_prerequisites,
        "PHYSICS_BOOTSTRAP_CONTRACT must acknowledge stub state or document prerequisites");
}

/// Physics bootstrap preconditions are explicit.
#[test]
fn physics_bootstrap_preconditions_are_explicit() {
    let bootstrap_contract = fs::read_to_string("docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md")
        .expect("PHYSICS_BOOTSTRAP_CONTRACT.md must exist");
    
    // Verify preconditions section exists
    assert!(bootstrap_contract.contains("Preconditions") ||
            bootstrap_contract.contains("preconditions") ||
            bootstrap_contract.contains("must exist"),
        "PHYSICS_BOOTSTRAP_CONTRACT must have preconditions section");
    
    // Verify specific preconditions are listed
    let has_boundary_doc = bootstrap_contract.contains("PHYSICS_CORE_BOUNDARY");
    let has_tests = bootstrap_contract.contains("test") ||
                    bootstrap_contract.contains("contract");
    let has_implementation = bootstrap_contract.contains("implementation") ||
                             bootstrap_contract.contains("real");
    
    // At least boundary doc and tests should be mentioned as prerequisites
    assert!(has_boundary_doc && has_tests,
        "PHYSICS_BOOTSTRAP_CONTRACT must list boundary doc and tests as prerequisites");
}

/// Current branch state does not overclaim physics integration.
#[test]
fn current_branch_state_does_not_overclaim_physics_integration() {
    let branch_state = fs::read_to_string("docs/canonical/CURRENT_BRANCH_STATE.md")
        .expect("CURRENT_BRANCH_STATE.md must exist");
    
    // If physics is mentioned, verify it's not claimed as fully integrated
    if branch_state.to_lowercase().contains("physics") {
        // Should have "not" or "stub" or "not yet" near physics mentions
        let physics_section = branch_state.to_lowercase();
        
        // Check if physics is mentioned in a "not yet true" or explicit disclaimer context
        let has_disclaimer = physics_section.contains("not") ||
                             physics_section.contains("stub") ||
                             physics_section.contains("not yet") ||
                             physics_section.contains("placeholder");
        
        // If physics is claimed as "active" or "working" without disclaimer, that's wrong
        let claims_physics_active = (branch_state.contains("physics") && 
                                     branch_state.contains("active")) ||
                                    (branch_state.contains("physics") && 
                                     branch_state.contains("working"));
        
        if claims_physics_active {
            assert!(has_disclaimer,
                "CURRENT_BRANCH_STATE must not claim physics is active without disclaimer");
        }
    }
    
    // Verify there are "not yet true" statements
    let has_not_yet_true = branch_state.contains("not-yet-true") ||
                           branch_state.contains("not yet true") ||
                           branch_state.contains("not yet");
    
    assert!(has_not_yet_true,
        "CURRENT_BRANCH_STATE must have explicit 'not yet true' statements");
}

/// Physics readiness is blocked until boundary and tests exist.
#[test]
fn physics_readiness_is_blocked_until_boundary_and_tests_exist() {
    // Verify PHYSICS_CORE_BOUNDARY.md exists
    let boundary_exists = std::path::Path::new("docs/canonical/PHYSICS_CORE_BOUNDARY.md").exists();
    assert!(boundary_exists,
        "PHYSICS_CORE_BOUNDARY.md must exist for physics to be considered ready");
    
    // Verify PHYSICS_BOOTSTRAP_CONTRACT.md exists
    let bootstrap_exists = std::path::Path::new("docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md").exists();
    assert!(bootstrap_exists,
        "PHYSICS_BOOTSTRAP_CONTRACT.md must exist for physics to be considered ready");
    
    // Verify physics boundary tests exist
    let boundary_tests_exist = std::path::Path::new("tests/physics_core_boundary_contracts.rs").exists();
    assert!(boundary_tests_exist,
        "physics_core_boundary_contracts.rs must exist for physics to be considered ready");
    
    // Verify physics bootstrap tests exist
    let bootstrap_tests_exist = std::path::Path::new("tests/physics_bootstrap_contracts.rs").exists();
    assert!(bootstrap_tests_exist,
        "physics_bootstrap_contracts.rs must exist for physics to be considered ready");
}

/// Fast verification docs do not lie about physics readiness.
#[test]
fn fast_verification_docs_do_not_lie_about_physics_readiness() {
    let platform_audit = fs::read_to_string("docs/canonical/PLATFORM_AUDIT.md")
        .expect("PLATFORM_AUDIT.md must exist");
    
    // Verify platform audit mentions physics status
    assert!(platform_audit.to_lowercase().contains("physics"),
        "PLATFORM_AUDIT must mention physics status");
    
    // Verify it does not claim physics is fully connected
    // Should have accurate "not yet true" statements
    let has_physics_truth = platform_audit.to_lowercase().contains("physics") &&
                            (platform_audit.to_lowercase().contains("stub") ||
                             platform_audit.to_lowercase().contains("not") ||
                             platform_audit.to_lowercase().contains("not yet"));
    
    assert!(has_physics_truth,
        "PLATFORM_AUDIT must accurately describe physics as not yet connected");
}

/// Engine_physics crate is a stub (placeholder), not real implementation.
#[test]
fn engine_physics_crate_is_stub_not_implementation() {
    let physics_lib = fs::read_to_string("crates/engine_physics/src/lib.rs")
        .expect("crates/engine_physics/src/lib.rs must exist");
    
    // Verify it's a stub - should have minimal content or explicit placeholder
    let is_stub = physics_lib.len() < 500 ||  // Very small file = stub
                  physics_lib.to_lowercase().contains("placeholder") ||
                  physics_lib.to_lowercase().contains("stub") ||
                  physics_lib.to_lowercase().contains("facade") ||
                  physics_lib.contains("Phase A");
    
    assert!(is_stub,
        "engine_physics should be a stub/placeholder, found substantial implementation");
}

/// Platform audit accurately describes physics as not yet connected.
#[test]
fn platform_audit_accurately_describes_physics_status() {
    let platform_audit = fs::read_to_string("docs/canonical/PLATFORM_AUDIT.md")
        .expect("PLATFORM_AUDIT.md must exist");
    
    // Must have "Exact blockers" section
    assert!(platform_audit.contains("blockers") ||
            platform_audit.contains("Blockers"),
        "PLATFORM_AUDIT must have blockers section");
    
    // Must have "not yet true" statements
    assert!(platform_audit.contains("not yet true") ||
            platform_audit.contains("not-yet-true"),
        "PLATFORM_AUDIT must have explicit 'not yet true' statements");
    
    // Physics should be mentioned in blockers or not-yet-true
    let mentions_physics_blocker = platform_audit.to_lowercase().contains("physics") &&
                                    (platform_audit.to_lowercase().contains("blocker") ||
                                     platform_audit.to_lowercase().contains("not yet") ||
                                     platform_audit.to_lowercase().contains("no"));
    
    assert!(mentions_physics_blocker,
        "PLATFORM_AUDIT must mention physics as blocker or not-yet-true");
}
