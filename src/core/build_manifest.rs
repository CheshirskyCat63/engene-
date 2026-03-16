//! Build manifest and save compatibility — Contract 4 (Save Schema & Migration Policy).
//!
//! Provides compile-time engine metadata and runtime schema versioning
//! for saves, chunks, and entity snapshots.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_VERSION_SAVE: u32 = 1;
pub const SCHEMA_VERSION_CHUNK: u32 = 1;
pub const SCHEMA_VERSION_ENTITY: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub engine_version: String,
    pub build_timestamp: String,
    pub git_hash: Option<String>,
    pub profile: String,
    pub feature_flags: Vec<String>,
    pub schema_version_save: u32,
    pub schema_version_chunk: u32,
    pub schema_version_entity: u32,
}

impl BuildManifest {
    pub fn current() -> Self {
        Self {
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            build_timestamp: option_env!("ENGENE_BUILD_TIME")
                .unwrap_or("unknown")
                .to_string(),
            git_hash: option_env!("ENGENE_GIT_HASH").map(|s| s.to_string()),
            profile: option_env!("ENGENE_BUILD_PROFILE")
                .unwrap_or(if cfg!(debug_assertions) { "dev" } else { "release" })
                .to_string(),
            feature_flags: Self::active_features(),
            schema_version_save: SCHEMA_VERSION_SAVE,
            schema_version_chunk: SCHEMA_VERSION_CHUNK,
            schema_version_entity: SCHEMA_VERSION_ENTITY,
        }
    }

    /// Returns true if `--version` was found in args (and printed manifest).
    /// Callers should exit early when this returns true.
    pub fn handle_version_flag() -> bool {
        if std::env::args().any(|a| a == "--version") {
            let m = Self::current();
            m.print_full();
            return true;
        }
        false
    }

    pub fn print_full(&self) {
        println!("  Engine:   v{}", self.engine_version);
        println!("  Profile:  {}", self.profile);
        println!(
            "  Git:      {}",
            self.git_hash.as_deref().unwrap_or("unknown")
        );
        println!("  Built:    {}", self.build_timestamp);
        println!(
            "  Features: [{}]",
            self.feature_flags.join(", ")
        );
        println!(
            "  Schema:   save=v{} chunk=v{} entity=v{}",
            self.schema_version_save, self.schema_version_chunk, self.schema_version_entity
        );
    }

    fn active_features() -> Vec<String> {
        let from_build = option_env!("ENGENE_FEATURE_FLAGS").unwrap_or("");
        if !from_build.is_empty() {
            return from_build.split(',').map(|s| s.to_string()).collect();
        }
        let mut flags = Vec::new();
        if cfg!(feature = "physics") {
            flags.push("physics".into());
        }
        if cfg!(feature = "render") {
            flags.push("render".into());
        }
        if cfg!(feature = "ai") {
            flags.push("ai".into());
        }
        if cfg!(feature = "audio") {
            flags.push("audio".into());
        }
        if cfg!(feature = "debug_ui") {
            flags.push("debug_ui".into());
        }
        if cfg!(feature = "headless") {
            flags.push("headless".into());
        }
        if cfg!(feature = "sdk_tools") {
            flags.push("sdk_tools".into());
        }
        if cfg!(feature = "low_spec") {
            flags.push("low_spec".into());
        }
        if cfg!(feature = "networking") {
            flags.push("networking".into());
        }
        if cfg!(feature = "body_sim") {
            flags.push("body_sim".into());
        }
        if cfg!(feature = "audio_playback") {
            flags.push("audio_playback".into());
        }
        flags
    }

    pub fn ensure_data_dirs() {
        for dir in &[
            "game/data",
            "game/world/chunks",
            "game/world/layouts",
            "game/world/prefabs",
            "game/assets",
            "game/saves",
            "game/replays",
            "game/logs",
            "game/crashes",
        ] {
            let _ = std::fs::create_dir_all(dir);
        }
    }

