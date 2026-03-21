//! Test Architecture Validation
//! 
//! Meta-tests that validate the test architecture itself.
//! Ownership: Architecture Team
//! Lane: smoke
//! Type: Meta Tests
//! Speed: Fast

#[cfg(test)]
mod architecture_validation_tests {
    use std::path::Path;
    use std::fs;

    #[test]
    fn no_files_over_500_lines() {
        let tests_dir = Path::new("tests");
        let violations = find_files_over_limit(tests_dir, 500);
        
        assert!(violations.is_empty(), 
            "Files over 500 lines found: {:?}", violations);
    }

    #[test]
    fn all_test_files_have_canonical_headers() {
        let tests_dir = Path::new("tests");
        let violations = find_files_without_headers(tests_dir);
        
        assert!(violations.is_empty(),
            "Files missing canonical headers: {:?}", violations);
    }

    #[test]
    fn no_local_production_mocks() {
        let tests_dir = Path::new("tests");
        let violations = find_files_with_local_mocks(tests_dir);
        
        assert!(violations.is_empty(),
            "Files with local production mocks: {:?}", violations);
    }

    #[test]
    fn lane_names_are_canonical() {
        let canonical_lanes = vec![
            "smoke", "contracts", "tools", "perf", "certification"
        ];
        let tests_dir = Path::new("tests");
        let violations = find_files_with_invalid_lanes(tests_dir, &canonical_lanes);
        
        assert!(violations.is_empty(),
            "Files with non-canonical lane names: {:?}", violations);
    }

    #[test]
    fn no_duplicate_invariants_across_suites() {
        let tests_dir = Path::new("tests");
        let violations = find_duplicate_invariants(tests_dir);
        
        assert!(violations.is_empty(),
            "Duplicate invariants found: {:?}", violations);
    }

    #[test]
    fn megasuites_eliminated() {
        let tests_dir = Path::new("tests");
        let megasuites = find_files_over_limit(tests_dir, 500);
        
        assert!(megasuites.is_empty(),
            "Megasuites still exist: {:?}", megasuites);
    }

    #[test]
    fn engine_contracts_is_thin_aggregator() {
        let engine_contracts_path = Path::new("tests/engine_contracts.rs");
        let content = fs::read_to_string(engine_contracts_path)
            .expect("Should read engine_contracts.rs");
        
        // Should not have large test implementations
        let test_impl_count = content.matches("#[test]").count();
        assert!(test_impl_count <= 5, 
            "engine_contracts.rs should be thin aggregator, found {} tests", test_impl_count);
        
        // Should have pub use statements for aggregation
        let pub_use_count = content.matches("pub use").count();
        assert!(pub_use_count >= 5,
            "engine_contracts.rs should aggregate suites, found {} pub use", pub_use_count);
    }
}

#[cfg(test)]
mod src_cleanup_validation_tests {
    use std::path::Path;
    use std::fs;

    #[test]
    fn no_old_domain_directories_in_src() {
        let src_dir = Path::new("src");
        let forbidden_dirs = vec![
            "game", "graphics", "world", "tools", "simulation", "body"
        ];
        
        for forbidden_dir in &forbidden_dirs {
            let path = src_dir.join(forbidden_dir);
            assert!(!path.exists(),
                "Old domain directory still exists: {:?}", path);
        }
    }

    #[test]
    fn no_duplicate_crates_and_src() {
        let src_dir = Path::new("src");
        let crates_dir = Path::new("crates");
        
        let src_modules = find_modules_in_dir(src_dir);
        let crate_names = find_crate_names_in_dir(crates_dir);
        
        let duplicates: Vec<_> = src_modules.iter()
            .filter(|mod_name| crate_names.contains(mod_name))
            .collect();
            
        assert!(duplicates.is_empty(),
            "Duplicate modules found in src and crates: {:?}", duplicates);
    }
}

// Helper functions
fn find_files_over_limit(dir: &Path, limit: usize) -> Vec<String> {
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let line_count = content.lines().count();
                    if line_count > limit {
                        violations.push(format!("{} ({} lines)", 
                            path.display(), line_count));
                    }
                }
            }
        }
    }
    
    violations
}

fn find_files_without_headers(dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if !has_canonical_header(&content) {
                        violations.push(path.display().to_string());
                    }
                }
            }
        }
    }
    
    violations
}

fn find_files_with_local_mocks(dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if has_local_production_mocks(&content) {
                        violations.push(path.display().to_string());
                    }
                }
            }
        }
    }
    
    violations
}

fn find_files_with_invalid_lanes(dir: &Path, canonical_lanes: &[&str]) -> Vec<String> {
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(lane) = extract_lane_from_header(&content) {
                        if !canonical_lanes.contains(&lane.as_str()) {
                            violations.push(format!("{}: lane '{}'", path.display(), lane));
                        }
                    }
                }
            }
        }
    }
    
    violations
}

fn find_duplicate_invariants(dir: &Path) -> Vec<String> {
    let mut invariants = std::collections::HashMap::new();
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        if line.trim().starts_with("fn ") && 
                           (line.contains("invariant") || line.contains("contract")) {
                            let invariant_name = extract_invariant_name(line);
                            if let Some(name) = invariant_name {
                                if let Some(prev_file) = invariants.get(&name) {
                                    violations.push(format!("Invariant '{}' in both {} and {}", 
                                        name, prev_file, path.display()));
                                } else {
                                    invariants.insert(name, path.display().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    violations
}

fn find_modules_in_dir(dir: &Path) -> Vec<String> {
    let mut modules = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    modules.push(name.to_string());
                }
            } else if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|n| n.to_str()) {
                    if stem != "mod" && stem != "lib" {
                        modules.push(stem.to_string());
                    }
                }
            }
        }
    }
    
    modules
}

fn find_crate_names_in_dir(dir: &Path) -> Vec<String> {
    let mut crates = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("engine_") {
                        crates.push(name.strip_prefix("engine_").unwrap_or(name).to_string());
                    }
                }
            }
        }
    }
    
    crates
}

fn has_canonical_header(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().take(10).collect();
    let header_content = lines.join("\n");
    
    header_content.contains("//! Ownership:") &&
    header_content.contains("//! Lane:") &&
    header_content.contains("//! Type:") &&
    header_content.contains("//! Speed:")
}

fn has_local_production_mocks(content: &str) -> bool {
    // Look for struct/impl definitions that look like production types
    let lines: Vec<&str> = content.lines().collect();
    let mut in_test_module = false;
    
    for line in lines {
        if line.trim().starts_with("#[cfg(test)]") {
            in_test_module = true;
            continue;
        }
        
        if in_test_module {
            // Look for production-like struct definitions
            if line.trim().starts_with("struct ") && 
               !line.contains("Test") && 
               !line.contains("Mock") &&
               !line.contains("Helper") {
                return true;
            }
            
            // Look for impl blocks for non-test types
            if line.trim().starts_with("impl ") &&
               !line.contains("Test") &&
               !line.contains("Mock") {
                return true;
            }
        }
    }
    
    false
}

fn extract_lane_from_header(content: &str) -> Option<String> {
    for line in content.lines().take(10) {
        if line.trim().starts_with("//! Lane:") {
            return Some(line.trim().strip_prefix("//! Lane:").unwrap().trim().to_string());
        }
    }
    None
}

fn extract_invariant_name(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with("fn ") {
        let name_part = line.strip_prefix("fn ").unwrap();
        let name_end = name_part.find('(').unwrap_or(name_part.len());
        Some(name_part[..name_end].trim().to_string())
    } else {
        None
    }
}
