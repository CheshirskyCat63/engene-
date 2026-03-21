//! Streaming Owner - Chunk residency management
//!
//! OWNER: engine_world::streaming::owner
//! PURPOSE: Complete streaming ownership system

use std::collections::{HashMap, HashSet};
use crate::coords::ChunkCoord;
use crate::chunk::{ChunkInfo, ChunkState};

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub max_loaded_chunks: usize,
    pub load_distance: i32,
    pub unload_distance: i32,
    pub max_operations_per_tick: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_loaded_chunks: 100,
            load_distance: 32,
            unload_distance: 64,
            max_operations_per_tick: 10,
        }
    }
}

/// Chunk residency state
#[derive(Debug, Clone)]
pub struct ChunkResidency {
    pub coord: ChunkCoord,
    pub state: ChunkState,
    pub load_priority: f32,
    pub last_access_tick: u64,
    pub entity_count: u32,
}

impl ChunkResidency {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            state: ChunkState::Unloaded,
            load_priority: 0.0,
            last_access_tick: 0,
            entity_count: 0,
        }
    }
}

/// Streaming update result
#[derive(Debug, Clone)]
pub enum StreamingUpdateResult {
    Success,
    BudgetExceeded,
    NetworkError,
    SerializationError,
}

/// Streaming Owner - manages chunk loading and unloading
#[derive(Debug, Clone)]
pub struct StreamingOwner {
    config: StreamingConfig,
    chunks: HashMap<(i32, i32), ChunkResidency>,
    loaded_chunks: HashSet<(i32, i32)>,
}

impl StreamingOwner {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            chunks: HashMap::new(),
            loaded_chunks: HashSet::new(),
        }
    }
    
    pub fn update(&mut self, player_pos: ChunkCoord) -> StreamingUpdateResult {
        // Generate chunks to load/unload based on player position
        let chunks_to_load = self.get_chunks_to_load(player_pos);
        let chunks_to_unload = self.get_chunks_to_unload(player_pos);
        
        // Check budget
        let total_operations = chunks_to_load.len() + chunks_to_unload.len();
        if total_operations > self.config.max_operations_per_tick {
            return StreamingUpdateResult::BudgetExceeded;
        }
        
        // Process unload operations
        for coord in chunks_to_unload {
            self.unload_chunk(&coord);
        }
        
        // Process load operations
        for coord in chunks_to_load {
            self.load_chunk(coord);
        }
        
        StreamingUpdateResult::Success
    }
    
    fn get_chunks_to_load(&self, player_pos: ChunkCoord) -> Vec<ChunkCoord> {
        let mut chunks = Vec::new();
        let load_distance_sq = self.config.load_distance * self.config.load_distance;
        
        for x in -1..=1 {
            for z in -1..=1 {
                let coord = ChunkCoord::new(
                    player_pos.x + x,
                    player_pos.z + z,
                );
                if coord.distance_sq(&player_pos) <= load_distance_sq {
                    chunks.push(coord);
                }
            }
        }
        
        chunks
    }
    
    fn get_chunks_to_unload(&self, player_pos: ChunkCoord) -> Vec<ChunkCoord> {
        let mut chunks = Vec::new();
        let unload_distance_sq = self.config.unload_distance * self.config.unload_distance;
        
        for (coord, _) in &self.loaded_chunks {
            if coord.distance_sq(&player_pos) > unload_distance_sq {
                chunks.push(coord);
            }
        }
        
        // Remove chunks that would be outside max loaded chunks
        while self.loaded_chunks.len() > self.config.max_loaded_chunks {
            if let Some(furthest) = chunks.pop() {
                self.loaded_chunks.remove(furthest);
            }
        }
        
        chunks
    }
    
    fn load_chunk(&mut self, coord: ChunkCoord) {
        let key = (coord.x, coord.z);
        let residency = ChunkResidency::new(coord);
        self.chunks.insert((coord.x, coord.z), residency);
        self.loaded_chunks.insert((coord.x, coord.z));
    }
    
    fn unload_chunk(&mut self, coord: &ChunkCoord) {
        let key = (coord.x, coord.z);
        self.chunks.remove(&key);
        self.loaded_chunks.remove(&key);
    }
    
    pub fn get_chunk_state(&self, coord: &ChunkCoord) -> Option<&ChunkState> {
        let key = (coord.x, coord.z);
        self.chunks.get(&key).map(|r| &r.state)
    }
    
    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoord> {
        self.loaded_chunks.iter().map(|&(x, z)| ChunkCoord::new(x, z)).collect()
    }
}
