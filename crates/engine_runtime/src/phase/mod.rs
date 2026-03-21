//! Phase execution contract for ENGENE runtime.
//!
//! This module defines the canonical phase order and contracts for frame execution.
//! Phases MUST execute in this order - no silent reordering allowed.
//!
//! ## Phase Order
//! 1. `tick` - Simulation truth changes first
//! 2. `streaming` - World residency decisions against updated state
//! 3. `persistence` - Loaded/unloaded transitions become durable
//! 4. `spatial` - Derived structure catches up to truth changes
//! 5. `audio` - Listener updates consume stable post-spatial state
//! 6. `editor_update` - Editor mutation against known post-sim frame state
//! 7. `render` - Consumes prepared state; does not define it
//!
//! ## Ownership
//! - This module is owned by `engine_runtime`
//! - Phase implementations may live in other crates
//! - This module defines the CONTRACT, not the implementation
//!
//! ## Law
//! - No silent phase reordering
//! - Render must NOT own simulation truth
//! - Audio must NOT depend on render success
//! - Spatial must be fed from explicit dirty causes

pub mod tick;
pub mod streaming;
pub mod persistence;
pub mod spatial;
pub mod audio;
pub mod editor;
pub mod render;

// Include streaming contract tests
#[cfg(test)]
mod streaming_test;

pub use tick::TickPhase;
pub use streaming::{StreamingPhase, run_streaming, StreamingInput, StreamingOutput};
pub use persistence::PersistencePhase;
pub use spatial::SpatialPhase;
pub use audio::AudioPhase;
pub use editor::EditorPhase;
pub use render::RenderPhase;

/// Canonical phase order - this is the law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Phase {
    Tick = 0,
    Streaming = 1,
    Persistence = 2,
    Spatial = 3,
    Audio = 4,
    EditorUpdate = 5,
    Render = 6,
}

impl Phase {
    /// Get all phases in canonical order.
    pub fn all() -> &'static [Phase] {
        &[
            Phase::Tick,
            Phase::Streaming,
            Phase::Persistence,
            Phase::Spatial,
            Phase::Audio,
            Phase::EditorUpdate,
            Phase::Render,
        ]
    }
    
    /// Get the next phase in order, or None if this is the last phase.
    pub fn next(self) -> Option<Phase> {
        match self {
            Phase::Tick => Some(Phase::Streaming),
            Phase::Streaming => Some(Phase::Persistence),
            Phase::Persistence => Some(Phase::Spatial),
            Phase::Spatial => Some(Phase::Audio),
            Phase::Audio => Some(Phase::EditorUpdate),
            Phase::EditorUpdate => Some(Phase::Render),
            Phase::Render => None,
        }
    }
    
    /// Get phase name as string.
    pub fn name(self) -> &'static str {
        match self {
            Phase::Tick => "tick",
            Phase::Streaming => "streaming",
            Phase::Persistence => "persistence",
            Phase::Spatial => "spatial",
            Phase::Audio => "audio",
            Phase::EditorUpdate => "editor_update",
            Phase::Render => "render",
        }
    }
}

/// Phase execution context passed to each phase.
pub struct PhaseContext {
    pub tick: u64,
    pub delta_seconds: f32,
    pub is_editor_mode: bool,
}

/// Result of phase execution.
pub struct PhaseResult {
    pub success: bool,
    pub duration_ms: f32,
    pub error_message: Option<String>,
}

impl PhaseResult {
    pub fn ok(duration_ms: f32) -> Self {
        Self {
            success: true,
            duration_ms,
            error_message: None,
        }
    }
    
    pub fn error(message: impl Into<String>, duration_ms: f32) -> Self {
        Self {
            success: false,
            duration_ms,
            error_message: Some(message.into()),
        }
    }
}

/// Trait that all phases must implement.
pub trait PhaseTrait: Send + Sync {
    /// Execute the phase.
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult;
    
    /// Get the phase type.
    fn phase_type(&self) -> Phase;
    
    /// Check if this phase should run in current configuration.
    fn should_run(&self, _ctx: &PhaseContext) -> bool {
        true
    }
}

/// Run all phases in canonical order.
pub fn run_all_phases<'a>(
    phases: &'a [&'a dyn PhaseTrait],
    ctx: &PhaseContext,
) -> Vec<PhaseResult> {
    let mut results = Vec::new();
    
    for phase in phases {
        let result = phase.execute(ctx);
        
        // Stop on first error (configurable - could continue)
        if !result.success {
            results.push(result);
            break;
        }
        
        results.push(result);
    }
    
    results
}

/// Validate phase order is correct.
/// Returns error if phases are not in canonical order.
pub fn validate_phase_order(phases: &[Phase]) -> Result<(), String> {
    let canonical = Phase::all();
    
    for (i, &phase) in phases.iter().enumerate() {
        if i >= canonical.len() {
            return Err(format!("Extra phase at index {}: {:?}", i, phase));
        }
        
        if phase != canonical[i] {
            return Err(format!(
                "Phase order violation at index {}. Expected {:?}, got {:?}",
                i, canonical[i], phase
            ));
        }
    }
    
    if phases.len() < canonical.len() {
        return Err(format!(
            "Missing phases. Have {}, expected all {}",
            phases.len(),
            canonical.len()
        ));
    }
    
    Ok(())
}
