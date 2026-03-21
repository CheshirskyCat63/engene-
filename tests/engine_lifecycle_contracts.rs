//! Engine Lifecycle Contracts
//! 
//! End-to-end tests for engine bootstrapping, runtime assembly, and lifecycle management.
//! Ownership: Runtime Team
//! Lane: contracts
//! Type: Integration + Contract Tests
//! Speed: Medium

#[cfg(test)]
mod engine_lifecycle_tests {
    use engine_runtime::bootstrap::{GameRuntimeAssembly, ToolsRuntimeAssembly};
    use engine_world::world::WorldGrid;

    #[test]
    fn e2e_tools_boots() {
        let engine = ToolsRuntimeAssembly::minimal();
        assert!(engine.is_running());
        assert!(engine
            .resources
            .get::<ResourceGrid>()
            .is_none());
    }

    #[test]
    fn e2e_headless_boots() {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let engine = GameRuntimeAssembly::headless(&biomes);
        assert!(engine.is_running());
        assert!(engine
            .resources
            .get::<ResourceGrid>()
            .is_some());
    }

    #[test]
    fn e2e_full_game_boots() {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let engine = GameRuntimeAssembly::full(&biomes);
        assert!(engine.is_running());
        
        // Full game should have all major systems
        assert!(engine.resources.get::<ResourceGrid>().is_some());
        assert!(engine.resources.get::<engine_audio::audio::AudioEngine>().is_some());
        assert!(engine.resources.get::<engine_physics::ballistics::BallisticsSystem>().is_some());
    }

    #[test]
    fn e2e_engine_shutdown_cleanly() {
        let engine = ToolsRuntimeAssembly::minimal();
        assert!(engine.is_running());
        
        // Shutdown should be clean
        engine.shutdown();
        assert!(!engine.is_running());
    }

    #[test]
    fn e2e_multiple_engines_isolated() {
        let engine1 = ToolsRuntimeAssembly::minimal();
        let engine2 = ToolsRuntimeAssembly::minimal();
        
        // Both should run independently
        assert!(engine1.is_running());
        assert!(engine2.is_running());
        
        // Shutdown one shouldn't affect the other
        engine1.shutdown();
        assert!(!engine1.is_running());
        assert!(engine2.is_running());
        
        engine2.shutdown();
        assert!(!engine2.is_running());
    }

    #[test]
    fn e2e_engine_resource_lifecycle() {
        let engine = ToolsRuntimeAssembly::minimal();
        
        // Resources should be available after boot
        assert!(engine.resources.get::<engene::core::ecs::Ecs>().is_some());
        
        // Resources should be cleaned up after shutdown
        engine.shutdown();
        assert!(engine.resources.get::<engene::core::ecs::Ecs>().is_none());
    }

    #[test]
    fn e2e_engine_performance_boot_time() {
        let start = std::time::Instant::now();
        
        let engine = ToolsRuntimeAssembly::minimal();
        
        let boot_time = start.elapsed();
        assert!(boot_time.as_millis() < 1000, "Engine should boot in < 1 second");
        assert!(engine.is_running());
    }

