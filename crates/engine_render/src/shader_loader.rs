//! Shader loading and hot reload support (Phase D.1)
//!
//! Provides external .wgsl shader loading from assets/shaders/
//! with fallback to embedded shaders.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Shader source storage
#[derive(Clone, Debug)]
pub struct ShaderSources {
    /// Name -> WGSL source code
    pub shaders: HashMap<String, String>,
}

impl Default for ShaderSources {
    fn default() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }
}

impl ShaderSources {
    /// Load all .wgsl files from a directory
    pub fn load_from_dir(dir: &Path) -> std::io::Result<Self> {
        let mut sources = Self::default();

        if !dir.exists() {
            tracing::info!(
                "shader directory does not exist: {}, using embedded fallback",
                dir.display()
            );
            return Ok(sources);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("wgsl") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path)?;
                    tracing::info!("loaded external shader: {} from {}", name, path.display());
                    sources.shaders.insert(name.to_string(), content);
                }
            }
        }

        tracing::info!("loaded {} external shaders", sources.shaders.len());
        Ok(sources)
    }

    /// Get shader source by name, returns None if not found
    pub fn get(&self, name: &str) -> Option<&String> {
        self.shaders.get(name)
    }

    /// Check if a shader exists
    pub fn has(&self, name: &str) -> bool {
        self.shaders.contains_key(name)
    }

    /// Number of loaded shaders
    pub fn count(&self) -> usize {
        self.shaders.len()
    }
}

/// Reload a single shader by name
pub fn reload_shader(name: &str, path: &Path) -> std::io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    tracing::info!("reloaded shader: {} from {}", name, path.display());
    Ok(Some(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sources() {
        let sources = ShaderSources::default();
        assert_eq!(sources.count(), 0);
        assert!(!sources.has("test"));
    }

    #[test]
    fn test_get_nonexistent() {
        let sources = ShaderSources::default();
        assert!(sources.get("nonexistent").is_none());
    }
}
