//! Persistence contracts and interfaces
//!
//! Defines contracts for persistence operations and state management.

use crate::coords::ChunkCoord;
use crate::chunk::{ChunkInfo, ChunkState};
use serde::{Deserialize, Serialize};

/// Persistence operation result
#[derive(Debug, Clone)]
pub enum PersistenceOperation {
    SaveChunk(ChunkCoord),
    LoadChunk(ChunkCoord),
    DeleteChunk(ChunkCoord),
    UpdateChunkSchema(ChunkCoord),
}

/// Persistence state contract
pub trait PersistenceState {
    fn get_chunk_state(&self, coord: &ChunkCoord) -> Option<&ChunkState>;
    fn set_chunk_state(&mut self, coord: ChunkCoord, state: ChunkState);
    fn get_saved_chunks(&self) -> Vec<&ChunkCoord>;
    fn get_schema_version(&self) -> u32;
}

/// Persistence operation contract
pub trait PersistenceOperation {
    fn execute(&self, state: &mut dyn PersistenceState) -> Result<(), PersistenceError>;
    fn get_cost(&self) -> PersistenceCost;
}

/// Persistence operation cost
#[derive(Debug, Clone, Copy)]
pub struct PersistenceCost {
    pub disk_io_mb: usize,
    pub serialization_time_ms: f32,
    pub compression_ratio: f32,
}

/// Persistence errors
#[derive(Debug, Clone)]
pub enum PersistenceError {
    DiskError,
    SerializationError,
    SchemaMismatch,
    OutOfSpace,
    PermissionDenied,
}
