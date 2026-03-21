//! Streaming phase - world residency decisions.
//!
//! OWNER: engine_runtime / engine_world
//! Second phase in canonical order - runs after tick.
//!
//! ## Purpose
//! Streaming determines which world chunks should be loaded/unloaded based on
//! player position, view distance, and residency budget.
//!
//! ## Input Contract
//! - `tick`: Current tick number
//! - `player_position`: Player/view anchor position (from camera)
//! - `view_distance`: Render distance in chunks
//! - `pending_unload_intents`: Chunks flagged for unload by persistence
//! - `residency_budget`: Max concurrent loads allowed
//!
//! ## Output Contract
//! - `chunks_to_load`: List of chunk coords to async-load
//! - `chunks_to_unload`: List of chunk coords to unload
//! - `load_decisions`: Count of load decisions made
//! - `unload_decisions`: Count of unload decisions made
//! - `budget_saturation`: Whether budget was hit
//! - `duration_ms`: Phase execution time

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};

/// Streaming phase input parameters.
#[derive(Debug, Clone, Default)]
pub struct StreamingInput {
    pub tick: u64,
    pub player_position: Option<[f32; 3]>,
    pub view_distance_chunks: u32,
    pub pending_unload_count: u32,
    pub residency_budget: u32,
    pub known_loaded_chunks: Vec<[i32; 2]>, // Current resident set
}

/// Streaming phase output results.
#[derive(Debug, Clone, Default)]
pub struct StreamingOutput {
    pub chunks_to_load: Vec<[i32; 2]>,
    pub chunks_to_unload: Vec<[i32; 2]>,
    pub load_decisions: u32,
    pub unload_decisions: u32,
    pub budget_saturation: bool,
    pub duration_ms: f32,
}

pub struct StreamingPhase;

impl StreamingPhase {
    pub fn new() -> Self {
        Self
    }
}

/// Execute streaming phase - world residency decisions.
/// 
/// OWNER: engine_runtime::phase::streaming
/// This is the second phase in canonical order.
/// 
/// # Arguments
/// * `input` - Streaming input parameters
/// 
/// # Returns
/// StreamingOutput with load/unload decisions
pub fn run_streaming(input: StreamingInput) -> StreamingOutput {
    let phase = StreamingPhase::new();
    let ctx = PhaseContext {
        tick: input.tick,
        delta_seconds: 1.0 / 20.0, // Fixed tick rate
        is_editor_mode: false,
    };
    phase.execute_with_input(&ctx, input)
}

impl StreamingPhase {
    fn execute_with_input(&self, _ctx: &PhaseContext, input: StreamingInput) -> StreamingOutput {
        let start = std::time::Instant::now();
        
        // ========================================================================
        // STREAMING LOGIC: World residency decisions
        // ========================================================================
        
        // Operational: Calculate view bounds from player position
        let mut chunks_to_load = Vec::new();
        let mut chunks_to_unload = Vec::new();
        
        if let Some(pos) = input.player_position {
            let view_radius = input.view_distance_chunks as i32;
            let center_x = (pos[0] / 64.0).floor() as i32; // CHUNK_SIZE = 64
            let center_z = (pos[2] / 64.0).floor() as i32;
            
            // Generate load candidates within view distance
            for dx in -view_radius..=view_radius {
                for dz in -view_radius..=view_radius {
                    let chunk_x = center_x + dx;
                    let chunk_z = center_z + dz;
                    let chunk_coord = [chunk_x, chunk_z];
                    
                    // Only load if not already resident
                    if !input.known_loaded_chunks.contains(&chunk_coord) {
                        chunks_to_load.push(chunk_coord);
                    }
                }
            }
            
            // Enforce residency budget
            let budget = input.residency_budget as usize;
            let budgeted_loads = chunks_to_load.len().min(budget);
            chunks_to_load.truncate(budgeted_loads);
            
            // Mark budget saturation if we hit the limit
            let budget_saturation = chunks_to_load.len() >= budget;
            
            // Process pending unload intents - unload chunks far from player
            let unload_count = input.pending_unload_count.min(10); // Cap per tick
            for loaded_chunk in &input.known_loaded_chunks {
                if chunks_to_unload.len() >= unload_count as usize {
                    break;
                }
                
                let chunk_x = loaded_chunk[0];
                let chunk_z = loaded_chunk[1];
                let dist_from_player = ((chunk_x - center_x).abs() + (chunk_z - center_z).abs()) as f32;
                
                // Unload if far from player (outside view distance + margin)
                if dist_from_player > (view_radius + 4) as f32 {
                    chunks_to_unload.push(*loaded_chunk);
                }
            }
            
            let load_decisions = chunks_to_load.len() as u32;
            let unload_decisions = chunks_to_unload.len() as u32;
            let duration_ms = start.elapsed().as_secs_f32() * 1000.0;
            
            StreamingOutput {
                chunks_to_load,
                chunks_to_unload,
                load_decisions,
                unload_decisions,
                budget_saturation,
                duration_ms,
            }
        } else {
            // No player position - no streaming decisions
            let duration_ms = start.elapsed().as_secs_f32() * 1000.0;
            
            StreamingOutput {
                chunks_to_load: Vec::new(),
                chunks_to_unload: Vec::new(),
                load_decisions: 0,
                unload_decisions: 0,
                budget_saturation: false,
                duration_ms,
            }
        }
    }
}

impl Default for StreamingPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseTrait for StreamingPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        // Default execution with empty input
        let input = StreamingInput {
            tick: ctx.tick,
            player_position: None,
            view_distance_chunks: 8,
            pending_unload_count: 0,
            residency_budget: 16,
            known_loaded_chunks: Vec::new(),
        };
        
        let output = self.execute_with_input(ctx, input);
        PhaseResult::ok(output.duration_ms)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Streaming
    }
}
