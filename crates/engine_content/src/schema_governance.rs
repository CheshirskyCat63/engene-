//! Schema & Content Governance (Phase D.7)
//!
//! Provides schema versioning, validation, and migration for content files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Current schema versions
pub const SCHEMA_VERSION_GAME: u32 = 1;
pub const SCHEMA_VERSION_ENTITY: u32 = 1;
pub const SCHEMA_VERSION_CHUNK: u32 = 1;
pub const SCHEMA_VERSION_SAVE: u32 = 1;

/// Schema version information for a content type
#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub name: String,
    pub version: u32,
    pub min_compatible: u32,
    pub max_compatible: u32,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
}

impl SchemaVersion {
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
            min_compatible: version.saturating_sub(2),
            max_compatible: version,
            deprecated: false,
            deprecation_message: None,
        }
    }

    pub fn is_compatible(&self, other_version: u32) -> bool {
        other_version >= self.min_compatible && other_version <= self.max_compatible
    }
}

/// Schema registry for tracking all schema versions
pub struct SchemaRegistry {
    schemas: HashMap<String, SchemaVersion>,
    migrations: HashMap<(String, u32, u32), Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: HashMap::new(),
            migrations: HashMap::new(),
        };

        // Register built-in schemas
        registry.register(SchemaVersion::new("game", SCHEMA_VERSION_GAME));
        registry.register(SchemaVersion::new("entity", SCHEMA_VERSION_ENTITY));
        registry.register(SchemaVersion::new("chunk", SCHEMA_VERSION_CHUNK));
        registry.register(SchemaVersion::new("save", SCHEMA_VERSION_SAVE));

        registry
    }

    pub fn register(&mut self, schema: SchemaVersion) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&SchemaVersion> {
        self.schemas.get(name)
    }

    pub fn register_migration<F>(&mut self, schema: &str, from: u32, to: u32, migrate: F)
    where
        F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static,
    {
        self.migrations
            .insert((schema.to_string(), from, to), Box::new(migrate));
    }

    pub fn migrate(&self, schema: &str, from: u32, to: u32, data: &[u8]) -> Option<Vec<u8>> {
        if from == to {
            return Some(data.to_vec());
        }

        // Try direct migration
        if let Some(migrator) = self.migrations.get(&(schema.to_string(), from, to)) {
            return Some(migrator(data));
        }

        // Try stepwise migration (from -> from+1 -> ... -> to)
        if from < to {
            let mut current_data = data.to_vec();
            let mut current_version = from;

            while current_version < to {
                let next_version = current_version + 1;
                if let Some(migrator) =
                    self.migrations
                        .get(&(schema.to_string(), current_version, next_version))
                {
                    current_data = migrator(&current_data);
                    current_version = next_version;
                } else {
                    return None; // Migration path broken
                }
            }

            return Some(current_data);
        }

        None
    }

    pub fn all_schemas(&self) -> impl Iterator<Item = &SchemaVersion> {
        self.schemas.values()
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Content file metadata
#[derive(Debug, Clone)]
pub struct ContentMetadata {
    pub path: PathBuf,
    pub schema_name: String,
    pub schema_version: u32,
    pub checksum: u64,
    pub size_bytes: u64,
    pub last_modified: std::time::SystemTime,
}

impl ContentMetadata {
    pub fn from_file(
        path: impl Into<PathBuf>,
        schema_name: impl Into<String>,
    ) -> std::io::Result<Self> {
        let path = path.into();
        let metadata = std::fs::metadata(&path)?;

        Ok(Self {
            schema_version: extract_version_from_file(&path).unwrap_or(1),
            checksum: compute_checksum(&path)?,
            size_bytes: metadata.len(),
            last_modified: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            path,
            schema_name: schema_name.into(),
        })
    }
}

fn extract_version_from_file(path: &Path) -> Option<u32> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 64];
    let bytes_read = file.read(&mut header).ok()?;

    let header_str = std::str::from_utf8(&header[..bytes_read]).ok()?;

    // Look for version marker like "version:1" or "schema_version:2"
    for line in header_str.lines().take(5) {
        if line.starts_with("version:") || line.starts_with("schema_version:") {
            let version_str: String = line
                .split(':')
                .nth(1)?
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            return version_str.parse().ok();
        }
    }

    None
}

fn compute_checksum(path: &Path) -> std::io::Result<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        buffer[..bytes_read].hash(&mut hasher);
    }

    Ok(hasher.finish())
}

/// Content validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub path: PathBuf,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub schema_version: u32,
    pub needs_migration: bool,
}

impl ValidationResult {
    pub fn pass(path: PathBuf, schema_version: u32) -> Self {
        Self {
            path,
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            schema_version,
            needs_migration: false,
        }
    }

    pub fn fail(path: PathBuf, error: impl Into<String>) -> Self {
        Self {
            path,
            valid: false,
            errors: vec![error.into()],
            warnings: Vec::new(),
            schema_version: 0,
            needs_migration: false,
        }
    }

