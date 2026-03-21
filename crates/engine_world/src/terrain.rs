//! Terrain generation and management
//!
//! Provides terrain generation, heightmap systems, and terrain masks.

use crate::coords::ChunkCoord;

/// Terrain heightmap
#[derive(Debug, Clone)]
pub struct Heightmap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl Heightmap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height],
        }
    }
    
    pub fn get_height(&self, x: usize, z: usize) -> f32 {
        if x >= self.width || z >= self.height {
            0.0
        } else {
            self.data[z * self.width + x]
        }
    }
    
    pub fn set_height(&mut self, x: usize, z: usize, height: f32) {
        if x < self.width && z < self.height {
            self.data[z * self.width + x] = height;
        }
    }
}

/// Terrain masks for spatial queries
#[derive(Debug, Clone, Copy)]
pub struct TerrainMask {
    pub bits: u32,
}

impl TerrainMask {
    pub const SOLID_GROUND: u32 = 0x00000001;
    pub const WATER: u32 = 0x00000002;
    pub const GRASS: u32 = 0x00000004;
    pub const STONE: u32 = 0x00000008;
    pub const SAND: u32 = 0x00000010;
    
    pub fn new(bits: u32) -> Self {
        Self { bits }
    }
    
    pub fn has_terrain(&self, terrain: u32) -> bool {
        (self.bits & terrain) != 0
    }
    
    pub fn add_terrain(&mut self, terrain: u32) {
        self.bits |= terrain;
    }
}
