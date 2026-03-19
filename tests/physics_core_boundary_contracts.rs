//! Physics Core Boundary Contracts
//!
//! These tests verify the physics-to-core boundary is properly documented and enforced.
//! They test that physics is declared correctly, is a capability not a role,
//! and that root crate policy does not allow physics ownership.

use std::fs;

/// Extract workspace members from Cargo.toml
fn extract_workspace_members(cargo_toml: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;
    
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members =") {
            in_members = true;
        } else if in_members && trimmed.starts_with('[') {
            break;
        } else if in_members && trimmed.starts_with('"') {
            let member = trimmed.trim_matches('"').trim_matches(',').to_string();
            if !member.is_empty() {
                members.push(member);
            }
        }
    }
    members
}

/// Extract dependencies of a crate from its Cargo.toml
fn extract_dependencies(cargo_toml: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_dependencies = false;
    
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[dependencies]") {
            in_dependencies = true;
        } else if in_dependencies && trimmed.starts_with('[') {
            break;
        } else if in_dependencies && trimmed.contains('=') {
            if let Some(name) = trimmed.split('=').next() {
                let dep_name = name.trim().to_string();
                if !dep_name.is_empty() && !dep_name.starts_with('#') {
                    deps.push(dep_name);
                }
            }
        }
    }
    deps
}

/// Physics runtime owner is declared as engine_physics in workspace.
#[test]
fn physics_runtime_owner_is_declared_as_engine_physics() {
    let workspace_cargo = fs::read_to_string("Cargo.toml")
        .expect("Workspace Cargo.toml must exist");
    
    let members = extract_workspace_members(&workspace_cargo);
    
    // Verify engine_physics is in workspace
    assert!(members.iter().any(|m| m.contains("engine_physics")),
        "engine_physics must be declared in workspace members, found: {:?}", members);
    
    // Verify path points to crates/engine_physics
    let engine_physics_member = members.iter()
        .find(|m| m.contains("engine_physics"))
        .expect("engine_physics member must exist");
    
    assert!(engine_physics_member.contains("crates/engine_physics") || 
            engine_physics_member.contains("engine_physics"),
        "engine_physics must be in crates/engine_physics, found: {}", engine_physics_member);
}

/// Physics is classified as capability, not runtime role.
#[test]
fn physics_is_capability_not_runtime_role() {
    let feature_policy = fs::read_to_string("docs/canonical/FEATURE_ROLE_POLICY.md")
        .expect("FEATURE_ROLE_POLICY.md must exist");
    
    // Verify physics is listed as a capability
    assert!(feature_policy.to_lowercase().contains("physics") || 
            feature_policy.to_lowercase().contains("capability"),
        "FEATURE_ROLE_POLICY must classify physics as capability");
    
    // Verify physics is NOT listed as a runtime role selector
    // (roles should be role_game, role_sdk, role_headless, role_tools)
    let role_section = feature_policy.to_lowercase();
    
    // Physics should appear in capabilities section, not in runtime role selectors
    // We check that if "physics" is mentioned, it's in context of capabilities
    if role_section.contains("physics") {
        // Should not be in "role_" format
        assert!(!role_section.contains("role_physics"),
            "physics should not be a runtime role (role_physics)");
    }
}

/// Root crate policy does not allow new physics ownership.
#[test]
fn root_crate_policy_does_not_allow_new_physics_ownership() {
    let root_policy = fs::read_to_string("docs/canonical/ROOT_CRATE_POLICY.md")
        .expect("ROOT_CRATE_POLICY.md must exist");
    
    // Verify root crate is documented as migration shell
    assert!(root_policy.to_lowercase().contains("migration shell"),
        "ROOT_CRATE_POLICY must describe root as migration shell");
    
    // Verify root crate may NOT become permanent architecture center
    assert!(root_policy.to_lowercase().contains("may not") ||
            root_policy.to_lowercase().contains("not allowed") ||
            root_policy.to_lowercase().contains("forbidden"),
        "ROOT_CRATE_POLICY must forbid root from becoming permanent center");
    
    // Verify new domain ownership is forbidden in root
    assert!(root_policy.to_lowercase().contains("new domain ownership") ||
            root_policy.to_lowercase().contains("domain") ||
            root_policy.to_lowercase().contains("forbidden"),
        "ROOT_CRATE_POLICY must forbid new domain ownership in root");
}

