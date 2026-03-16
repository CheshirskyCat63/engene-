use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CookVariant {
    Dev,
    Shipping,
    LowSpec,
}

#[derive(Clone, Debug)]
pub struct CookedArtifact {
    pub source_id: u64,
    pub variant: CookVariant,
    pub output_path: PathBuf,
    pub size_bytes: u64,
}

pub struct CookPipeline {
    artifacts: HashMap<(u64, CookVariant), CookedArtifact>,
    output_dir: PathBuf,
}

impl CookPipeline {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            artifacts: HashMap::new(),
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub fn cook_asset(&mut self, source_id: u64, variant: CookVariant, data: &[u8]) -> PathBuf {
        let filename = format!("{}_{:?}.cooked", source_id, variant);
        let output_path = self.output_dir.join(&filename);

        self.artifacts.insert((source_id, variant.clone()), CookedArtifact {
            source_id,
            variant,
            output_path: output_path.clone(),
            size_bytes: data.len() as u64,
        });

        output_path
    }

    pub fn get_artifact(&self, source_id: u64, variant: &CookVariant) -> Option<&CookedArtifact> {
        self.artifacts.get(&(source_id, variant.clone()))
    }

    pub fn total_cooked_size(&self) -> u64 {
        self.artifacts.values().map(|a| a.size_bytes).sum()
    }

    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }
}