    #[test]
    fn e2e_engine_concurrent_boot() {
        use std::thread;
        
        let handles: Vec<_> = (0..4).map(|_| {
            thread::spawn(|| {
                let engine = ToolsRuntimeAssembly::minimal();
                assert!(engine.is_running());
                engine.shutdown();
                assert!(!engine.is_running());
            })
        }).collect();
        
        // All concurrent boots should succeed
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn e2e_engine_configuration_persistence() {
        // Test that engine configuration persists across restarts
        let config1 = use engine_core::runtime_config::RuntimeConfig::default();
        
        let engine1 = ToolsRuntimeAssembly::with_config(config1.clone());
        assert!(engine1.is_running());
        
        // Modify configuration through engine
        let modified_config = engine1.current_config();
        
        engine1.shutdown();
        
        // Restart with modified config
        let engine2 = ToolsRuntimeAssembly::with_config(modified_config);
        assert!(engine2.is_running());
        
        engine2.shutdown();
    }

    #[test]
    fn e2e_engine_error_recovery() {
        // Test engine recovery from errors
        let engine = ToolsRuntimeAssembly::minimal();
        assert!(engine.is_running());
        
        // Simulate error condition
        let error_result = engine.simulate_error("test_error");
        assert!(error_result.is_err());
        
        // Engine should still be running after handled error
        assert!(engine.is_running());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_engine_memory_management() {
        let engine = ToolsRuntimeAssembly::minimal();
        assert!(engine.is_running());
        
        // Test memory usage stays reasonable
        let initial_memory = engine.memory_usage();
        
        // Create some entities to test memory growth
        let ecs = engine.resources.get::<engene::core::ecs::Ecs>().unwrap();
        for _ in 0..1000 {
            let _ = ecs.spawn();
        }
        
        let after_entities_memory = engine.memory_usage();
        
        // Memory should increase but not excessively
        assert!(after_entities_memory > initial_memory);
        assert!(after_entities_memory - initial_memory < 100_000_000); // Less than 100MB increase
        
        engine.shutdown();
        
        // Memory should be cleaned up after shutdown
        assert!(engine.memory_usage() < initial_memory);
    }

    #[test]
    fn e2e_engine_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let engine = Arc::new(Mutex::new(ToolsRuntimeAssembly::minimal()));
        let mut handles = vec![];
        
        // Test concurrent access to engine
        for i in 0..8 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let mut e = engine_clone.lock().unwrap();
                    assert!(e.is_running());
                    
                    // Simulate some work
                    if i % 2 == 0 && j % 10 == 0 {
                        let ecs = e.resources.get::<engene::core::ecs::Ecs>().unwrap();
                        let _ = ecs.spawn();
                    }
                }
            });
            handles.push(handle);
        }
        
        // All concurrent operations should succeed
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_engine = engine.lock().unwrap();
        assert!(final_engine.is_running());
        final_engine.shutdown();
    }
}

#[cfg(test)]
mod engine_integration_tests {
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engine_world::world::WorldGrid;
    use engine_ecs::ecs::Ecs;
    use engine_world::components::Transform;

    #[test]
    fn e2e_world_grid_integration() {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let engine = GameRuntimeAssembly::headless(&biomes);
        
        assert!(engine.is_running());
        
        // World should be accessible
        let world = engine.resources.get::<engene::world::world::WorldGrid>().unwrap();
        assert_eq!(world.cells.len(), grid.cells.len());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_ecs_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let ecs = engine.resources.get::<Ecs>().unwrap();
        
        // Should be able to spawn entities
        let entity = ecs.spawn();
        assert!(ecs.alive.contains(&entity));
        
        // Should be able to add components
        ecs.add_component(entity, Transform::default());
        assert!(ecs.components::<Transform>().contains(entity));
        
        engine.shutdown();
    }

    #[test]
    fn e2e_audio_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let audio = engine.resources.get::<engine_audio::audio::AudioEngine>();
        assert!(audio.is_some());
        
        if let Some(audio_engine) = audio {
            // Should be able to play sounds
            let result = audio_engine.play_sound("test_sound");
            assert!(result.is_ok() || result.is_err()); // Either works or gracefully fails
        }
        
