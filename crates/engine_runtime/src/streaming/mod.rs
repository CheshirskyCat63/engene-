use std::collections::{HashMap, HashSet};

use engine_world::chunk::{ChunkResidency, ChunkState};
use engine_world::coords::ChunkCoord;
use engine_world::chunk::CHUNK_SIZE;

/// Configuration for the streaming owner.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub max_loaded_chunks: usize,
    pub load_distance_chunks: i32,
    pub unload_distance_chunks: i32,
    pub async_load_batch_size: usize,
    pub priority_distance_weight: f32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_loaded_chunks: 16,
            load_distance_chunks: 6,
            unload_distance_chunks: 8,
            async_load_batch_size: 4,
            priority_distance_weight: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamingUpdateResult {
    pub chunks_to_load: Vec<ChunkCoord>,
    pub chunks_to_unload: Vec<ChunkCoord>,
    pub load_decisions: u32,
    pub unload_decisions: u32,
    pub budget_saturation: bool,
    pub total_loaded_chunks: usize,
}

/// Streaming state manager for chunk residency.
pub struct StreamingOwner {
    config: StreamingConfig,
    loaded_chunks: HashMap<ChunkCoord, ChunkResidency>,
}

impl StreamingOwner {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            loaded_chunks: HashMap::new(),
        }
    }

    pub fn force_load_chunk(&mut self, coord: ChunkCoord) {
        if let Some(entry) = self.loaded_chunks.get_mut(&coord) {
            entry.state = ChunkState::Loaded;
            entry.last_access_tick = 0;
            entry.load_priority = 0.0;
            return;
        }

        let mut residency = ChunkResidency::new(coord);
        residency.state = ChunkState::Loaded;
        residency.last_access_tick = 0;
        residency.load_priority = 1.0;
        self.loaded_chunks.insert(coord, residency);
    }

    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoord> {
        self.loaded_chunks.keys().cloned().collect()
    }

    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&ChunkResidency> {
        self.loaded_chunks.get(coord)
    }

    pub fn update(&mut self, tick: u64, player_position: Option<[f32; 3]>) -> StreamingUpdateResult {
        let position_chunk = player_position.map(|p| {
            ChunkCoord::new((p[0] / CHUNK_SIZE).floor() as i32, (p[2] / CHUNK_SIZE).floor() as i32)
        });

        let mut chunks_to_load = Vec::new();
        let mut chunks_to_unload = Vec::new();

        if let Some(center) = position_chunk {
            let mut target_set = HashSet::new();
            for dx in -self.config.load_distance_chunks..=self.config.load_distance_chunks {
                for dz in -self.config.load_distance_chunks..=self.config.load_distance_chunks {
                    let candidate = ChunkCoord::new(center.x + dx, center.z + dz);
                    target_set.insert(candidate);
                }
            }

            // Identify chunks to load (not already loaded)
            let mut candidates: Vec<ChunkCoord> = target_set
                .iter()
                .cloned()
                .filter(|coord| !self.loaded_chunks.contains_key(coord))
                .collect();

            // Sort by distance ascending (higher priority closer chunks)
            candidates.sort_by(|a, b| {
                let da = a.distance_sq(&center);
                let db = b.distance_sq(&center);
                da.cmp(&db)
            });

            let max_load = self.config.async_load_batch_size.min(candidates.len());
            let budget_saturation = candidates.len() > self.config.async_load_batch_size;

            for candidate in candidates.into_iter().take(max_load) {
                let mut residency = ChunkResidency::new(candidate);
                residency.state = ChunkState::Loaded;
                residency.last_access_tick = tick;
                residency.load_priority = 1.0 / (1.0 + candidate.distance(&center));
                self.loaded_chunks.insert(candidate, residency);
                chunks_to_load.push(candidate);
            }

            // Unload chunks outside unload distance
            let limit = self.config.unload_distance_chunks;
            let out_of_range: Vec<_> = self
                .loaded_chunks
                .iter()
                .filter(|(&coord, _)| coord.distance_sq(&center) > (limit * limit))
                .map(|(&coord, _)| coord)
                .collect();

            for coord in out_of_range {
                self.loaded_chunks.remove(&coord);
                chunks_to_unload.push(coord);
            }

            // Drop extra chunks over max total
            if self.loaded_chunks.len() > self.config.max_loaded_chunks {
                let mut sorted: Vec<_> = self.loaded_chunks.values().collect();
                sorted.sort_by(|a, b| {
                    b.coord
                        .distance_sq(&center)
                        .cmp(&a.coord.distance_sq(&center))
                });

                while self.loaded_chunks.len() > self.config.max_loaded_chunks {
                    if let Some(lone) = sorted.pop() {
                        let coord = lone.coord;
                        self.loaded_chunks.remove(&coord);
                        chunks_to_unload.push(coord);
                    } else {
                        break;
                    }
                }
            }

            // Ensure priorities updated
            for (_, residency) in self.loaded_chunks.iter_mut() {.
                residency.last_access_tick = tick;
                residency.load_priority = 1.0 / (1.0 + residency.coord.distance(&center));
            }

            StreamingUpdateResult {
                chunks_to_load,
                chunks_to_unload,
                load_decisions: chunks_to_load.len() as u32,
                unload_decisions: chunks_to_unload.len() as u32,
                budget_saturation,
                total_loaded_chunks: self.loaded_chunks.len(),
            }
        } else {
            // Editor mode: unload all chunks in one pass
            let existing: Vec<_> = self.loaded_chunks.keys().cloned().collect();
            for coord in existing.iter() {
                self.loaded_chunks.remove(coord);
            }
            chunks_to_unload = existing;

            StreamingUpdateResult {
                chunks_to_load: Vec::new(),
                chunks_to_unload,
                load_decisions: 0,
                unload_decisions: chunks_to_unload.len() as u32,
                budget_saturation: false,
                total_loaded_chunks: self.loaded_chunks.len(),
            }
        }
    }
}