    pub fn write_manifest_json(&self) {
        let path = "game/logs/last_manifest.json";
        if let Ok(s) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = std::fs::write(path, s);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub entries: Vec<AssetEntry>,
    pub total_size_bytes: u64,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub hash: u64,
}

impl AssetManifest {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            total_size_bytes: 0,
            schema_version: SCHEMA_VERSION_SAVE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCompatibility {
    pub min_engine_version: String,
    pub current_engine_version: String,
    pub schema_version_save: u32,
    pub schema_version_chunk: u32,
    pub schema_version_entity: u32,
    pub breaking_changes: Vec<String>,
}

impl SaveCompatibility {
    pub fn current() -> Self {
        Self {
            min_engine_version: env!("CARGO_PKG_VERSION").to_string(),
            current_engine_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version_save: SCHEMA_VERSION_SAVE,
            schema_version_chunk: SCHEMA_VERSION_CHUNK,
            schema_version_entity: SCHEMA_VERSION_ENTITY,
            breaking_changes: Vec::new(),
        }
    }

    pub fn is_compatible(&self, save_version: u32) -> bool {
        save_version <= self.schema_version_save
    }
}

pub type MigrationFn = fn(old_version: u32, data: Vec<u8>) -> Result<Vec<u8>, MigrationError>;

#[derive(Debug)]
pub enum MigrationError {
    UnsupportedVersion(u32),
    DataCorruption(String),
    MissingHandler { from: u32, to: u32 },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported schema version: {}", v),
            Self::DataCorruption(msg) => write!(f, "data corruption during migration: {}", msg),
            Self::MissingHandler { from, to } => {
                write!(f, "no migration handler from v{} to v{}", from, to)
            }
        }
    }
}

pub struct SchemaMigrationRegistry {
    save_handlers: HashMap<(u32, u32), MigrationFn>,
    chunk_handlers: HashMap<(u32, u32), MigrationFn>,
    entity_handlers: HashMap<(u32, u32), MigrationFn>,
}

impl SchemaMigrationRegistry {
    pub fn new() -> Self {
        Self {
            save_handlers: HashMap::new(),
            chunk_handlers: HashMap::new(),
            entity_handlers: HashMap::new(),
        }
    }

    pub fn register_save_migration(&mut self, from: u32, to: u32, handler: MigrationFn) {
        self.save_handlers.insert((from, to), handler);
    }

    pub fn register_chunk_migration(&mut self, from: u32, to: u32, handler: MigrationFn) {
        self.chunk_handlers.insert((from, to), handler);
    }

    pub fn register_entity_migration(&mut self, from: u32, to: u32, handler: MigrationFn) {
        self.entity_handlers.insert((from, to), handler);
    }

    pub fn migrate_save(&self, from: u32, data: Vec<u8>) -> Result<Vec<u8>, MigrationError> {
        self.run_chain(&self.save_handlers, from, SCHEMA_VERSION_SAVE, data)
    }

    pub fn migrate_chunk(&self, from: u32, data: Vec<u8>) -> Result<Vec<u8>, MigrationError> {
        self.run_chain(&self.chunk_handlers, from, SCHEMA_VERSION_CHUNK, data)
    }

    pub fn migrate_entity(&self, from: u32, data: Vec<u8>) -> Result<Vec<u8>, MigrationError> {
        self.run_chain(&self.entity_handlers, from, SCHEMA_VERSION_ENTITY, data)
    }

    fn run_chain(
        &self,
        handlers: &HashMap<(u32, u32), MigrationFn>,
        from: u32,
        to: u32,
        mut data: Vec<u8>,
    ) -> Result<Vec<u8>, MigrationError> {
        if from > to {
            return Err(MigrationError::UnsupportedVersion(from));
        }
        let mut current = from;
        while current < to {
            let next = current + 1;
            let handler = handlers
                .get(&(current, next))
                .ok_or(MigrationError::MissingHandler {
                    from: current,
                    to: next,
                })?;
            data = handler(current, data)?;
            current = next;
        }
        Ok(data)
    }

    pub fn has_path_save(&self, from: u32) -> bool {
        self.has_chain(&self.save_handlers, from, SCHEMA_VERSION_SAVE)
    }

    pub fn has_path_chunk(&self, from: u32) -> bool {
        self.has_chain(&self.chunk_handlers, from, SCHEMA_VERSION_CHUNK)
    }

    fn has_chain(&self, handlers: &HashMap<(u32, u32), MigrationFn>, from: u32, to: u32) -> bool {
        if from >= to {
            return from == to;
        }
        let mut current = from;
        while current < to {
            let next = current + 1;
            if !handlers.contains_key(&(current, next)) {
                return false;
            }
            current = next;
        }
        true
    }
}

impl Default for SchemaMigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