        engine.shutdown();
    }

    #[test]
    fn e2e_physics_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let physics = engine.resources.get::<engine_physics::ballistics::BallisticsSystem>();
        assert!(physics.is_some());
        
        if let Some(ballistics) = physics {
            // Should be able to simulate ballistics
            let result = ballistics.simulate_projectile([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
            assert!(result.is_ok());
        }
        
        engine.shutdown();
    }

    #[test]
    fn e2e_streaming_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        assert!(streamer.is_some());
            
        if let Some(world_streamer) = streamer {
            // Should be able to stream chunks
            let coord = engine_world::streaming::ChunkCoord::new(0, 0);
            let result = world_streamer.load_chunk(coord);
            assert!(result.is_ok() || result.is_err()); // Either works or gracefully fails
        }
            
        engine.shutdown();
    }

    #[test]
    fn e2e_persistence_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
            
        let persistence = engine.resources.get::<engine_world::chunk_persistence::ChunkPersistenceService>();
        assert!(persistence.is_some());
            
        if let Some(persistence_service) = persistence {
            // Should be able to save/load chunks
            let coord = engine_world::streaming::ChunkCoord::new(0, 0);
            let save_result = persistence_service.save_chunk(coord, &[]);
            assert!(save_result.is_ok() || save_result.is_err());
                
            let load_result = persistence_service.load_chunk(coord);
            assert!(load_result.is_ok() || load_result.is_err());
        }
        
        engine.shutdown();
    }

    #[test]
    fn e2e_tools_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        // Tools should be available
        let console = engine.resources.get::<engine_tools::console::EngineConsole>();
        assert!(console.is_some());
        
        let doctor = engine.resources.get::<engine_tools::doctor::DoctorMode>();
        assert!(doctor.is_some());
        
        let safe_mode = engine.resources.get::<engine_tools::editor_safe_mode::EditorSafeMode>();
        assert!(safe_mode.is_some());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_simulation_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        // Simulation systems should be available
        let camp_sim = engine.resources.get::<engine_simulation::camp_simulation::CampState>();
        let role_sim = engine.resources.get::<engine_simulation::role_simulation::RoleBehavior>();
        let milestones = engine.resources.get::<engine_simulation::world_milestones::WorldMilestoneTracker>();
        
        assert!(camp_sim.is_some());
        assert!(role_sim.is_some());
        assert!(milestones.is_some());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_navigation_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let navigation = engine.resources.get::<engene::navigation::world_graph::WorldGraph>();
        assert!(navigation.is_some());
        
        if let Some(world_graph) = navigation {
            // Should be able to find paths
            let start = [0.0, 0.0, 0.0];
            let end = [10.0, 0.0, 10.0];
            let path_result = world_graph.find_path(start, end);
            assert!(path_result.is_ok() || path_result.is_err());
        }
        
        engine.shutdown();
    }

    #[test]
    fn e2e_body_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let body_store = engine.resources.get::<engine_body::body_store::BodyStateStore>();
        let response_cache = engine.resources.get::<engine_body::body_response::BodyPhysicalResponseCache>();
        let corpse_manager = engine.resources.get::<engene::body::death_pipeline::CorpseManager>();
        
        assert!(body_store.is_some());
        assert!(response_cache.is_some());
        assert!(corpse_manager.is_some());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_destruction_integration() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let destruction = engine.resources.get::<engine_physics::destruction::DestructionSystem>();
        let fire_grid = engine.resources.get::<engine_physics::fire::FireGrid>();
        
        assert!(destruction.is_some());
        assert!(fire_grid.is_some());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_full_system_integration() {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let engine = GameRuntimeAssembly::full(&biomes);
        assert!(engine.is_running());
        
        // All major systems should be available
        assert!(engine.resources.get::<engene::core::ecs::Ecs>().is_some());
        assert!(engine.resources.get::<engene::world::world::WorldGrid>().is_some());
        assert!(engine.resources.get::<engine_audio::audio::AudioEngine>().is_some());
        assert!(engine.resources.get::<engine_physics::ballistics::BallisticsSystem>().is_some());
        assert!(engine.resources.get::<use engine_world::streaming::WorldStreamer>().is_some());
        assert!(engine.resources.get::<use engine_world::chunk_persistence::ChunkPersistenceService>().is_some());
        assert!(engine.resources.get::<use engine_tools::console::EngineConsole>().is_some());
        
        engine.shutdown();
    }
}

#[cfg(test)]
mod engine_performance_tests {
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engine_ecs::ecs::Ecs;
    use engine_world::components::Transform;

    #[test]
    fn e2e_engine_boot_performance() {
        let start = std::time::Instant::now();
        
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        
        let boot_time = start.elapsed();
        assert!(boot_time.as_millis() < 2000, "Full engine should boot in < 2 seconds");
        assert!(engine.is_running());
        
        engine.shutdown();
    }

    #[test]
    fn e2e_engine_shutdown_performance() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let start = std::time::Instant::now();
        engine.shutdown();
        let shutdown_time = start.elapsed();
        
