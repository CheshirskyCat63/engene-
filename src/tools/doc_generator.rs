//! Documentation Generator (Phase D.6)
//! 
//! Generates API documentation, READMEs, and inline docs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Documentation configuration
#[derive(Debug, Clone)]
pub struct DocConfig {
    /// Output directory for generated docs
    pub output_dir: PathBuf,
    /// Include private items
    pub include_private: bool,
    /// Generate README files
    pub generate_readmes: bool,
    /// Generate API docs
    pub generate_api_docs: bool,
    /// Source directory
    pub src_dir: PathBuf,
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs/generated"),
            include_private: false,
            generate_readmes: true,
            generate_api_docs: true,
            src_dir: PathBuf::from("src"),
        }
    }
}

/// Documentation generator
pub struct DocGenerator {
    config: DocConfig,
    modules: HashMap<String, ModuleDoc>,
}

#[derive(Debug, Default)]
struct ModuleDoc {
    name: String,
    description: String,
    status: String,
    integration: String,
    tests: String,
    functions: Vec<FunctionDoc>,
    structs: Vec<StructDoc>,
}

#[derive(Debug, Default)]
struct FunctionDoc {
    name: String,
    signature: String,
    description: String,
    stability: String,
}

#[derive(Debug, Default)]
struct StructDoc {
    name: String,
    description: String,
    fields: Vec<FieldDoc>,
}

#[derive(Debug, Default)]
struct FieldDoc {
    name: String,
    field_type: String,
    description: String,
}

impl DocGenerator {
    pub fn new(config: DocConfig) -> Self {
        Self {
            config,
            modules: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(DocConfig::default())
    }

    /// Generate all documentation
    pub fn generate(&mut self) -> Result<DocReport, Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Scan source modules
        self.scan_modules()?;

        // Generate API documentation
        if self.config.generate_api_docs {
            self.generate_api_docs()?;
        }

        // Generate README files
        if self.config.generate_readmes {
            self.generate_readmes()?;
        }

        // Generate module index
        self.generate_module_index()?;

        Ok(DocReport {
            modules_documented: self.modules.len(),
            files_generated: 0,
        })
    }

