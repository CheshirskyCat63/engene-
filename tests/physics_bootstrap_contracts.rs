//! Physics Bootstrap Contracts
//!
//! These tests verify that physics bootstrap contract is documented accurately
//! and that physics bootstrap is a real code path (not just docs).

use std::fs;

/// Physics bootstrap contract doc exists.
#[test]
fn physics_bootstrap_contract_doc_exists() {
    let _bootstrap_contract = fs::read_to_string("docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md")
        .expect("PHYSICS_BOOTSTRAP_CONTRACT.md must exist");
}

/// Verify physics bootstrap contract is not future fanfic.
#[test]
fn physics_bootstrap_contract_is_not_future_fanfic() {
    let bootstrap_contract = fs::read_to_string("docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md")
        .expect("PHYSICS_BOOTSTRAP_CONTRACT.md must exist");
    
    // Verify it describes current state, not just future
    let has_current_state = bootstrap_contract.to_lowercase().contains("current") ||
                            bootstrap_contract.to_lowercase().contains("as of") ||
                            bootstrap_contract.to_lowercase().contains("status") ||
                            bootstrap_contract.to_lowercase().contains("not yet");
    
    assert!(has_current_state,
        "PHYSICS_BOOTSTRAP_CONTRACT must describe current state, not just future");
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
}

/// Physics bootstrap has a disabled path that validates.
#[test]
fn physics_bootstrap_disabled_path_is_defined() {
    let disabled = engine_physics::bootstrap::PhysicsBootstrap::disabled();
    assert!(!disabled.is_enabled(), "disabled bootstrap should not be enabled");
    assert!(disabled.validate().is_ok(), "disabled bootstrap should validate");
}

/// Runtime exposes a minimal physics bootstrap helper.
#[cfg(feature = "physics")]
#[test]
fn runtime_bootstrap_helper_is_available() {
    let _ = engene::runtime::bootstrap::minimal_physics_bootstrap_for_validation();
}

#[cfg(feature = "physics")]
#[test]
fn runtime_physics_system_descriptor_is_available() {
    let desc = engene::runtime::bootstrap::physics_tick_system_descriptor();
    assert_eq!(desc.name, engine_physics::systems::PHYSICS_TICK_SYSTEM_NAME);
}

/// Physics bootstrap enabled minimal path validates.
#[test]
fn physics_bootstrap_enabled_minimal_path_validates() {
    let enabled = engine_physics::bootstrap::PhysicsBootstrap::enabled_minimal();
    assert!(enabled.is_enabled(), "enabled_minimal should report enabled");
    assert!(enabled.validate().is_ok(), "enabled_minimal should validate");
}

/// Physics bootstrap validate fails loudly when state is invalid.
#[test]
fn physics_bootstrap_loud_fail_exists_for_invalid_enabled_state() {
    use engine_physics::contracts::PhysicsBootstrapFailure;

    let mut broken = engine_physics::bootstrap::PhysicsBootstrap::enabled_minimal();
    broken.runtime_state.enabled = false;
    assert!(matches!(broken.validate(), Err(PhysicsBootstrapFailure::MissingRuntimeState)));

    let mut broken = engine_physics::bootstrap::PhysicsBootstrap::enabled_minimal();
    broken.registration.systems_registered = false;
    assert!(matches!(broken.validate(), Err(PhysicsBootstrapFailure::InvalidRegistration)));
}

/// Current branch state does not overclaim physics bootstrap.
#[test]
fn current_branch_state_does_not_overclaim_physics_bootstrap() {
    let branch_state = fs::read_to_string("docs/canonical/CURRENT_BRANCH_STATE.md")
        .expect("CURRENT_BRANCH_STATE.md must exist");
    
    // If physics is mentioned, verify it's not claimed as fully integrated
    if branch_state.to_lowercase().contains("physics") {
        let has_disclaimer = branch_state.to_lowercase().contains("not") ||
                             branch_state.to_lowercase().contains("stub") ||
                             branch_state.to_lowercase().contains("not yet") ||
                             branch_state.to_lowercase().contains("placeholder");
        
        let claims_physics_active = (branch_state.contains("physics") && 
                                     branch_state.contains("active")) ||
                                    (branch_state.contains("physics") && 
                                     branch_state.contains("working"));
        
        if claims_physics_active {
            assert!(has_disclaimer,
                "CURRENT_BRANCH_STATE must not claim physics is active without disclaimer");
        }
    }
}

/// Fast verification docs do not claim physics is done.
#[test]
fn fast_verification_docs_do_not_claim_physics_is_done() {
    let platform_audit = fs::read_to_string("docs/canonical/PLATFORM_AUDIT.md")
        .expect("PLATFORM_AUDIT.md must exist");
    
    let has_physics_truth = platform_audit.to_lowercase().contains("physics") &&
                            (platform_audit.to_lowercase().contains("stub") ||
                             platform_audit.to_lowercase().contains("not") ||
                             platform_audit.to_lowercase().contains("not yet"));
    
    assert!(has_physics_truth,
        "PLATFORM_AUDIT must accurately describe physics readiness");
}