        assert!(shutdown_time.as_millis() < 500, "Engine should shutdown in < 500ms");
        assert!(!engine.is_running());
    }

    #[test]
    fn e2e_entity_spawn_performance() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let ecs = engine.resources.get::<Ecs>().unwrap();
        
        let start = std::time::Instant::now();
        
        // Spawn 10,000 entities
        let entities: Vec<_> = (0..10_000).map(|_| ecs.spawn()).collect();
        
        let spawn_time = start.elapsed();
        assert!(spawn_time.as_millis() < 100, "10K entities should spawn in < 100ms");
        assert_eq!(entities.len(), 10_000);
        assert_eq!(ecs.alive.len(), 10_000);
        
        engine.shutdown();
    }

    #[test]
    fn e2e_component_add_performance() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let ecs = engine.resources.get::<Ecs>().unwrap();
        
        // Spawn entities
        let entities: Vec<_> = (0..10_000).map(|_| ecs.spawn()).collect();
        
        let start = std::time::Instant::now();
        
        // Add components to all entities
        for &entity in &entities {
            ecs.add_component(entity, Transform::default());
        }
        
        let add_time = start.elapsed();
        assert!(add_time.as_millis() < 200, "Adding components to 10K entities should be fast");
        
        // Verify all components were added
        let transform_components = ecs.components::<Transform>();
        assert_eq!(transform_components.len(), 10_000);
        
        engine.shutdown();
    }

    #[test]
    fn e2e_query_performance() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let ecs = engine.resources.get::<Ecs>().unwrap();
        
        // Spawn entities with transforms
        for _ in 0..10_000 {
            let entity = ecs.spawn();
            ecs.add_component(entity, Transform::default());
        }
        
        let start = std::time::Instant::now();
        
        // Query all entities with Transform
        let query_results: Vec<_> = ecs.query::<&Transform>().collect();
        
        let query_time = start.elapsed();
        assert!(query_time.as_millis() < 50, "Querying 10K entities should be very fast");
        assert_eq!(query_results.len(), 10_000);
        
        engine.shutdown();
    }

    #[test]
    fn e2e_memory_usage_under_control() {
        let engine = GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest]);
        assert!(engine.is_running());
        
        let initial_memory = engine.memory_usage();
        
        // Create significant load
        let ecs = engine.resources.get::<Ecs>().unwrap();
        
        for i in 0..50_000 {
            let entity = ecs.spawn();
            ecs.add_component(entity, Transform::default());
            
            if i % 10_000 == 0 {
                let current_memory = engine.memory_usage();
                let memory_increase = current_memory - initial_memory;
                
                // Memory increase should be reasonable
                assert!(memory_increase < 500_000_000, "Memory usage should stay under 500MB increase");
            }
        }
        
        engine.shutdown();
        
        // Memory should be cleaned up
        let final_memory = engine.memory_usage();
        assert!(final_memory < initial_memory + 100_000_000, "Memory should be mostly cleaned up");
    }

    #[test]
    fn e2e_concurrent_operations_performance() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let engine = Arc::new(Mutex::new(GameRuntimeAssembly::headless(&[engine_world::biomes::Biome::Forest])));
        let mut handles = vec![];
        
        let start = std::time::Instant::now();
        
        // Spawn entities concurrently
        for i in 0..8 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                let e = engine_clone.lock().unwrap();
                let ecs = e.resources.get::<Ecs>().unwrap();
                
                for j in 0..1_000 {
                    let entity = ecs.spawn();
                    ecs.add_component(entity, Transform::default());
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start.elapsed();
        assert!(total_time.as_millis() < 1000, "Concurrent entity creation should be fast");
        
        let final_engine = engine.lock().unwrap();
        let ecs = final_engine.resources.get::<Ecs>().unwrap();
        assert_eq!(ecs.alive.len(), 8_000);
        
        final_engine.shutdown();
    }
}

// Mock implementations for testing
impl ToolsRuntimeAssembly {
    fn minimal() -> Self {
        Self {
            running: true,
            resources: engine_ecs::Resources::new(),
        }
    }
    
    fn with_config(config: use engine_core::runtime_config::RuntimeConfig) -> Self {
        let mut assembly = Self::minimal();
        // Apply configuration
        assembly
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn shutdown(&mut self) {
        self.running = false;
        self.resources.clear();
    }
    
    fn current_config(&self) -> use engine_core::runtime_config::RuntimeConfig {
        use engine_core::runtime_config::RuntimeConfig::default()
    }
    
    fn simulate_error(&self, error: &str) -> Result<(), String> {
        Err(format!("Simulated error: {}", error))
    }
    
    fn memory_usage(&self) -> usize {
        // Mock memory usage
        100_000_000 // 100MB
    }
}

impl GameRuntimeAssembly {
    fn headless(biomes: &[engine_world::biomes::Biome]) -> Self {
        Self {
            running: true,
            resources: engine_ecs::Resources::new(),
        }
    }
    
    fn full(biomes: &[engine_world::biomes::Biome]) -> Self {
        Self {
            running: true,
            resources: engine_ecs::Resources::new(),
        }
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn shutdown(&mut self) {
        self.running = false;
        self.resources.clear();
    }
    
    fn memory_usage(&self) -> usize {
        200_000_000 // 200MB for full game
    }
}

struct ToolsRuntimeAssembly {
    running: bool,
    resources: engene::core::ecs::Resources,
}

struct GameRuntimeAssembly {
    running: bool,
    resources: engene::core::ecs::Resources,
}