    pub fn warn(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Content validator
pub struct ContentValidator {
    registry: SchemaRegistry,
}

impl ContentValidator {
    pub fn new(registry: SchemaRegistry) -> Self {
        Self { registry }
    }

    pub fn validate_file(&self, path: &Path, schema_name: &str) -> ValidationResult {
        // Check file exists
        if !path.exists() {
            return ValidationResult::fail(
                path.to_path_buf(),
                format!("File not found: {}", path.display()),
            );
        }

        // Get schema info
        let schema = match self.registry.get(schema_name) {
            Some(s) => s,
            None => {
                return ValidationResult::fail(
                    path.to_path_buf(),
                    format!("Unknown schema: {}", schema_name),
                )
            }
        };

        // Extract version from file
        let file_version = extract_version_from_file(path).unwrap_or(1);

        let mut result = ValidationResult::pass(path.to_path_buf(), file_version);

        // Check version compatibility
        if !schema.is_compatible(file_version) {
            result.valid = false;
            result.errors.push(format!(
                "Schema version {} is not compatible with current version {} (range: {}-{})",
                file_version, schema.version, schema.min_compatible, schema.max_compatible
            ));
            result.needs_migration = true;
        }

        // Check if deprecated
        if schema.deprecated {
            result.warnings.push(format!(
                "Schema '{}' is deprecated: {}",
                schema.name,
                schema
                    .deprecation_message
                    .as_deref()
                    .unwrap_or("no message")
            ));
        }

        result
    }

    pub fn validate_directory(&self, dir: &Path) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if !dir.exists() {
            return results;
        }

        let schema_map = [
            ("game.ron", "game"),
            ("entities", "entity"),
            ("chunks", "chunk"),
            ("save.ron", "save"),
        ];

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Determine schema type from path
            let schema_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|name| {
                    schema_map
                        .iter()
                        .find(|(pattern, _)| name.contains(pattern))
                        .map(|(_, schema)| *schema)
                })
                .unwrap_or("unknown");

            results.push(self.validate_file(path, schema_name));
        }

        results
    }

    pub fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }
}

impl Default for ContentValidator {
    fn default() -> Self {
        Self::new(SchemaRegistry::new())
    }
}

/// Generate content governance report
pub fn generate_governance_report(results: &[ValidationResult]) -> String {
    let mut report = String::new();

    report.push_str("# Content Governance Report\n\n");

    let valid = results.iter().filter(|r| r.valid).count();
    let invalid = results.iter().filter(|r| !r.valid).count();
    let warnings = results.iter().map(|r| r.warnings.len()).sum::<usize>();
    let needs_migration = results.iter().filter(|r| r.needs_migration).count();

    report.push_str("## Summary\n");
    report.push_str(&format!("- **Valid**: {}/{}\n", valid, results.len()));
    report.push_str(&format!("- **Invalid**: {}/{}\n", invalid, results.len()));
    report.push_str(&format!("- **Warnings**: {}\n", warnings));
    report.push_str(&format!("- **Needs Migration**: {}\n\n", needs_migration));

    if invalid > 0 {
        report.push_str("## Errors\n\n");
        for result in results.iter().filter(|r| !r.valid) {
            report.push_str(&format!(
                "### {} (v{})\n",
                result.path.display(),
                result.schema_version
            ));
            for error in &result.errors {
                report.push_str(&format!("- {}\n", error));
            }
            report.push_str("\n");
        }
    }

    if warnings > 0 {
        report.push_str("## Warnings\n\n");
        for result in results.iter().filter(|r| !r.warnings.is_empty()) {
            report.push_str(&format!("### {}\n", result.path.display()));
            for warning in &result.warnings {
                report.push_str(&format!("- {}\n", warning));
            }
            report.push_str("\n");
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_compatibility() {
        let v1 = SchemaVersion::new("test", 3);

        assert!(v1.is_compatible(3));
        assert!(v1.is_compatible(2));
        assert!(v1.is_compatible(1));
        assert!(!v1.is_compatible(0));
        assert!(!v1.is_compatible(4));
    }

    #[test]
    fn test_schema_registry() {
        let registry = SchemaRegistry::new();

        assert!(registry.get("game").is_some());
        assert!(registry.get("entity").is_some());
        assert!(registry.get("chunk").is_some());
        assert!(registry.get("save").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_schema_migration_stepwise() {
        let mut registry = SchemaRegistry::new();

        // Register migration from v1 to v2
        registry.register_migration("test", 1, 2, |data| {
            let mut result = data.to_vec();
            result.extend_from_slice(b"_v2");
            result
        });

        // Register migration from v2 to v3
        registry.register_migration("test", 2, 3, |data| {
            let mut result = data.to_vec();
            result.extend_from_slice(b"_v3");
            result
        });

        // Migrate from v1 to v3
        let result = registry.migrate("test", 1, 3, b"original");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"original_v2_v3");
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::pass(PathBuf::from("test.ron"), 1);
        result.warn("test warning");

        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_content_validator() {
        let validator = ContentValidator::default();

        // Validate non-existent file
        let result = validator.validate_file(Path::new("nonexistent.ron"), "game");
        assert!(!result.valid);
        assert!(result.errors[0].contains("not found"));
    }
}
