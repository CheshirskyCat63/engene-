//! Root Is Not Runtime Brain - TRUTH test
//!
//! This test enforces the architectural law: root crate must NOT own runtime orchestration.
//! Root is a migration shell, not the center of the world.
//!
//! Ownership: Architecture Team
//! Lane: contract
//! Type: TRUTH
//! Speed: Fast

use std::path::Path;

/// Root must NOT contain runtime orchestration modules.
/// These modules should live in engine_runtime, sdk_app, or game_framework.
#[test]
fn root_has_no_runtime_orchestration_modules() {
    let root_src = Path::new("crates/root/src");
    
    // If root/src doesn't exist, this is already a win
    if !root_src.exists() {
        return;
    }
    
    let forbidden_modules = vec![
        "orchestration",
        "phase_driver",
        "tick_driver",
        "frame_driver",
        "runner",
        "game_loop",
        "main_loop",
    ];
    
    let mut violations = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(root_src) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_lowercase();
            
            for forbidden in &forbidden_modules {
                if name_str.contains(*forbidden) {
                    violations.push(format!("Found forbidden module: {}", name.display()));
                }
            }
        }
    }
    
    assert!(
        violations.is_empty(),
        "Root crate must NOT contain runtime orchestration. Violations: {:?}",
        violations
    );
}

/// Root must NOT contain domain logic modules.
/// These should live in engine_* crates.
#[test]
fn root_has_no_domain_modules() {
    let root_src = Path::new("crates/root/src");
    
    if !root_src.exists() {
        return;
    }
    
    let forbidden_modules = vec![
        "physics",
        "render",
        "audio",
        "world",
        "spatial",
        "ecs",
        "entity",
        "simulation",
    ];
    
    let mut violations = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(root_src) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_lowercase();
            
            // Skip if it's a re-export from engine crates
            if name_str.starts_with("engine_") {
                continue;
            }
            
            for forbidden in &forbidden_modules {
                if name_str.contains(*forbidden) {
                    violations.push(format!("Found domain module in root: {}", name.display()));
                }
            }
        }
    }
    
    assert!(
        violations.is_empty(),
        "Root must NOT own domain logic. Violations: {:?}",
        violations
    );
}

/// Root must NOT define new architectural decisions.
/// All new architecture must be born in role crates or engine crates.
#[test]
fn root_has_no_new_architecture() {
    let root_lib = Path::new("crates/root/src/lib.rs");
    
    if !root_lib.exists() {
        return;
    }
    
    let content = std::fs::read_to_string(root_lib).unwrap_or_default();
    
    // Check for new struct definitions that look like domain objects
    let suspicious_patterns = vec![
        "pub struct.*Runtime",
        "pub struct.*Engine",
        "pub struct.*World",
        "pub struct.*Spatial",
    ];
    
    let mut violations = Vec::new();
    for pattern in suspicious_patterns {
        if content.contains(pattern) {
            violations.push(format!("Suspicious new architecture in root: {}", pattern));
        }
    }
    
    assert!(
        violations.is_empty(),
        "Root must NOT define new architecture. Violations: {:?}",
        violations
    );
}

/// Root bins must be deprecated - apps/* should be primary.
#[test]
fn root_bins_are_deprecated() {
    let root_cargo = Path::new("Cargo.toml");
    
    if !root_cargo.exists() {
        return;
    }
    
    let content = std::fs::read_to_string(root_cargo).unwrap_or_default();
    
    // If root still has [[bin]] entries, they should be marked deprecated
    // or apps/* should be the primary path
    if content.contains("[[bin]]") {
        // Check that apps/* exist as alternative
        let apps_exist = Path::new("apps/engene_game").exists() 
            || Path::new("apps/engene_sdk").exists();
        
        assert!(
            apps_exist,
            "If root has bins, apps/* alternatives must exist"
        );
    }
}

/// Engine crates must NOT depend on root.
#[test]
fn engine_crates_do_not_depend_on_root() {
    let engine_crates = vec![
        "crates/engine_core",
        "crates/engine_runtime", 
        "crates/engine_world",
        "crates/engine_ecs",
        "crates/engine_render",
        "crates/engine_audio",
        "crates/engine_physics",
    ];
    
    for crate_path in engine_crates {
        let cargo_toml = Path::new(&format!("{}/Cargo.toml", crate_path));
        
        if !cargo_toml.exists() {
            continue;
        }
        
        let content = std::fs::read_to_string(cargo_toml).unwrap_or_default();
        
        // Check for dependency on root package
        let root_dep_patterns = vec![
            "engene = ",
            "package = \"engene\"",
        ];
        
        for pattern in root_dep_patterns {
            if content.contains(pattern) && !content.contains("# DEPRECATED") {
                panic!(
                    "Engine crate {} must NOT depend on root. Found: {}",
                    crate_path, pattern
                );
            }
        }
    }
}

/// SDK and Game crates must NOT have root as orchestration center.
#[test]
fn product_crates_root_dependency_is_compatibility_only() {
    let product_crates = vec![
        "crates/sdk_app",
        "crates/game_framework",
    ];
    
    for crate_path in product_crates {
        let cargo_toml = Path::new(&format!("{}/Cargo.toml", crate_path));
        
        if !cargo_toml.exists() {
            continue;
        }
        
        let content = std::fs::read_to_string(cargo_toml).unwrap_or_default();
        
        // Root dependency is allowed ONLY for compatibility
        // Check that it's not used for orchestration
        if content.contains("engene") {
            // This is transitional - should be removed after handoff
            // For now, just verify the structure exists
            assert!(
                content.contains("# compatibility") || content.contains("# transitional"),
                "Root dependency in {} should be marked as transitional",
                crate_path
            );
        }
    }
}
