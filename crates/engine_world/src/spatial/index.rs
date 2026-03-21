//! Spatial indexing system
//!
//! Provides spatial indexing for efficient world queries.

use std::collections::HashMap;
use crate::coords::ChunkCoord;
use crate::chunk::ChunkInfo;

/// Spatial index for fast chunk lookup
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    pub chunks: HashMap<(i32, i32), ChunkInfo>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, chunk: ChunkInfo) {
        let key = (chunk.coord.x, chunk.coord.z);
        self.chunks.insert(key, chunk);
    }
    
    pub fn get(&self, coord: &ChunkCoord) -> Option<&ChunkInfo> {
        let key = (coord.x, coord.z);
        self.chunks.get(&key)
    }
    
    pub fn remove(&mut self, coord: &ChunkCoord) -> Option<ChunkInfo> {
        let key = (coord.x, coord.z);
        self.chunks.remove(&key)
    }
    
    pub fn contains(&self, coord: &ChunkCoord) -> bool {
        let key = (coord.x, coord.z);
        self.chunks.contains_key(&key)
    }
    
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &ChunkInfo> {
        self.chunks.values()
    }
}
