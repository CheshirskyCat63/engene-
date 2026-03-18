use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: f32 = 1000.0; // 1 km

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn from_world(wx: f32, wz: f32) -> Self {
        Self {
            x: (wx / CHUNK_SIZE).floor() as i32,
            z: (wz / CHUNK_SIZE).floor() as i32,
        }
    }

    pub fn world_center(&self) -> (f32, f32) {
        (
            (self.x as f32 + 0.5) * CHUNK_SIZE,
            (self.z as f32 + 0.5) * CHUNK_SIZE,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded,
}

pub struct ChunkInfo {
    pub state: ChunkState,
    pub entity_count: u32,
}

pub struct WorldStreamer {
    pub chunks: HashMap<ChunkCoord, ChunkInfo>,
    pub load_radius: f32,
    pub unload_radius: f32,
}

impl WorldStreamer {
    pub fn new(load_radius: f32, unload_radius: f32) -> Self {
        Self {
            chunks: HashMap::new(),
            load_radius,
            unload_radius,
        }
    }

    pub fn update(&mut self, camera_x: f32, camera_z: f32) -> (Vec<ChunkCoord>, Vec<ChunkCoord>) {
        let cam_chunk = ChunkCoord::from_world(camera_x, camera_z);
        let load_chunks_r = (self.load_radius / CHUNK_SIZE).ceil() as i32;
        let unload_r2 = self.unload_radius * self.unload_radius;

        let mut needed = HashSet::new();
        for dz in -load_chunks_r..=load_chunks_r {
            for dx in -load_chunks_r..=load_chunks_r {
                let coord = ChunkCoord {
                    x: cam_chunk.x + dx,
                    z: cam_chunk.z + dz,
                };
                let (cx, cz) = coord.world_center();
                let dist2 = (cx - camera_x).powi(2) + (cz - camera_z).powi(2);
                if dist2 <= self.load_radius * self.load_radius {
                    needed.insert(coord);
                }
            }
        }

        let mut to_load = Vec::new();
        for &coord in &needed {
            if !self.chunks.contains_key(&coord) {
                self.chunks.insert(
                    coord,
                    ChunkInfo {
                        state: ChunkState::Loading,
                        entity_count: 0,
                    },
                );
                to_load.push(coord);
            }
        }

        let mut to_unload = Vec::new();
        let loaded: Vec<ChunkCoord> = self.chunks.keys().copied().collect();
        for coord in loaded {
            let (cx, cz) = coord.world_center();
            let dist2 = (cx - camera_x).powi(2) + (cz - camera_z).powi(2);
            if dist2 > unload_r2 {
                self.chunks.remove(&coord);
                to_unload.push(coord);
            }
        }

        (to_load, to_unload)
    }

    pub fn mark_loaded(&mut self, coord: ChunkCoord) {
        if let Some(info) = self.chunks.get_mut(&coord) {
            info.state = ChunkState::Loaded;
        }
    }

    pub fn is_loaded(&self, coord: &ChunkCoord) -> bool {
        self.chunks
            .get(coord)
            .map_or(false, |c| c.state == ChunkState::Loaded)
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks
            .values()
            .filter(|c| c.state == ChunkState::Loaded)
            .count()
    }

    pub fn total_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Begin a streaming transaction. Returns a snapshot that can be used to rollback
    /// if the load/unload operation fails partway through.
    pub fn begin_transaction(&self) -> StreamingTransaction {
        StreamingTransaction {
            snapshot: self.chunks.clone(),
        }
    }

    /// Rollback to a previous state if a streaming operation failed.
    pub fn rollback(&mut self, transaction: StreamingTransaction) {
        self.chunks = transaction.snapshot;
    }
}

pub struct StreamingTransaction {
    snapshot: HashMap<ChunkCoord, ChunkInfo>,
}

impl Clone for ChunkInfo {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            entity_count: self.entity_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coord_from_world() {
        let c1 = ChunkCoord::from_world(0.0, 0.0);
        assert_eq!(c1.x, 0);
        assert_eq!(c1.z, 0);

        let c2 = ChunkCoord::from_world(500.0, 500.0);
        assert_eq!(c2.x, 0);
        assert_eq!(c2.z, 0);

        let c3 = ChunkCoord::from_world(1000.0, 1000.0);
        assert_eq!(c3.x, 1);
        assert_eq!(c3.z, 1);

        let c4 = ChunkCoord::from_world(-500.0, -500.0);
        assert_eq!(c4.x, -1);
        assert_eq!(c4.z, -1);
    }

    #[test]
    fn test_chunk_coord_world_center() {
        let c = ChunkCoord { x: 0, z: 0 };
        let (cx, cz) = c.world_center();
        assert_eq!(cx, 500.0);
        assert_eq!(cz, 500.0);

        let c2 = ChunkCoord { x: 1, z: 2 };
        let (cx2, cz2) = c2.world_center();
        assert_eq!(cx2, 1500.0);
        assert_eq!(cz2, 2500.0);
    }

    #[test]
    fn test_streamer_load_unload() {
        let mut streamer = WorldStreamer::new(1500.0, 2500.0);

        // Camera at origin
        let (to_load, to_unload) = streamer.update(0.0, 0.0);

        // Should load chunks within 1500 radius
        assert!(!to_load.is_empty());
        assert!(to_unload.is_empty());

        // Mark first chunk as loaded
        if let Some(&coord) = to_load.first() {
            streamer.mark_loaded(coord);
            assert!(streamer.is_loaded(&coord));
        }
    }

    #[test]
    fn test_streamer_unload_distant() {
        let mut streamer = WorldStreamer::new(1500.0, 2500.0);

        // Load chunks at origin
        streamer.update(0.0, 0.0);

        // Move camera far away
        let (_, to_unload) = streamer.update(10000.0, 10000.0);

        // Should unload old chunks
        assert!(!to_unload.is_empty());
    }

    #[test]
    fn test_transaction_rollback() {
        let mut streamer = WorldStreamer::new(1500.0, 2500.0);

        // Initial load
        streamer.update(0.0, 0.0);
        let initial_count = streamer.total_chunk_count();
        assert!(initial_count > 0);

        // Begin transaction
        let tx = streamer.begin_transaction();

        // Make changes - move far away
        let _ = streamer.update(10000.0, 10000.0);
        let _after_move_count = streamer.total_chunk_count();

        // Rollback should restore original state
        streamer.rollback(tx);
        assert_eq!(streamer.total_chunk_count(), initial_count);
    }

    #[test]
    fn test_loaded_chunk_count() {
        let mut streamer = WorldStreamer::new(1500.0, 2500.0);

        assert_eq!(streamer.loaded_chunk_count(), 0);

        let (to_load, _) = streamer.update(0.0, 0.0);

        // Initially all are Loading, not Loaded
        assert_eq!(streamer.loaded_chunk_count(), 0);

        // Mark one as loaded
        if let Some(coord) = to_load.first() {
            streamer.mark_loaded(*coord);
            assert_eq!(streamer.loaded_chunk_count(), 1);
        }
    }
}
