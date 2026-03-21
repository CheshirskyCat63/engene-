use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::hashes::{hash_file_contents, hash_import_settings, AssetVersion};

pub type AssetId = u64;

#[derive(Clone, Debug)]
pub enum AssetType {
    Model,
    Texture,
    Material,
    Audio,
    Animation,
    CollisionProxy,
    FractureProxy,
    TerrainMask,
    CoverHint,
    Prefab,
}

#[derive(Clone, Debug)]
pub struct ImportRecord {
    pub asset_id: AssetId,
    pub asset_type: AssetType,
    pub source_path: PathBuf,
    pub version: AssetVersion,
    pub dependencies: Vec<AssetId>,
}

pub struct ImportPipeline {
    records: HashMap<AssetId, ImportRecord>,
    path_to_id: HashMap<PathBuf, AssetId>,
    next_id: AssetId,
    worker_count: usize,
}

impl ImportPipeline {
    pub fn new(worker_count: usize) -> Self {
        Self {
            records: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
            worker_count: worker_count.max(1),
        }
    }

    pub fn register_source(
        &mut self,
        path: &Path,
        asset_type: AssetType,
        settings: &str,
    ) -> AssetId {
        if let Some(&existing) = self.path_to_id.get(path) {
            return existing;
        }

        let source_hash = std::fs::read(path)
            .map(|data| hash_file_contents(&data))
            .unwrap_or(0);
        let settings_hash = hash_import_settings(settings);

        let id = self.next_id;
        self.next_id += 1;

        let record = ImportRecord {
            asset_id: id,
            asset_type,
            source_path: path.to_path_buf(),
            version: AssetVersion {
                source_hash,
                settings_hash,
                derived_version: 1,
            },
            dependencies: Vec::new(),
        };

        self.records.insert(id, record);
        self.path_to_id.insert(path.to_path_buf(), id);
        id
    }

    pub fn add_dependency(&mut self, asset: AssetId, depends_on: AssetId) {
        if let Some(record) = self.records.get_mut(&asset) {
            if !record.dependencies.contains(&depends_on) {
                record.dependencies.push(depends_on);
            }
        }
    }

    pub fn needs_reimport(&self, asset: AssetId) -> bool {
        let record = match self.records.get(&asset) {
            Some(r) => r,
            None => return false,
        };

        let current_source = std::fs::read(&record.source_path)
            .map(|data| hash_file_contents(&data))
            .unwrap_or(0);

        record
            .version
            .needs_reimport(current_source, record.version.settings_hash)
    }

    pub fn stale_assets(&self) -> Vec<AssetId> {
        self.records
            .keys()
            .filter(|&&id| self.needs_reimport(id))
            .copied()
            .collect()
    }

    pub fn get_record(&self, id: AssetId) -> Option<&ImportRecord> {
        self.records.get(&id)
    }

    pub fn asset_count(&self) -> usize {
        self.records.len()
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn parallel_batch_size(&self) -> usize {
        let total = self.records.len();
        if self.worker_count == 0 {
            return total;
        }
        (total / self.worker_count).max(1)
    }
}
