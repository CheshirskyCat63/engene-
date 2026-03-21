//! Streaming Owner - Complete world streaming management.
//!
//! OWNER: engine_world
//! This module provides complete streaming ownership including:
//! - Chunk residency management
//! - Load/unload orchestration  
//! - Resource budget enforcement
//! - State persistence integration

use std::collections::{HashMap, HashSet};
use crate::streaming::{ChunkCoord, ChunkInfo, ChunkState, CHUNK_SIZE};
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
    
    pub fn mark_unloaded(&mut self) {
        self.state = ChunkState::Unloaded;
        self.entity_count = 0;
    }
}

/// Streaming configuration and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub max_loaded_chunks: usize,
    pub load_distance_chunks: u32,
    pub unload_distance_chunks: u32,
    pub async_load_batch_size: usize,
    pub priority_distance_weight: f32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_loaded_chunks: 64,
            load_distance_chunks: 8,
            unload_distance_chunks: 12,
            async_load_batch_size: 4,
            priority_distance_weight: 1.0,
        }
    }
}

/// Streaming owner - manages complete world residency
pub struct StreamingOwner {
    config: StreamingConfig,
    chunks: HashMap<ChunkCoord, ChunkResidency>,
    player_position: Option<[f32; 3]>,
    current_tick: u64,
    load_queue: Vec<ChunkCoord>,
    unload_queue: Vec<ChunkCoord>,
}

