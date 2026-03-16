use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::streaming::ChunkCoord;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnDescriptor {
    pub prefab_name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
    pub overrides: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkAuthoring {
    pub coord: ChunkCoord,
    pub spawns: Vec<SpawnDescriptor>,
    pub terrain_layer: Option<String>,
    pub biome: Option<String>,
    pub nav_hints: Vec<NavHint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NavHint {
    BlockedRect { min: [f32; 2], max: [f32; 2] },
    CoverPoint { position: [f32; 3], direction: [f32; 2] },
    Waypoint { position: [f32; 3], tags: Vec<String> },
}

impl ChunkAuthoring {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            spawns: Vec::new(),
            terrain_layer: None,
            biome: None,
            nav_hints: Vec::new(),
        }
    }

    pub fn add_spawn(&mut self, spawn: SpawnDescriptor) {
        self.spawns.push(spawn);
    }
}

pub struct WorldAuthoringDatabase {
    chunks: HashMap<ChunkCoord, ChunkAuthoring>,
}

impl WorldAuthoringDatabase {
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    pub fn set_chunk(&mut self, auth: ChunkAuthoring) {
        self.chunks.insert(auth.coord, auth);
    }

    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&ChunkAuthoring> {
        self.chunks.get(coord)
    }

    pub fn load_from_dir(&mut self, dir: &std::path::Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        match ron::from_str::<ChunkAuthoring>(&contents) {
                            Ok(auth) => {
                                self.set_chunk(auth);
                                count += 1;
                            }
                            Err(e) => {
                                println!("[world-auth] failed to load {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        count
    }

    pub fn total_spawns(&self) -> usize {
        self.chunks.values().map(|c| c.spawns.len()).sum()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn coords(&self) -> impl Iterator<Item = &ChunkCoord> {
        self.chunks.keys()
    }
}

impl Default for WorldAuthoringDatabase {
    fn default() -> Self { Self::new() }
}
