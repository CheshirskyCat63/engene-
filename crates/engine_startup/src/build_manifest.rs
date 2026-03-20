use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BuildManifest {
    pub version: String,
    pub build_time: String,
    pub git_commit: Option<String>,
    pub features: Vec<String>,
    pub dependencies: HashMap<String, String>,
}

impl BuildManifest {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: chrono::Utc::now().to_rfc3339(),
            git_commit: None,
            features: vec![],
            dependencies: HashMap::new(),
        }
    }

    pub fn with_git_commit(mut self, commit: String) -> Self {
        self.git_commit = Some(commit);
        self
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    pub fn with_dependencies(mut self, deps: HashMap<String, String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn print_full(&self) {
        println!("=== Build Manifest ===");
        println!("Version: {}", self.version);
        println!("Build Time: {}", self.build_time);
        if let Some(ref commit) = self.git_commit {
            println!("Git Commit: {}", commit);
        }
        if !self.features.is_empty() {
            println!("Features: {}", self.features.join(", "));
        }
        if !self.dependencies.is_empty() {
            println!("Dependencies:");
            for (name, version) in &self.dependencies {
                println!("  {}: {}", name, version);
            }
        }
        println!("=====================");
    }
}