impl StreamingOwner {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            chunks: HashMap::new(),
            player_position: None,
            current_tick: 0,
            load_queue: Vec::new(),
            unload_queue: Vec::new(),
        }
    }
    
    /// Update streaming state for new tick
    pub fn update(&mut self, tick: u64, player_position: Option<[f32; 3]>) -> StreamingUpdateResult {
        self.current_tick = tick;
        self.player_position = player_position;
        
        // Clear previous queues
        self.load_queue.clear();
        self.unload_queue.clear();
        
        if let Some(pos) = player_position {
            self.calculate_streaming_decisions(pos);
        } else {
            // No player - unload distant chunks
            self.calculate_unload_only_decisions();
        }
        
        StreamingUpdateResult {
            chunks_to_load: self.load_queue.clone(),
            chunks_to_unload: self.unload_queue.clone(),
            load_decisions: self.load_queue.len() as u32,
            unload_decisions: self.unload_queue.len() as u32,
            budget_saturation: self.is_budget_saturated(),
            total_loaded_chunks: self.get_loaded_chunk_count(),
        }
    }
    
    /// Get chunk residency state
    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&ChunkResidency> {
        self.chunks.get(coord)
    }
    
    /// Get mutable chunk residency state
    pub fn get_chunk_mut(&mut self, coord: &ChunkCoord) -> Option<&mut ChunkResidency> {
        self.chunks.get_mut(coord)
    }
    
    /// Check if chunk is loaded
    pub fn is_chunk_loaded(&self, coord: &ChunkCoord) -> bool {
        self.chunks.get(coord)
            .map(|r| r.is_loaded())
            .unwrap_or(false)
    }
    
    /// Get all loaded chunk coordinates
    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoord> {
        self.chunks.iter()
            .filter(|(_, r)| r.is_loaded())
            .map(|(coord, _)| *coord)
            .collect()
    }
    
    /// Get count of loaded chunks
    pub fn get_loaded_chunk_count(&self) -> usize {
        self.chunks.values()
            .filter(|r| r.is_loaded())
            .count()
    }
    
    /// Mark chunk as loaded (called by async loading system)
    pub fn mark_chunk_loaded(&mut self, coord: ChunkCoord, entity_count: u32) {
        if let Some(residency) = self.chunks.get_mut(&coord) {
            residency.mark_loaded(entity_count);
        }
    }
    
    /// Mark chunk as unloaded (called by persistence system)
    pub fn mark_chunk_unloaded(&mut self, coord: &ChunkCoord) {
        if let Some(residency) = self.chunks.get_mut(coord) {
            residency.mark_unloaded();
        }
    }
    
    /// Force load a chunk (debug/admin function)
    pub fn force_load_chunk(&mut self, coord: ChunkCoord) {
        let residency = self.chunks.entry(coord).or_insert_with(|| ChunkResidency::new(coord));
        if !residency.is_loaded() {
            residency.mark_loading(self.current_tick);
            self.load_queue.push(coord);
        }
    }
    
    /// Force unload a chunk (debug/admin function)
    pub fn force_unload_chunk(&mut self, coord: &ChunkCoord) {
        if let Some(residency) = self.chunks.get_mut(&coord) {
            if residency.is_loaded() {
                self.unload_queue.push(coord);
            }
        }
    }
    
    /// Calculate streaming decisions based on player position
    fn calculate_streaming_decisions(&mut self, player_pos: [f32; 3]) {
        let center_chunk = ChunkCoord::from_world(player_pos[0], player_pos[2]);
        let load_radius = self.config.load_distance_chunks as i32;
        let unload_radius = self.config.unload_distance_chunks as i32;
        
        // Generate load candidates
        for dx in -load_radius..=load_radius {
            for dz in -load_radius..=load_radius {
                let chunk_coord = ChunkCoord {
                    x: center_chunk.x + dx,
                    z: center_chunk.z + dz,
                };
                
                if !self.chunks.contains_key(&chunk_coord) {
                    // New chunk - add to load queue
                    let mut residency = ChunkResidency::new(chunk_coord);
                    residency.load_priority = self.calculate_priority(&chunk_coord, &center_chunk);
                    residency.mark_loading(self.current_tick);
                    self.chunks.insert(chunk_coord, residency);
                    self.load_queue.push(chunk_coord);
                }
            }
        }
        
        // Generate unload candidates
        for (coord, residency) in &self.chunks {
            if residency.is_loaded() {
                let distance = self.chunk_distance(coord, &center_chunk);
                if distance > unload_radius as f32 {
                    self.unload_queue.push(*coord);
                }
            }
        }
        
        // Enforce budget limits
        self.enforce_budget_limits();
    }
    
    /// Calculate unload-only decisions (no player position)
    fn calculate_unload_only_decisions(&mut self) {
        // Unload chunks that haven't been accessed recently
        let stale_threshold = 100; // ticks
        
        for (coord, residency) in &self.chunks {
            if residency.is_loaded() {
                let ticks_since_access = self.current_tick.saturating_sub(residency.last_access_tick);
                if ticks_since_access > stale_threshold {
                    self.unload_queue.push(*coord);
                }
            }
        }
    }
    
    /// Calculate priority for chunk loading
    fn calculate_priority(&self, chunk: &ChunkCoord, center: &ChunkCoord) -> f32 {
        let distance = self.chunk_distance(chunk, center);
        let distance_factor = 1.0 / (1.0 + distance);
        
        // Higher priority for chunks closer to player
        distance_factor * self.config.priority_distance_weight
    }
    
    /// Calculate distance between chunks
    fn chunk_distance(&self, a: &ChunkCoord, b: &ChunkCoord) -> f32 {
        let dx = (a.x - b.x) as f32;
        let dz = (a.z - b.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }
    
    /// Enforce budget limits on load/unload queues
    fn enforce_budget_limits(&mut self) {
        // Limit load queue to budget
        let current_loaded = self.get_loaded_chunk_count();
        let remaining_budget = self.config.max_loaded_chunks.saturating_sub(current_loaded);
        let max_loads = remaining_budget.min(self.config.async_load_batch_size);
        
        if self.load_queue.len() > max_loads {
            // Sort by priority and keep highest priority chunks
            self.load_queue.sort_by(|a, b| {
                let priority_a = self.chunks.get(a).map(|r| r.load_priority).unwrap_or(0.0);
                let priority_b = self.chunks.get(b).map(|r| r.load_priority).unwrap_or(0.0);
                priority_b.partial_cmp(&priority_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            self.load_queue.truncate(max_loads);
        }
        
        // Limit unload queue to prevent thrashing
        let max_unloads = 4;
        if self.unload_queue.len() > max_unloads {
            self.unload_queue.truncate(max_unloads);
        }
    }
    
    /// Check if streaming budget is saturated
    fn is_budget_saturated(&self) -> bool {
        let current_loaded = self.get_loaded_chunk_count();
        current_loaded >= self.config.max_loaded_chunks
    }
}

/// Result of streaming update
#[derive(Debug, Clone)]
pub struct StreamingUpdateResult {
    pub chunks_to_load: Vec<ChunkCoord>,
    pub chunks_to_unload: Vec<ChunkCoord>,
    pub load_decisions: u32,
    pub unload_decisions: u32,
    pub budget_saturation: bool,
    pub total_loaded_chunks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_streaming_owner_basic_functionality() {
        let config = StreamingConfig::default();
        let mut owner = StreamingOwner::new(config);
        
        // Test initial state
        assert_eq!(owner.get_loaded_chunk_count(), 0);
        assert!(!owner.is_budget_saturated());
        
        // Test force load
        let coord = ChunkCoord { x: 0, z: 0 };
        owner.force_load_chunk(coord);
        assert!(owner.is_chunk_loaded(&coord));
        
        // Test update with player position
        let result = owner.update(1, Some([0.0, 0.0, 0.0]));
        assert!(result.load_decisions > 0);
        assert!(!result.chunks_to_load.is_empty());
    }
    
    #[test]
    fn test_streaming_budget_enforcement() {
        let config = StreamingConfig {
            max_loaded_chunks: 4,
            load_distance_chunks: 2,
            unload_distance_chunks: 4,
            async_load_batch_size: 10,
            priority_distance_weight: 1.0,
        };
        let mut owner = StreamingOwner::new(config);
        
        // Load chunks up to budget
        for i in 0..6 {
            owner.force_load_chunk(ChunkCoord { x: i, z: 0 });
        }
        
        let result = owner.update(1, Some([0.0, 0.0, 0.0]));
        
        // Should respect budget limit
        assert!(result.total_loaded_chunks <= 4);
        assert!(owner.is_budget_saturated());
    }
}
