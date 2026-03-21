//! Chunk system for world management
//!
//! Provides chunk coordinate systems, state management, and spatial indexing.

use crate::coords::ChunkCoord;
use serde::{Deserialize, Serialize};

/// Chunk size in world units
pub const CHUNK_SIZE: f32 = 16.0;

/// Chunk state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded,
}

/// Chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub coord: ChunkCoord,
    pub state: ChunkState,
    pub entity_count: u32,
}

impl ChunkInfo {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            state: ChunkState::Unloaded,
            entity_count: 0,
        }
    }
    
    pub fn is_loaded(&self) -> bool {
        matches!(self.state, ChunkState::Loaded)
    }
    
    pub fn is_loading(&self) -> bool {
        matches!(self.state, ChunkState::Loading)
    }
}

/// Chunk residency information
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
    
    pub fn is_loaded(&self) -> bool {
        matches!(self.state, ChunkState::Loaded)
    }
    
    pub fn is_loading(&self) -> bool {
        matches!(self.state, ChunkState::Loading)
    }
}
