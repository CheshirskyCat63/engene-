//! Render phase - submit to GPU.
//!
//! OWNER: engine_render
//! Seventh (final) phase - runs last.
//! Render must NOT own simulation truth. It only consumes prepared state.

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};

pub struct RenderPhase;

impl RenderPhase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseTrait for RenderPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        // TODO: Implement render logic
        // - Extract renderable data from world state
        // - Submit to GPU
        // - Handle present/swap
        
        PhaseResult::ok(0.0)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Render
    }
    
    fn should_run(&self, ctx: &PhaseContext) -> bool {
        // Skip in headless mode
        !ctx.is_editor_mode || cfg!(feature = "render")
    }
}
