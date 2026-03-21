pub mod simulation_core;
pub mod minimal_runtime_test;

pub mod api {
    /// Stable runtime crate identifier.
    pub const CRATE: &str = "engine_runtime";

    pub use crate::simulation_core;
}

// Re-export systems for transitional access from root
pub use simulation_core::systems::{EngineSystem, FixedTickContext, Scheduler, WorldTickSystem};

// Minimal config for engine path (temporary until proper config crate)
#[derive(Debug, Clone, Default)]
pub struct GameConfig {
    pub tick_rate: f32,
    pub max_entities: usize,
}

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
    
    pub fn add_system(&mut self, system: Box<dyn EngineSystem>) {
        self.systems.push(system);
    }
    
    pub fn due_ticks(&self) -> u32 {
        // For now, return 0 until we have proper scheduler integration
        0
    }
}
