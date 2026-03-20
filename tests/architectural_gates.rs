//! Architectural Gates - Enforces crate ownership rules
//! 
//! These tests FAIL if legacy patterns are used.
//! Run with: cargo test --test architectural_gates
//! 
//! GATE 1: No legacy imports from root modules
//! GATE 2: No root ownership growth  
//! GATE 3: Apps launch only from canonical crates

use std::fs;
use std::path::Path;

#[cfg(test)]
mod gates {
    use super::*;

    fn scan_files_for_pattern(dir: &str, pattern: &str) -> Vec<String> {
        let mut matches = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    matches.extend(scan_files_for_pattern(&path.to_string_lossy(), pattern));
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            for (line_num, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    matches.push(format!("{}:{}: {}", path.display(), line_num + 1, line.trim()));
                                }
                            }
                        }
                    }
                }
            }
        }
        matches
    }

    #[test]
    fn gate_1_no_legacy_imports() {
        // This test searches for forbidden legacy import patterns
        // If any are found, the test FAILS
        
        let forbidden_patterns = [
            "use crate::graphics::",
            "use crate::audio::", 
            "use crate::world::",
            "use crate::body::",
            "use crate::tools::",
            "use crate::game::",
            "use crate::input::",
            "use crate::memory::",
            "use crate::navigation::",
            "use crate::network::",
            "use crate::simulation::",
            "use crate::core::system::EngineSystem",
            "use crate::engine::",
        ];

        let mut violations = Vec::new();
        
        for pattern in &forbidden_patterns {
            let matches = scan_files_for_pattern("src", pattern);
            if !matches.is_empty() {
                violations.push(format!("Pattern '{}':\n{}", pattern, matches.join("\n")));
            }
        }

        if !violations.is_empty() {
            panic!("GATE 1 VIOLATIONS - Legacy imports detected:\n\n{}", violations.join("\n\n"));
        }
    }

    #[test]
    fn gate_2_no_root_ownership_growth() {
        // This test ensures root doesn't grow new owner-like modules
        // Root should only contain thin compat layers
        
        let allowed_root_modules = [
            "app",      // transitional app layer
            "runtime",  // transitional runtime layer  
            "core",     // transitional core layer
            "testsupport", // testing compatibility
        ];

        let lib_path = "src/lib.rs";
        let content = fs::read_to_string(lib_path)
            .expect("Could not read src/lib.rs");

        let mut found_modules = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            // Check for both pub mod and private mod
            if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
                let module_name = trimmed.strip_prefix("pub mod ").unwrap_or_else(|| {
                    trimmed.strip_prefix("mod ").unwrap()
                })
                    .split(';').next().unwrap()
                    .trim();
                found_modules.push(module_name.to_string());
            }
        }

        let mut violations = Vec::new();
        for module in &found_modules {
            if !allowed_root_modules.contains(&module.as_str()) {
                violations.push(module.clone());
            }
        }

        // Also check for physical directories that shouldn't exist
        let src_dir = "src";
        if let Ok(entries) = fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(name_str) = dir_name.to_str() {
                            if !allowed_root_modules.contains(&name_str) && 
                               name_str != "bin" && // Allow bin directory
                               !name_str.starts_with('.') { // Skip hidden dirs
                                violations.push(format!("{} (physical directory)", name_str));
                            }
                        }
                    }
                }
            }
        }

        if !violations.is_empty() {
            panic!("GATE 2 VIOLATIONS - Disallowed root modules found: {:?}\nFound modules: {:?}\nAllowed: {:?}", 
                   violations, found_modules, allowed_root_modules);
        }
    }

    #[test]
    fn gate_3_apps_launch_from_canonical_crates() {
        // This test ensures apps only use canonical crates for launch
        
        let expected_app_launches = [
            ("engene_game", "game_framework::run_from_env_args"),
            ("engene_sdk", "sdk_app::run_from_env_args"), 
            ("engene_headless", "game_framework::run_headless_from_env_args"),
        ];

        let mut violations = Vec::new();

        for (app_name, expected_launch) in &expected_app_launches {
            let main_path = format!("apps/{}/src/main.rs", app_name);
            
            if !Path::new(&main_path).exists() {
                violations.push(format!("{}: missing main.rs", app_name));
                continue;
            }

            let content = fs::read_to_string(&main_path)
                .unwrap_or_else(|_| panic!("Could not read {}", main_path));

            // Check for the expected launch pattern
            if !content.contains(expected_launch) {
                violations.push(format!("{}: does not contain expected launch '{}'", app_name, expected_launch));
            }

            // Check for forbidden patterns (direct root usage)
            let forbidden_patterns = [
                "use engene::",
                "use crate::",
                "engene::run",
            ];

            for pattern in &forbidden_patterns {
                if content.contains(pattern) {
                    violations.push(format!("{}: contains forbidden pattern '{}'", app_name, pattern));
                }
            }
        }

        if !violations.is_empty() {
            panic!("GATE 3 VIOLATIONS - App launch verification failed:\n\n{}", violations.join("\n"));
        }

        println!("GATE 3: All app entrypoints verified successfully");
    }
}
