pub mod phase;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";
}

// ================================================================================
// RUNTIME ASSEMBLY - Engine-owned runtime state
// ================================================================================
pub struct EngineEcs {
    pub tick: u32,
    pub alive: Vec<u32>,
}

pub struct EngineRuntimeAssembly {
    ecs: EngineEcs,
}

impl EngineRuntimeAssembly {
    pub fn kernel_headless() -> Self { 
        Self { 
            ecs: EngineEcs { tick: 0, alive: vec![] }
        } 
    }
    
    pub fn ecs(&mut self) -> &mut EngineEcs {
        &mut self.ecs
    }
}

// ================================================================================
// PHASE API - Canonical phase execution contract
// ================================================================================
pub use phase::{
    Phase, PhaseContext, PhaseResult, PhaseTrait,
    validate_phase_order, run_all_phases,
};
pub use phase::tick::run_tick;
pub use phase::streaming::{run_streaming, StreamingInput, StreamingOutput};
pub use phase::persistence::{run_persistence, PersistenceInput, PersistenceOutput};
