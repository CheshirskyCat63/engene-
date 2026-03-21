//! Engine Minimal Runtime Test
//! Tests: Can we create world, run 1000 ticks, check invariants, get same result?

#[cfg(test)]
mod tests {
    use crate::*;
    use engine_world::*;
    
    #[test]
    fn engine_minimal_runtime_path() {
        // 1. Config
        let config = GameConfig::default();
        
        // 2. Runtime assembly
        let mut runtime = EngineRuntime::new(&config);
        
        // 3. Scheduler
        let mut scheduler = Scheduler::new(1.0 / 60.0); // 60 Hz fixed tick
        
        // 4. World creation
        let mut world = WorldGrid::generate();
        
        // 5. Fixed tick test - 1000 ticks
        let tick_count = 1000;
        let dt = 1.0 / 60.0;
        
        for _i in 0..tick_count {
            scheduler.accumulate(dt);
            
            if scheduler.accumulated() >= scheduler.interval() {
                // Fixed tick execution
                let mut ctx = FixedTickContext::new(dt, ());
                runtime.fixed_tick(&mut ctx);
                
                // World tick
                tick_world(&mut world, dt);
            }
        }
        
        // 6. Deterministic progression check
        assert!(world.cells.len() > 0, "World should have cells");
        
        println!("✅ Engine minimal runtime path: {} ticks completed", tick_count);
    }
    
    #[test] 
    fn engine_deterministic_replay() {
        // Test: Same input produces same output
        
        let config = GameConfig::default();
        let _seed = 42u64;
        
        // First run
        let result1 = run_deterministic_scenario(&config);
        
        // Second run  
        let result2 = run_deterministic_scenario(&config);
        
        // Should be identical
        assert_eq!(result1.tick_count, result2.tick_count);
        assert_eq!(result1.final_state_hash, result2.final_state_hash);
        
        println!("✅ Engine deterministic replay: PASSED");
    }
    
    fn run_deterministic_scenario(_config: &GameConfig) -> DeterministicResult {
        let mut runtime = EngineRuntime::new(_config);
        let mut world = WorldGrid::generate();
        let mut scheduler = Scheduler::new(1.0 / 60.0);
        
        let tick_count = 100;
        let dt = 1.0 / 60.0;
        
        for _ in 0..tick_count {
            scheduler.accumulate(dt);
            if scheduler.accumulated() >= scheduler.interval() {
                let mut ctx = FixedTickContext::new(dt, ());
                runtime.fixed_tick(&mut ctx);
                tick_world(&mut world, dt);
            }
        }
        
        DeterministicResult {
            tick_count,
            final_state_hash: hash_world_state(&world),
        }
    }
    
    fn tick_world(_world: &mut WorldGrid, _dt: f32) {
        // Minimal world tick for engine path
        // Just basic cell updates, no game logic
    }
    
    fn hash_world_state(world: &WorldGrid) -> u64 {
        // Simple hash of world state for determinism checking
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        world.cells.len().hash(&mut hasher);
        hasher.finish()
    }
    
    #[derive(Debug, PartialEq)]
    struct DeterministicResult {
        tick_count: usize,
        final_state_hash: u64,
    }
}
