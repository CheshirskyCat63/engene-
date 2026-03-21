pub mod simulation_core;
pub mod minimal_runtime_test;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";

    pub use crate::simulation_core;
}

// Re-export systems for transitional access from root
pub use simulation_core::systems::{EngineSystem, FixedTickContext, Scheduler, WorldTickSystem};

use engine_core::config::GameConfig;

// Minimal EngineRuntime for headless path
pub struct EngineRuntime {
    systems: Vec<Box<dyn EngineSystem>>,
}

impl EngineRuntime {
    pub fn new(_config: &GameConfig) -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    
    pub fn fixed_tick(&mut self, _ctx: &mut FixedTickContext) {
        // Execute all systems with fixed tick
        for system in &mut self.systems {
            system.fixed_tick(_ctx);
        }
    }
}