    fn scan_modules(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let src_dir = &self.config.src_dir;
        
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let module_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Skip internal dirs
                if module_name.starts_with('.') || module_name == "target" {
                    continue;
                }

                // Parse module mod.rs for documentation
                let mod_rs = path.join("mod.rs");
                if mod_rs.exists() {
                    let doc = self.parse_module_doc(&mod_rs, module_name)?;
                    self.modules.insert(module_name.to_string(), doc);
                }
            }
        }

        Ok(())
    }

    fn parse_module_doc(&self, path: &Path, name: &str) -> Result<ModuleDoc, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        
        let mut doc = ModuleDoc::default();
        doc.name = name.to_string();

        // Extract module documentation from file header
        let lines: Vec<&str> = content.lines().collect();
        let mut in_doc_comment = false;
        let mut doc_lines = Vec::new();

        for line in &lines {
            let trimmed = line.trim();
            
            if trimmed.starts_with("//!") {
                in_doc_comment = true;
                let doc_line = trimmed.trim_start_matches("//!").trim();
                doc_lines.push(doc_line.to_string());
            } else if in_doc_comment && trimmed.starts_with("//!") == false {
                in_doc_comment = false;
            }

            // Extract status/integration/tests annotations
            if trimmed.starts_with("# Status:") {
                doc.status = trimmed.trim_start_matches("# Status:").trim().to_string();
            }
            if trimmed.starts_with("# Integration:") {
                doc.integration = trimmed.trim_start_matches("# Integration:").trim().to_string();
            }
            if trimmed.starts_with("# Tests:") {
                doc.tests = trimmed.trim_start_matches("# Tests:").trim().to_string();
            }
        }

        // Join doc lines as description
        doc.description = doc_lines.join("\n");

        Ok(doc)
    }

    fn generate_api_docs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.config.output_dir.join("api.md");
        let mut content = String::new();

        content.push_str("# API Documentation\n\n");
        content.push_str("## Modules\n\n");

        for (name, module) in &self.modules {
            content.push_str(&format!("### `{}`\n\n", name));
            
            if !module.description.is_empty() {
                content.push_str(&format!("{}\n\n", module.description));
            }

            content.push_str(&format!(
                "- **Status**: {}\n\
                 - **Integration**: {}\n\
                 - **Tests**: {}\n\n",
                module.status, module.integration, module.tests
            ));
        }

        fs::write(output_path, content)?;
        Ok(())
    }

    fn generate_readmes(&self) -> Result<(), Box<dyn std::error::Error>> {
        let readme_path = self.config.output_dir.join("README.md");
        let mut content = String::new();

        content.push_str("# ENGENE Engine Documentation\n\n");
        content.push_str("Auto-generated documentation. See [ROADMAP.md](../ROADMAP.md) for development status.\n\n");
        content.push_str("## Modules\n\n");
        content.push_str("| Module | Status | Integration | Tests |\n");
        content.push_str("|--------|--------|-------------|-------|\n");

        for module in self.modules.values() {
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                module.name, module.status, module.integration, module.tests
            ));
        }

        content.push_str("\n## API Docs\n\n");
        content.push_str("- [API Documentation](api.md)\n");

        fs::write(readme_path, content)?;
        Ok(())
    }

    fn generate_module_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = self.config.output_dir.join("modules.json");
        
        let mut json = String::from("[\n");
        
        let modules: Vec<_> = self.modules.values().collect();
        for (i, module) in modules.iter().enumerate() {
            json.push_str(&format!(
                "  {{\"name\": \"{}\", \"status\": \"{}\", \"integration\": \"{}\"}}",
                module.name, module.status, module.integration
            ));
            if i < modules.len() - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }
        
        json.push_str("]\n");
        
        fs::write(index_path, json)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DocReport {
    pub modules_documented: usize,
    pub files_generated: usize,
}

impl DocReport {
    pub fn summary(&self) -> String {
        format!(
            "Documentation generated:\n\
             - Modules documented: {}\n\
             - Files generated: {}",
            self.modules_documented, self.files_generated
        )
    }
}

/// Run documentation generation
pub fn run_docs() -> Result<DocReport, Box<dyn std::error::Error>> {
    let mut generator = DocGenerator::with_default_config();
    generator.generate()
}

/// Generate quick reference card
pub fn generate_quick_reference(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# ENGENE Quick Reference

## Building
```bash
cargo build --release
cargo test --lib
cargo run --bin engene_game
```

## Development
```bash
# Run with debug UI
cargo run --bin engene_sdk

# Run headless server
cargo run --bin engene_headless

# Run benchmarks
cargo bench
```

## Configuration
- Config files: `game/data/*.ron`
- Assets: `assets/`
- Shaders: `assets/shaders/`

## Key Systems
- ECS: `src/core/ecs.rs`
- Renderer: `src/graphics/renderer.rs`  
- AI: `src/ai/`
- Physics: Physics integration via Rapier3D
- World: `src/world/`

## Editor
- Run `engene_sdk` binary for editor UI
- Panels: Inspector, SceneHierarchy, Profiler, EventMonitor

## Testing
```bash
# Unit tests
cargo test

# Integration tests  
cargo test --test '*'

# Torture tests
cargo test --test save_load_torture
```
"#;

    fs::write(output_dir.join("quick_reference.md"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_config_default() {
        let config = DocConfig::default();
        assert!(config.generate_readmes);
        assert!(config.generate_api_docs);
    }

    #[test]
    fn test_module_doc_default() {
        let doc = ModuleDoc::default();
        assert!(doc.name.is_empty());
    }
}
