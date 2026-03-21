//! Enhanced Streaming Phase - Complete streaming owner integration.
//!
//! OWNER: engine_runtime / engine_world
//! This phase integrates the StreamingOwner with canonical phase execution.

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};
use engine_world::streaming_owner::{StreamingOwner, StreamingConfig, StreamingUpdateResult};

/// Enhanced streaming phase that uses StreamingOwner
pub struct StreamingEnhancedPhase {
    owner: StreamingOwner,
}

impl StreamingEnhancedPhase {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            owner: StreamingOwner::new(config),
        }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(StreamingConfig::default())
    }
}

impl PhaseTrait for StreamingEnhancedPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        let start = std::time::Instant::now();
        
        // Update streaming state with current tick and player position
        let result = self.owner.update(ctx.tick, ctx.player_position);
        
        // Convert StreamingUpdateResult to legacy StreamingOutput for compatibility
        let duration_ms = start.elapsed().as_secs_f32() * 1000.0;
        
        // Log streaming decisions for debugging
        if result.load_decisions > 0 || result.unload_decisions > 0 {
            println!(
                "[streaming] tick {}: load={}, unload={}, budget_sat={}, total_loaded={}",
                ctx.tick,
                result.load_decisions,
                result.unload_decisions,
                result.budget_saturation,
                result.total_loaded_chunks
            );
        }
        
        PhaseResult::ok(duration_ms)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Streaming
    }
    
    fn should_run(&self, ctx: &PhaseContext) -> bool {
        // Skip streaming in editor mode (no player position)
        !ctx.is_editor_mode
    }
}

/// Legacy compatibility wrapper for existing StreamingOutput
#[derive(Debug, Clone)]
pub struct StreamingOutput {
    pub chunks_to_load: Vec<[i32; 2]>,
    pub chunks_to_unload: Vec<[i32; 2]>,
    pub load_decisions: u32,
    pub unload_decisions: u32,
    pub budget_saturation: bool,
    pub duration_ms: f32,
}

impl From<StreamingUpdateResult> for StreamingOutput {
    fn from(result: StreamingUpdateResult) -> Self {
        // Convert ChunkCoord to legacy [i32; 2] format
        let chunks_to_load: Vec<[i32; 2]> = result.chunks_to_load
            .into_iter()
            .map(|coord| [coord.x, coord.z])
            .collect();
        
        let chunks_to_unload: Vec<[i32; 2]> = result.chunks_to_unload
            .into_iter()
            .map(|coord| [coord.x, coord.z])
            .collect();
        
        Self {
            chunks_to_load,
            chunks_to_unload,
            load_decisions: result.load_decisions,
            unload_decisions: result.unload_decisions,
            budget_saturation: result.budget_saturation,
            duration_ms: 0.0, // Will be set by phase execution
        }
    }
}

/// Enhanced streaming input that works with StreamingOwner
#[derive(Debug, Clone, Default)]
pub struct StreamingEnhancedInput {
    pub tick: u64,
    pub player_position: Option<[f32; 3]>,
    pub view_distance_chunks: u32,
    pub pending_unload_count: u32,
    pub residency_budget: u32,
    pub known_loaded_chunks: Vec<[i32; 2]>, // Legacy format
    pub streaming_config: StreamingConfig,
}

impl StreamingEnhancedInput {
    pub fn from_legacy(input: super::StreamingInput) -> Self {
        Self {
            tick: input.tick,
            player_position: input.player_position,
            view_distance_chunks: input.view_distance_chunks,
            pending_unload_count: input.pending_unload_count,
            residency_budget: input.residency_budget,
            known_loaded_chunks: input.known_loaded_chunks,
            streaming_config: StreamingConfig::default(),
        }
    }
}

/// Run enhanced streaming with StreamingOwner integration
pub fn run_streaming_enhanced(input: StreamingEnhancedInput) -> StreamingOutput {
    let phase = StreamingEnhancedPhase::new(input.streaming_config.clone());
    let ctx = PhaseContext {
        tick: input.tick,
        delta_seconds: 1.0 / 20.0, // Fixed tick rate
        is_editor_mode: false,
    };
    
    let result = phase.execute(&ctx);
    
    // Update duration in output
    let mut output = StreamingOutput::from(phase.owner.update(input.tick, input.player_position));
    output.duration_ms = result.duration_ms;
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_world::streaming_owner::ChunkCoord;
    
    #[test]
    fn test_enhanced_streaming_basic_functionality() {
        let input = StreamingEnhancedInput {
            tick: 1,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 4,
            pending_unload_count: 0,
            residency_budget: 8,
            known_loaded_chunks: vec![[0, 0], [1, 0]], // Already loaded chunks
            streaming_config: StreamingConfig::default(),
        };
        
        let output = run_streaming_enhanced(input);
        
        // Should make streaming decisions based on player position
        assert!(output.load_decisions > 0, "Should load new chunks");
        assert!(output.duration_ms > 0.0, "Should take some time");
    }
    
    #[test]
    fn test_enhanced_streaming_budget_enforcement() {
        let input = StreamingEnhancedInput {
            tick: 1,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 10, // Large view distance
            pending_unload_count: 0,
            residency_budget: 2, // Small budget
            known_loaded_chunks: vec![[0, 0], [1, 0]], // Already loaded
            streaming_config: StreamingConfig {
                max_loaded_chunks: 4,
                load_distance_chunks: 8,
                unload_distance_chunks: 12,
                async_load_batch_size: 10,
                priority_distance_weight: 1.0,
            },
        };
        
        let output = run_streaming_enhanced(input);
        
        // Should respect budget limit
        assert!(output.load_decisions <= 2, "Load decisions should respect budget");
        
        // Should trigger budget saturation
        assert!(output.budget_saturation, "Budget should be saturated");
    }
    
    #[test]
    fn test_enhanced_streaming_no_player() {
        let input = StreamingEnhancedInput {
            tick: 1,
            player_position: None, // No player
            view_distance_chunks: 8,
            pending_unload_count: 0,
            residency_budget: 16,
            known_loaded_chunks: vec![[0, 0], [1, 0], [2, 0]], // Many loaded chunks
            streaming_config: StreamingConfig::default(),
        };
        
        let output = run_streaming_enhanced(input);
        
        // Should unload chunks when no player position
        assert!(output.unload_decisions > 0, "Should unload chunks without player");
        assert!(!output.budget_saturation, "Budget should not be saturated when unloading");
    }
}
