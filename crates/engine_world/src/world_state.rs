//! World state management
//!
//! Provides world state tracking and management systems.

use std::collections::HashMap;

/// World state container
#[derive(Debug, Clone)]
pub struct WorldState {
    pub chunks: HashMap<(i32, i32), ChunkInfo>,
    pub tick: u64,
    pub entity_count: u64,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            tick: 0,
            entity_count: 0,
        }
    }
    
    pub fn add_chunk(&mut self, chunk: ChunkInfo) {
        let key = (chunk.coord.x, chunk.coord.z);
        self.chunks.insert(key, chunk);
        self.entity_count += chunk.entity_count as u64;
    }
    
    pub fn remove_chunk(&mut self, coord: &ChunkCoord) -> Option<ChunkInfo> {
        let key = (coord.x, coord.z);
        let chunk = self.chunks.remove(&key)?;
        self.entity_count = self.entity_count.saturating_sub(chunk.entity_count as u64);
        Some(chunk)
    }
    
    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&ChunkInfo> {
        let key = (coord.x, coord.z);
        self.chunks.get(&key)
    }
    
    pub fn tick(&mut self) {
        self.tick += 1;
    }
    
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}
