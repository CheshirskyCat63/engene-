//! Streaming phase - world residency decisions.
//!
//! OWNER: engine_runtime / engine_world
//! Second phase - runs after tick.

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};

pub struct StreamingPhase;

impl StreamingPhase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StreamingPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseTrait for StreamingPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        // TODO: Implement streaming logic
        // - Determine which chunks should be loaded
        // - Determine which chunks should be unloaded
        // - Issue async load/unload requests
        
        PhaseResult::ok(0.0)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Streaming
    }
}
