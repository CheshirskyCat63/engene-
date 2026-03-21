//! Tick phase - simulation truth changes first.
//!
//! OWNER: engine_runtime
//! This is the first phase in the canonical order.
//!
//! ## Purpose
//! Tick is the first phase - it runs simulation systems, updates entity positions,
//! processes physics, runs AI decisions. This is where world truth changes.
//!
//! ## What happens here
//! - Fixed tick simulation at 20Hz (50ms per tick)
//! - Entity position updates from velocity
//! - Physics stepping
//! - AI decision processing

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};

pub struct TickPhase;

impl TickPhase {
    pub fn new() -> Self {
        Self
    }
}

/// Execute tick phase - the first phase in canonical order.
/// 
/// OWNER: engine_runtime::phase::tick
/// This is the entrypoint for tick sequencing.
/// 
/// # Arguments
/// * `tick` - Current tick number
/// * `delta_seconds` - Time since last tick
/// 
/// # Returns
/// PhaseResult with elapsed time
pub fn run_tick(tick: u64, delta_seconds: f32) -> PhaseResult {
    let phase = TickPhase::new();
    let ctx = PhaseContext {
        tick,
        delta_seconds,
        is_editor_mode: false,
    };
    phase.execute(&ctx)
}

impl Default for TickPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseTrait for TickPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        // ========================================================================
        // TICK PHASE: First phase in canonical order
        // This is the operational tick entrypoint - replaces old orchestration
        // ========================================================================
        
        let tick_start = std::time::Instant::now();
        let tick = ctx.tick;
        let dt = ctx.delta_seconds;
        
        // Operational tick work:
        // - Track tick progression
        // - Emit timing metrics
        // - (Full ECS integration pending: velocity apply, physics step, AI decisions)
        
        // Log tick progression every 1000 ticks
        if tick % 1000 == 0 {
            tracing::debug!(
                target: "tick",
                tick = tick,
                dt = dt,
                "tick phase executed"
            );
        }
        
        let elapsed = tick_start.elapsed().as_secs_f32() * 1000.0;
        
        PhaseResult::ok(elapsed)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Tick
    }
}
