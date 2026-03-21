//! World coordinates and chunk systems
//!
//! Provides coordinate systems for world positioning and chunk management.

use serde::{Deserialize, Serialize};

/// World coordinate in chunk space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    
    pub fn distance_sq(&self, other: &ChunkCoord) -> i32 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        dx * dx + dz * dz
    }
    
    pub fn distance(&self, other: &ChunkCoord) -> f32 {
        (self.distance_sq(other) as f32).sqrt()
    }
    
    pub fn manhattan_distance(&self, other: &ChunkCoord) -> i32 {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }
}

/// World position in continuous space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl WorldPos {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn from_chunk_coord(coord: ChunkCoord, chunk_size: f32) -> Self {
        Self {
            x: coord.x as f32 * chunk_size,
            y: 0.0,
            z: coord.z as f32 * chunk_size,
        }
    }
}

/// Region in world space
#[derive(Debug, Clone)]
pub struct WorldRegion {
    pub min: ChunkCoord,
    pub max: ChunkCoord,
}

impl WorldRegion {
    pub fn new(min: ChunkCoord, max: ChunkCoord) -> Self {
        Self { min, max }
    }
    
    pub fn contains(&self, coord: ChunkCoord) -> bool {
        coord.x >= self.min.x && coord.x <= self.max.x &&
        coord.z >= self.min.z && coord.z <= self.max.z
    }
    
    pub fn size(&self) -> (i32, i32) {
        (
            self.max.x - self.min.x + 1,
            self.max.z - self.min.z + 1,
        )
    }
}
