//! Streaming contracts and interfaces
//!
//! OWNER: engine_world::streaming::contracts
//! PURPOSE: Streaming operation contracts and error types

use crate::coords::ChunkCoord;
use crate::chunk::{ChunkInfo, ChunkState};

/// Streaming operation result
#[derive(Debug, Clone)]
pub enum StreamingUpdateResult {
    Success,
    BudgetExceeded,
    NetworkError,
    SerializationError,
}

/// Streaming budget state
#[derive(Debug, Clone)]
pub struct StreamingBudget {
    pub max_operations_per_tick: usize,
    pub max_memory_mb: usize,
    pub max_bandwidth_mb_per_sec: f32,
}

impl Default for StreamingBudget {
    fn default() -> Self {
        Self {
            max_operations_per_tick: 10,
            max_memory_mb: 512,
            max_bandwidth_mb_per_sec: 10.0,
        }
    }
}

/// Streaming state contract
pub trait StreamingState {
    fn get_chunk_state(&self, coord: &ChunkCoord) -> Option<&ChunkState>;
    fn set_chunk_state(&mut self, coord: ChunkCoord, state: ChunkState);
    fn get_loaded_chunks(&self) -> Vec<&ChunkCoord>;
    fn get_budget(&self) -> &StreamingBudget;
}

/// Streaming operation contract
pub trait StreamingExecutor {
    fn execute(&self, state: &mut dyn StreamingState) -> Result<(), StreamingUpdateResult>;
    fn get_cost(&self) -> StreamingCost;
}

/// Streaming operation cost
#[derive(Debug, Clone, Copy)]
pub struct StreamingCost {
    pub memory_mb: usize,
    pub bandwidth_mb: f32,
    pub time_ms: f32,
}
