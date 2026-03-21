//! Spatial contracts and interfaces
//!
//! Defines contracts for spatial indexing and query operations.

use crate::coords::ChunkCoord;

/// Spatial query result
#[derive(Debug, Clone)]
pub enum SpatialQuery {
    ChunkInRadius(ChunkCoord, i32),
    ChunksInRegion(ChunkCoord, ChunkCoord),
    NearestChunks(ChunkCoord, usize),
}

/// Spatial query contract
pub trait SpatialQuery {
    fn execute(&self, state: &dyn SpatialState) -> Result<SpatialQueryResult, SpatialError>;
}

/// Spatial state contract
pub trait SpatialState {
    fn get_chunk_at(&self, coord: &ChunkCoord) -> Option<ChunkInfo>;
    fn get_chunks_in_region(&self, min: ChunkCoord, max: ChunkCoord) -> Vec<&ChunkCoord>;
    fn find_nearest_chunks(&self, coord: ChunkCoord, count: usize) -> Vec<&ChunkCoord>;
}

/// Spatial query result
#[derive(Debug, Clone)]
pub struct SpatialQueryResult {
    pub chunks_found: Vec<ChunkCoord>,
    pub operations_performed: usize,
}

/// Spatial errors
#[derive(Debug, Clone)]
pub enum SpatialError {
    ChunkNotFound,
    InvalidRegion,
    OutOfBounds,
    IndexCorrupted,
}
