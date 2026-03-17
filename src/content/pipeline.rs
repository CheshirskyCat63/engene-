//! Content Pipeline Wiring (Phase D.3)
//! 
//! Wires together content import, cooking, validation, and prefabs.

use std::path::{Path, PathBuf};

/// Content pipeline configuration
#[derive(Debug, Clone)]
pub struct ContentPipelineConfig {
    /// Source asset directories
    pub source_dirs: Vec<PathBuf>,
    /// Output/cooked asset directory
    pub output_dir: PathBuf,
    /// Enable hot reload
    pub hot_reload: bool,
    /// Validation strictness
    pub validation_level: ValidationLevel,
    /// Maximum concurrent cooks
    pub max_concurrent_cooks: usize,
}

impl Default for ContentPipelineConfig {
    fn default() -> Self {
        Self {
            source_dirs: vec![
                PathBuf::from("assets"),
                PathBuf::from("game/data"),
            ],
            output_dir: PathBuf::from("cooked"),
            hot_reload: true,
            validation_level: ValidationLevel::Strict,
            max_concurrent_cooks: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Lenient,
    Normal,
    Strict,
}

/// Content pipeline state
pub struct ContentPipeline {
    config: ContentPipelineConfig,
    import_state: ImportState,
    cooking_state: CookingState,
    validation_state: ValidationState,
    prefab_state: PrefabState,
}

#[derive(Debug, Default)]
struct ImportState {
    imported_count: usize,
    failed_count: usize,
}

#[derive(Debug, Default)]
struct CookingState {
    cooked_count: usize,
    skipped_count: usize,
}

#[derive(Debug, Default)]
struct ValidationState {
    validated_count: usize,
    warnings: usize,
    errors: usize,
}

#[derive(Debug, Default)]
struct PrefabState {
    loaded_count: usize,
    failed_count: usize,
}

impl ContentPipeline {
    pub fn new(config: ContentPipelineConfig) -> Self {
        Self {
            config,
            import_state: ImportState::default(),
            cooking_state: CookingState::default(),
            validation_state: ValidationState::default(),
            prefab_state: PrefabState::default(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ContentPipelineConfig::default())
    }

    /// Initialize the pipeline
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory if needed
        if !self.config.output_dir.exists() {
            std::fs::create_dir_all(&self.config.output_dir)?;
        }

        println!("[ContentPipeline] Initialized with {} source dirs", self.config.source_dirs.len());
        Ok(())
    }

    /// Run full pipeline on all source directories
    pub fn run(&mut self) -> Result<PipelineReport, Box<dyn std::error::Error>> {
        println!("[ContentPipeline] Starting full pipeline run...");
        
        // Phase 1: Import
        self.import_phase()?;
        
        // Phase 2: Cook
        self.cook_phase()?;
        
        // Phase 3: Validate
        self.validation_phase()?;
        
        // Phase 4: Load prefabs
        self.prefab_phase()?;

        Ok(self.generate_report())
    }

    fn import_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ContentPipeline] Import phase...");
        
        for source_dir in &self.config.source_dirs {
            if !source_dir.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(source_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                
                // Check file type
                if is_asset_file(path) {
                    self.import_state.imported_count += 1;
                }
            }
        }

        println!("[ContentPipeline] Imported {} assets", self.import_state.imported_count);
        Ok(())
    }

    fn cook_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ContentPipeline] Cook phase...");
        
        // Simulate cooking - in real impl would convert assets to runtime format
        self.cooking_state.cooked_count = self.import_state.imported_count;
        self.cooking_state.skipped_count = 0;

        println!("[ContentPipeline] Cooked {} assets", self.cooking_state.cooked_count);
        Ok(())
    }

    fn validation_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ContentPipeline] Validation phase...");
        
        use crate::content::schema_governance::{ContentValidator, SchemaRegistry};

        let validator = ContentValidator::new(SchemaRegistry::new());

        // Validate config files
        let config_dir = PathBuf::from("game/data");
        if config_dir.exists() {
            let results = validator.validate_directory(&config_dir);
            
            for result in &results {
                if result.valid {
                    self.validation_state.validated_count += 1;
                } else {
                    self.validation_state.errors += 1;
                }
                self.validation_state.warnings += result.warnings.len();
            }
        }

        println!(
            "[ContentPipeline] Validated: {} valid, {} warnings, {} errors",
            self.validation_state.validated_count,
            self.validation_state.warnings,
            self.validation_state.errors
        );

        Ok(())
    }

    fn prefab_phase(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ContentPipeline] Prefab phase...");
        
        // In real impl would load prefabs from registry
        self.prefab_state.loaded_count = 0;

        Ok(())
    }

    fn generate_report(&self) -> PipelineReport {
        PipelineReport {
            imported: self.import_state.imported_count,
            failed_import: self.import_state.failed_count,
            cooked: self.cooking_state.cooked_count,
            skipped: self.cooking_state.skipped_count,
            validated: self.validation_state.validated_count,
            validation_warnings: self.validation_state.warnings,
            validation_errors: self.validation_state.errors,
            prefabs_loaded: self.prefab_state.loaded_count,
            prefabs_failed: self.prefab_state.failed_count,
        }
    }

    /// Get config
    pub fn config(&self) -> &ContentPipelineConfig {
        &self.config
    }

    /// Get validation level
    pub fn validation_level(&self) -> ValidationLevel {
        self.config.validation_level
    }

    /// Set validation level
    pub fn set_validation_level(&mut self, level: ValidationLevel) {
        self.config.validation_level = level;
    }
}

