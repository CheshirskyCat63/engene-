use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
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

    pub fn handle_version_flag() -> bool {
        std::env::args().any(|arg| arg == "--version" || arg == "-v")
    }

    pub fn ensure_data_dirs() {
        let dirs = [
            "game",
            "game/logs",
            "game/crashes",
            "game/saves",
            "game/saves/chunks",
        ];
        for dir in dirs {
            std::fs::create_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("Failed to create directory {}: {}", dir, e);
            });
        }
    }

    pub fn current() -> Self {
        Self::new()
    }

    pub fn write_manifest_json(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write("game/logs/last_manifest.json", json).unwrap_or_else(|e| {
            eprintln!("Failed to write manifest: {}", e);
        });
    }
}
