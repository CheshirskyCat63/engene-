use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::streaming::ChunkCoord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StreamPriority {
    Critical = 0,
    Gameplay = 1,
    RenderDetail = 2,
    Cosmetic = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BundleKind {
    Collision,
    Navigation,
    GameplayEntities,
    Terrain,
    RenderMeshes,
    Vegetation,
    Decals,
    Audio,
}

impl BundleKind {
    pub fn priority(&self) -> StreamPriority {
        match self {
            Self::Collision | Self::Navigation => StreamPriority::Critical,
            Self::GameplayEntities | Self::Terrain => StreamPriority::Gameplay,
            Self::RenderMeshes => StreamPriority::RenderDetail,
            Self::Vegetation | Self::Decals | Self::Audio => StreamPriority::Cosmetic,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub kind: BundleKind,
    pub size_bytes: u64,
    pub compressed_size: u64,
    pub asset_refs: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkPackage {
    pub coord: ChunkCoord,
    pub region_id: u32,
    pub bundles: Vec<Bundle>,
    pub metadata: ChunkMetadata,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub entity_count: u32,
    pub has_navigation: bool,
    pub has_destruction: bool,
    pub terrain_lod_levels: u8,
    pub biome_tags: Vec<String>,
}

impl ChunkPackage {
    pub fn new(coord: ChunkCoord, region_id: u32) -> Self {
        Self {
            coord,
            region_id,
            bundles: Vec::new(),
            metadata: ChunkMetadata::default(),
        }
    }

    pub fn add_bundle(&mut self, bundle: Bundle) {
        self.bundles.push(bundle);
    }

    pub fn total_size(&self) -> u64 {
        self.bundles.iter().map(|b| b.size_bytes).sum()
    }

    pub fn compressed_size(&self) -> u64 {
        self.bundles.iter().map(|b| b.compressed_size).sum()
    }

    pub fn bundles_by_priority(&self) -> Vec<&Bundle> {
        let mut sorted: Vec<&Bundle> = self.bundles.iter().collect();
        sorted.sort_by_key(|b| b.kind.priority());
        sorted
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildDescriptor {
    pub region_id: u32,
    pub chunk_coords: Vec<ChunkCoord>,
    pub total_bundles: usize,
    pub total_size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionDescriptor {
    pub id: u32,
    pub name: String,
    pub chunks: Vec<ChunkCoord>,
}

pub struct ChunkPackageRegistry {
    packages: HashMap<ChunkCoord, ChunkPackage>,
    regions: HashMap<u32, RegionDescriptor>,
}

impl ChunkPackageRegistry {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            regions: HashMap::new(),
        }
    }

    pub fn register_region(&mut self, region: RegionDescriptor) {
        self.regions.insert(region.id, region);
    }

    pub fn register_package(&mut self, pkg: ChunkPackage) {
        self.packages.insert(pkg.coord, pkg);
    }

    pub fn get_package(&self, coord: &ChunkCoord) -> Option<&ChunkPackage> {
        self.packages.get(coord)
    }

    pub fn build_descriptor(&self, region_id: u32) -> Option<BuildDescriptor> {
        let region = self.regions.get(&region_id)?;
        let mut total_bundles = 0;
        let mut total_size = 0u64;
        for coord in &region.chunks {
            if let Some(pkg) = self.packages.get(coord) {
                total_bundles += pkg.bundles.len();
                total_size += pkg.total_size();
            }
        }
        Some(BuildDescriptor {
            region_id,
            chunk_coords: region.chunks.clone(),
            total_bundles,
            total_size_bytes: total_size,
        })
    }

    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

impl Default for ChunkPackageRegistry {
    fn default() -> Self { Self::new() }
}