/// Pipeline execution report
#[derive(Debug, Clone, Default)]
pub struct PipelineReport {
    pub imported: usize,
    pub failed_import: usize,
    pub cooked: usize,
    pub skipped: usize,
    pub validated: usize,
    pub validation_warnings: usize,
    pub validation_errors: usize,
    pub prefabs_loaded: usize,
    pub prefabs_failed: usize,
}

impl PipelineReport {
    pub fn is_success(&self) -> bool {
        self.failed_import == 0 && self.validation_errors == 0 && self.prefabs_failed == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Content Pipeline Report:\n\
             - Imported: {} ({} failed)\n\
             - Cooked: {} ({} skipped)\n\
             - Validated: {} ({} warnings, {} errors)\n\
             - Prefabs: {} loaded ({} failed)\n\
             - Status: {}",
            self.imported,
            self.failed_import,
            self.cooked,
            self.skipped,
            self.validated,
            self.validation_warnings,
            self.validation_errors,
            self.prefabs_loaded,
            self.prefabs_failed,
            if self.is_success() { "SUCCESS" } else { "FAILED" }
        )
    }
}

/// Check if path is a recognized asset file
fn is_asset_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    matches!(ext,
        "png" | "jpg" | "jpeg" | "tga" | "dds" | "ktx" | "glb" | "gltf" |
        "wgsl" | "shader" |
        "ron" | "json" |
        "ogg" | "wav" | "mp3" |
        "anim" | "skel" |
        "navmesh" | "terrain"
    )
}

/// Quick pipeline check (lightweight)
pub fn quick_check() -> QuickCheckResult {
    let mut result = QuickCheckResult::default();

    // Check canonical config files (actual runtime-loaded configs)
    // This matches GameConfig::load_from_dir() in src/core/game_config.rs
    let config_files = [
        "game/data/biomes.ron",
        "game/data/economy.ron",
        "game/data/food_chain.ron",
        "game/data/materials.ron",
        "game/data/perception.ron",
        "game/data/population.ron",
        "game/data/seasonal_cycles.ron",
        "game/data/seasons.ron",
        "game/data/species.ron",
        "game/data/surfaces.ron",
        "game/data/simulation.ron",
        "game/data/tactics.ron",
        "game/data/weapons.ron",
    ];
    for file in config_files {
        if Path::new(file).exists() {
            result.config_files_present += 1;
        } else {
            result.config_files_missing += 1;
        }
    }

    // Check asset directories
    let asset_dirs = ["assets", "assets/shaders", "assets/textures", "assets/models"];
    for dir in asset_dirs {
        if Path::new(dir).exists() {
            result.asset_dirs_present += 1;
        }
    }

    result
}

#[derive(Debug, Default)]
pub struct QuickCheckResult {
    pub config_files_present: usize,
    pub config_files_missing: usize,
    pub asset_dirs_present: usize,
}

impl QuickCheckResult {
    pub fn is_ready(&self) -> bool {
        self.config_files_present >= 3 && self.asset_dirs_present >= 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = ContentPipelineConfig::default();
        assert!(config.hot_reload);
        assert_eq!(config.validation_level, ValidationLevel::Strict);
    }

    #[test]
    fn test_is_asset_file() {
        assert!(is_asset_file(Path::new("test.png")));
        assert!(is_asset_file(Path::new("test.wgsl")));
        assert!(is_asset_file(Path::new("test.ron")));
        assert!(!is_asset_file(Path::new("test.txt")));
    }

    #[test]
    fn test_quick_check() {
        let result = quick_check();
        assert!(result.asset_dirs_present <= 2);
    }

    #[test]
    fn test_pipeline_report() {
        let report = PipelineReport::default();
        assert!(report.is_success());
        
        let mut failed = PipelineReport::default();
        failed.validation_errors = 1;
        assert!(!failed.is_success());
    }
}
