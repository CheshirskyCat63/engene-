//! World Tick System - Runtime orchestration for world simulation.
//! Phase 3: Moved from root src/runtime/wiring/world_tick.rs
//!
//! TRANSITIONAL: Depends on root types (WorldGrid, ResourceGrid, etc.)
//! Removal condition: when world types move to engine_world

use crate::simulation_core::systems::engine_system::EngineSystem;
use crate::simulation_core::systems::scheduler::Scheduler;

/// World tick system - handles world simulation at fixed intervals.
pub struct WorldTickSystem {
    scheduler: Scheduler,
    interval: f32,
}

impl WorldTickSystem {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(5.0),
            interval: 5.0,
        }
    }

    pub fn new_with_interval(interval: f32) -> Self {
        Self {
            scheduler: Scheduler::new(interval),
            interval,
        }
    }

    /// Returns the interval in seconds.
    pub fn interval(&self) -> f32 {
        self.interval
    }

    /// Returns the scheduler for external inspection.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }
}

impl Default for WorldTickSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineSystem for WorldTickSystem {
    fn name(&self) -> &str {
        "WorldTick"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_tick_creation() {
        let system = WorldTickSystem::new();
        assert_eq!(system.name(), "WorldTick");
        assert_eq!(system.interval(), 5.0);
    }

    #[test]
    fn test_world_tick_with_interval() {
        let system = WorldTickSystem::new_with_interval(10.0);
        assert_eq!(system.interval(), 10.0);
    }
}