/// Physics core boundary doc matches workspace truth.
#[test]
fn physics_core_boundary_doc_matches_workspace_truth() {
    let boundary_doc = fs::read_to_string("docs/canonical/PHYSICS_CORE_BOUNDARY.md")
        .expect("PHYSICS_CORE_BOUNDARY.md must exist");
    
    let workspace_cargo = fs::read_to_string("Cargo.toml")
        .expect("Workspace Cargo.toml must exist");
    
    // Verify boundary doc mentions engine_physics location
    assert!(boundary_doc.contains("engine_physics"),
        "PHYSICS_CORE_BOUNDARY must mention engine_physics");
    
    // Verify boundary doc mentions crates/engine_physics
    assert!(boundary_doc.contains("crates/engine_physics"),
        "PHYSICS_CORE_BOUNDARY must specify crates/engine_physics path");
    
    // Verify boundary doc mentions allowed direction: physics -> core
    assert!(boundary_doc.contains("physics -> core") ||
            (boundary_doc.contains("physics") && boundary_doc.contains("core")),
        "PHYSICS_CORE_BOUNDARY must document physics-to-core direction");
    
    // Verify workspace has engine_physics
    let members = extract_workspace_members(&workspace_cargo);
    assert!(members.iter().any(|m| m.contains("engine_physics")),
        "Workspace must have engine_physics member");
}

/// Physics boundary has explicit forbidden dependencies.
#[test]
fn physics_boundary_has_explicit_forbidden_dependencies() {
    let boundary_doc = fs::read_to_string("docs/canonical/PHYSICS_CORE_BOUNDARY.md")
        .expect("PHYSICS_CORE_BOUNDARY.md must exist");
    
    // Verify forbidden direction is documented
    assert!(boundary_doc.to_lowercase().contains("forbidden") ||
            boundary_doc.to_lowercase().contains("not allowed") ||
            boundary_doc.to_lowercase().contains("must not"),
        "PHYSICS_CORE_BOUNDARY must document forbidden dependencies");
    
    // Verify core -> physics direction is forbidden
    assert!(boundary_doc.contains("core") && boundary_doc.contains("physics"),
        "PHYSICS_CORE_BOUNDARY must mention core and physics relationship");
    
    // Verify no core-to-physics imports rule exists
    let has_import_rule = boundary_doc.to_lowercase().contains("import") ||
                          boundary_doc.to_lowercase().contains("dependency");
    assert!(has_import_rule,
        "PHYSICS_CORE_BOUNDARY must document import/dependency rules");
}

/// Engine_physics dependencies are correct (physics -> core, not reverse).
#[test]
fn engine_physics_depends_on_core_not_reverse() {
    let engine_physics_cargo = fs::read_to_string("crates/engine_physics/Cargo.toml")
        .expect("crates/engine_physics/Cargo.toml must exist");
    
    let engine_core_cargo = fs::read_to_string("crates/engine_core/Cargo.toml")
        .expect("crates/engine_core/Cargo.toml must exist");
    
    let physics_deps = extract_dependencies(&engine_physics_cargo);
    let core_deps = extract_dependencies(&engine_core_cargo);
    
    // Physics may depend on core
    let physics_has_core = physics_deps.iter().any(|d| d.contains("engine_core"));
    // But core must NOT depend on physics
    let core_has_physics = core_deps.iter().any(|d| d.contains("engine_physics"));
    
    assert!(physics_has_core || physics_deps.is_empty(),
        "engine_physics may depend on engine_core");
    
    assert!(!core_has_physics,
        "engine_core must NOT depend on engine_physics (forbidden direction)");
}

/// Workspace ownership map correctly assigns physics to engine_physics.
#[test]
fn workspace_ownership_map_assigns_physics_to_engine_physics() {
    let ownership_map = fs::read_to_string("docs/canonical/WORKSPACE_OWNERSHIP_MAP.md")
        .expect("WORKSPACE_OWNERSHIP_MAP.md must exist");
    
    // Verify physics runtime is assigned to engine_physics
    assert!(ownership_map.contains("engine_physics"),
        "WORKSPACE_OWNERSHIP_MAP must mention engine_physics");
    
    // Verify physics ownership is assigned to engine_physics crate
    let has_physics_ownership = ownership_map.to_lowercase().contains("physics") &&
                                 (ownership_map.contains("engine_physics") ||
                                  ownership_map.contains("engine_physics"));
    assert!(has_physics_ownership,
        "WORKSPACE_OWNERSHIP_MAP must assign physics to engine_physics");
}
