use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type ContentHash = u64;

pub fn hash_file_contents(data: &[u8]) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

pub fn hash_import_settings(settings: &str) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    settings.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetVersion {
    pub source_hash: ContentHash,
    pub settings_hash: ContentHash,
    pub derived_version: u32,
}

impl AssetVersion {
    pub fn needs_reimport(
        &self,
        current_source: ContentHash,
        current_settings: ContentHash,
    ) -> bool {
        self.source_hash != current_source || self.settings_hash != current_settings
    }
}
