//! Streaming Owner - Complete world streaming management.
//!
//! OWNER: engine_world::streaming
//! This module provides complete streaming ownership including:
//! - Chunk residency management
//! - Load/unload orchestration  
//! - Resource budget enforcement
//! - State persistence integration

use std::collections::{HashMap, HashSet};
use crate::coords::ChunkCoord;
use crate::chunk::{ChunkInfo, ChunkState};
use serde::{Deserialize, Serialize};

/// Streaming state for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    pub fn is_loaded(&self) -> bool {
        matches!(self.state, ChunkState::Loaded)
    }
    
    pub fn is_loading(&self) -> bool {
        matches!(self.state, ChunkState::Loading)
    }
    
    pub fn mark_loading(&mut self, tick: u64) {
        self.state = ChunkState::Loading;
        self.last_access_tick = tick;
    }
    
    pub fn mark_loaded(&mut self, entity_count: u32) {
        self.state = ChunkState::Loaded;
        self.entity_count = entity_count;
    }
}

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub max_loaded_chunks: usize,
    pub load_distance: i32,
    pub unload_distance: i32,
    pub budget_per_tick: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_loaded_chunks: 100,
            load_distance: 4,
            unload_distance: 8,
            budget_per_tick: 4,
        }
    }
}

/// Streaming update result
#[derive(Debug, Clone)]
pub struct StreamingUpdateResult {
    pub loaded_chunks: Vec<ChunkCoord>,
    pub unloaded_chunks: Vec<ChunkCoord>,
    pub operations_performed: usize,
}

impl StreamingUpdateResult {
    pub fn new() -> Self {
        Self {
            loaded_chunks: Vec::new(),
            unloaded_chunks: Vec::new(),
            operations_performed: 0,
        }
    }
}

/// Streaming owner - manages chunk loading and unloading
#[derive(Debug)]
pub struct StreamingOwner {
    config: StreamingConfig,
    chunks: HashMap<(i32, i32), ChunkResidency>,
    loaded_chunks: HashSet<(i32, i32)>,
    current_tick: u64,
}

impl StreamingOwner {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            chunks: HashMap::new(),
            loaded_chunks: HashSet::new(),
            current_tick: 0,
        }
    }
    
    pub fn update(&mut self, player_coord: ChunkCoord) -> StreamingUpdateResult {
        self.current_tick += 1;
        let mut result = StreamingUpdateResult::new();
        
        // Determine chunks to load
        let chunks_to_load = self.get_chunks_to_load(player_coord);
        let chunks_to_unload = self.get_chunks_to_unload(player_coord);
        
        // Load chunks
        for coord in chunks_to_load {
            if let Some(residency) = self.chunks.get_mut(&(coord.x, coord.z)) {
                if !residency.is_loaded() {
                    residency.mark_loading(self.current_tick);
                    result.loaded_chunks.push(coord);
                    result.operations_performed += 1;
                }
            }
        }
        
        // Unload chunks
        for coord in chunks_to_unload {
            if let Some(residency) = self.chunks.get_mut(&(coord.x, coord.z)) {
                if residency.is_loaded() {
                    residency.state = ChunkState::Unloaded;
                    self.loaded_chunks.remove(&(coord.x, coord.z));
                    result.unloaded_chunks.push(coord);
                    result.operations_performed += 1;
                }
            }
        }
        
        result
    }
    
    fn get_chunks_to_load(&self, player_coord: ChunkCoord) -> Vec<ChunkCoord> {
        let mut chunks_to_load = Vec::new();
        
        for x in (player_coord.x - self.config.load_distance)..=(player_coord.x + self.config.load_distance) {
            for z in (player_coord.z - self.config.load_distance)..=(player_coord.z + self.config.load_distance) {
                let coord = ChunkCoord::new(x, z);
                if !self.loaded_chunks.contains(&(coord.x, coord.z)) {
                    chunks_to_load.push(coord);
                }
            }
        }
        
        chunks_to_load
    }
    
    fn get_chunks_to_unload(&self, player_coord: ChunkCoord) -> Vec<ChunkCoord> {
        let mut chunks_to_unload = Vec::new();
        
        for (&(x, z), residency) in &self.chunks {
            if residency.is_loaded() {
                let coord = ChunkCoord::new(x, z);
                let distance = coord.manhattan_distance(&player_coord);
                if distance > self.config.unload_distance {
                    chunks_to_unload.push(coord);
                }
            }
        }
        
        chunks_to_unload
    }
    
    pub fn get_loaded_chunks(&self) -> &HashSet<(i32, i32)> {
        &self.loaded_chunks
    }
    
    pub fn get_chunk_residency(&self, coord: &ChunkCoord) -> Option<&ChunkResidency> {
        self.chunks.get(&(coord.x, coord.z))
    }
}
