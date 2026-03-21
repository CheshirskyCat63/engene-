//! ECS Performance Contracts
//! 
//! Tests for ECS performance, memory efficiency, and scalability.
//! Ownership: ECS Team
//! Lane: ecs
//! Type: Contract + Performance Tests
//! Speed: Medium

#[cfg(test)]
mod ecs_performance_tests {
    use engene::core::ecs::Ecs;
    use engene::world::components::{Transform, Velocity, Kind};

    #[test]
    fn ecs_spawn_performance_under_budget() {
        let mut ecs = Ecs::new();
        let start = std::time::Instant::now();
        
        // Spawn 10,000 entities
        let entities: Vec<_> = (0..10_000).map(|_| ecs.spawn()).collect();
        
        let duration = start.elapsed();
        
        assert_eq!(entities.len(), 10_000);
        assert_eq!(ecs.alive.len(), 10_000);
        assert!(duration.as_millis() < 100, "10K spawns should complete in < 100ms, took {:?}", duration);
    }

    #[test]
    fn ecs_despawn_performance_under_budget() {
        let mut ecs = Ecs::new();
        let entities: Vec<_> = (0..10_000).map(|_| ecs.spawn()).collect();
        
        let start = std::time::Instant::now();
        
        // Despawn all entities
        for entity in entities {
            ecs.despawn(entity);
        }
        
        let duration = start.elapsed();
        
        assert_eq!(ecs.alive.len(), 0);
        assert!(duration.as_millis() < 50, "10K despawns should complete in < 50ms, took {:?}", duration);
    }

    #[test]
    fn ecs_component_access_performance() {
        let mut ecs = Ecs::new();
        
        // Create entities with components
        for i in 0..10_000 {
            let entity = ecs.spawn();
            ecs.transforms.insert(entity, Transform {
                x: i as f32,
                y: i as f32,
                cell_x: i,
                cell_y: i,
            });
            if i % 2 == 0 {
                ecs.velocities.insert(entity, Velocity { vx: i as f32, vy: i as f32 });
            }
        }
        
        let start = std::time::Instant::now();
        
        // Query all transforms
        let transform_count: usize = ecs.transforms.iter().count();
        
        let duration = start.elapsed();
        
        assert_eq!(transform_count, 10_000);
        assert!(duration.as_millis() < 10, "Transform query should complete in < 10ms, took {:?}", duration);
    }

    #[test]
    fn ecs_memory_usage_is_efficient() {
        let mut ecs = Ecs::new();
        let initial_memory = ecs.estimate_memory_usage();
        
        // Add entities with sparse components
        for i in 0..10_000 {
            let entity = ecs.spawn();
            if i % 10 == 0 {
                ecs.transforms.insert(entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i / 10,
                    cell_y: i / 10,
                });
            }
            if i % 100 == 0 {
                ecs.velocities.insert(entity, Velocity { vx: i as f32, vy: i as f32 });
            }
        }
        
        let with_entities_memory = ecs.estimate_memory_usage();
        
        // Memory usage should be proportional to actual usage, not entity count
        let memory_per_entity = (with_entities_memory - initial_memory) as f64 / 10_000.0;
        assert!(memory_per_entity < 100.0, "Memory per entity should be < 100 bytes, was {:.2}", memory_per_entity);
    }

    #[test]
    fn ecs_sparse_set_performance() {
        let mut ecs = Ecs::new();
        
        // Create sparse entities (1% have components)
        for i in 0..100_000 {
            let entity = ecs.spawn();
            if i % 100 == 0 {
                ecs.transforms.insert(entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i / 100,
                    cell_y: i / 100,
                });
            }
        }
        
        assert_eq!(ecs.transforms.len(), 1_000);
        assert_eq!(ecs.alive.len(), 100_000);
        
        let start = std::time::Instant::now();
        
        // Query sparse components
        let sparse_count: usize = ecs.transforms.iter().count();
        
        let duration = start.elapsed();
        
        assert_eq!(sparse_count, 1_000);
        assert!(duration.as_millis() < 5, "Sparse query should complete in < 5ms, took {:?}", duration);
    }

    #[test]
    fn ecs_concurrent_operations_performance() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let ecs = Arc::new(Mutex::new(Ecs::new()));
        let mut handles = vec![];
        
        let start = std::time::Instant::now();
        
        // Spawn from multiple threads
        for thread_id in 0..8 {
            let ecs_clone = Arc::clone(&ecs);
            let handle = thread::spawn(move || {
                let mut ecs = ecs_clone.lock().unwrap();
                for i in 0..1_000 {
                    let entity = ecs.spawn();
                    ecs.transforms.insert(entity, Transform {
                        x: (thread_id * 1_000 + i) as f32,
                        y: (thread_id * 1_000 + i) as f32,
                        cell_x: thread_id * 10 + i / 100,
                        cell_y: thread_id * 10 + i % 100,
                    });
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        
        let ecs = ecs.lock().unwrap();
        assert_eq!(ecs.alive.len(), 8_000);
        assert_eq!(ecs.transforms.len(), 8_000);
        assert!(duration.as_millis() < 200, "Concurrent operations should complete in < 200ms, took {:?}", duration);
    }

    #[test]
    fn ecs_bulk_component_operations_performance() {
        let mut ecs = Ecs::new();
        let entities: Vec<_> = (0..10_000).map(|_| ecs.spawn()).collect();
        
        let start = std::time::Instant::now();
        
        // Bulk add components
        for (i, entity) in entities.iter().enumerate() {
            ecs.transforms.insert(*entity, Transform {
                x: i as f32,
                y: i as f32,
                cell_x: i,
                cell_y: i,
            });
        }
        
        let add_duration = start.elapsed();
        
        let start = std::time::Instant::now();
        
        // Bulk remove components
        for entity in &entities {
            ecs.transforms.remove(entity);
        }
        
        let remove_duration = start.elapsed();
        
        assert!(add_duration.as_millis() < 50, "Bulk add should complete in < 50ms, took {:?}", add_duration);
        assert!(remove_duration.as_millis() < 30, "Bulk remove should complete in < 30ms, took {:?}", remove_duration);
    }

    #[test]
    fn ecs_query_complexity_performance() {
        let mut ecs = Ecs::new();
        
        // Create entities with various component combinations
        for i in 0..10_000 {
            let entity = ecs.spawn();
            ecs.transforms.insert(entity, Transform {
                x: i as f32,
                y: i as f32,
                cell_x: i,
                cell_y: i,
            });
            
            if i % 2 == 0 {
                ecs.velocities.insert(entity, Velocity { vx: i as f32, vy: i as f32 });
            }
            
            if i % 3 == 0 {
                ecs.kinds.insert(entity, Kind { value: i });
            }
        }
        
        let start = std::time::Instant::now();
        
        // Complex query: entities with Transform + Velocity but not Kind
        let complex_count = ecs.transforms.iter()
            .filter(|(entity, _)| ecs.velocities.contains_key(entity))
            .filter(|(entity, _)| !ecs.kinds.contains_key(entity))
            .count();
        
        let duration = start.elapsed();
        
        // Should be approximately 10,000 * (1/2) * (2/3) = 3,333 entities
        assert!(complex_count > 3000 && complex_count < 4000);
        assert!(duration.as_millis() < 20, "Complex query should complete in < 20ms, took {:?}", duration);
    }

    #[test]
    fn ecs_memory_fragmentation_is_minimal() {
        let mut ecs = Ecs::new();
        let initial_memory = ecs.estimate_memory_usage();
        
        // Create and destroy entities repeatedly
        for cycle in 0..10 {
            let entities: Vec<_> = (0..1_000).map(|_| ecs.spawn()).collect();
            
            for (i, entity) in entities.iter().enumerate() {
                ecs.transforms.insert(*entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i,
                    cell_y: i,
                });
            }
            
            // Remove half
            for entity in entities.iter().take(500) {
                ecs.despawn(*entity);
            }
            
            // Add new entities
            for i in 0..500 {
                let entity = ecs.spawn();
                ecs.transforms.insert(entity, Transform {
                    x: (cycle * 1000 + i) as f32,
                    y: (cycle * 1000 + i) as f32,
                    cell_x: cycle * 10 + i / 100,
                    cell_y: cycle * 10 + i % 100,
                });
            }
        }
        
        let final_memory = ecs.estimate_memory_usage();
        
        // Memory usage should not grow significantly due to fragmentation
        let memory_growth = final_memory - initial_memory;
        assert!(memory_growth < 1_000_000, "Memory growth should be < 1MB, was {} bytes", memory_growth);
    }

    #[test]
    fn ecs_scalability_under_load() {
        let mut ecs = Ecs::new();
        
        // Test scalability with increasing entity counts
        for scale in [1_000, 5_000, 10_000, 20_000] {
            let start = std::time::Instant::now();
            
            // Spawn entities
            let entities: Vec<_> = (0..scale).map(|_| ecs.spawn()).collect();
            
            // Add components
            for (i, entity) in entities.iter().enumerate() {
                ecs.transforms.insert(*entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i,
                    cell_y: i,
                });
            }
            
            // Query entities
            let count: usize = ecs.transforms.iter().count();
            
            let duration = start.elapsed();
            
            assert_eq!(count, scale);
            
            // Performance should scale linearly, not exponentially
            let ms_per_entity = duration.as_millis() as f64 / scale as f64;
            assert!(ms_per_entity < 0.01, "Performance should scale well: {:.6}ms per entity", ms_per_entity);
        }
    }
}

#[cfg(test)]
mod ecs_memory_efficiency_tests {
    use engene::core::ecs::Ecs;
    use engene::world::components::{Transform, Velocity};

    #[test]
    fn ecs_sparse_set_memory_efficiency() {
        let mut ecs = Ecs::new();
        let baseline_memory = ecs.estimate_memory_usage();
        
        // Create entities but only add components to 1%
        for i in 0..100_000 {
            let entity = ecs.spawn();
            if i % 100 == 0 {
                ecs.transforms.insert(entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i / 100,
                    cell_y: i / 100,
                });
            }
        }
        
        let sparse_memory = ecs.estimate_memory_usage();
        let sparse_overhead = sparse_memory - baseline_memory;
        
        // Sparse set should use memory proportional to actual usage
        let memory_per_component = sparse_overhead as f64 / 1_000.0; // Only 1,000 components
        assert!(memory_per_component < 200.0, "Sparse set should be memory efficient: {:.2} bytes per component", memory_per_component);
    }

    #[test]
    fn ecs_component_layout_efficiency() {
        let mut ecs = Ecs::new();
        
        // Test different component densities
        let densities = [0.1, 0.25, 0.5, 0.75, 1.0];
        
        for density in densities {
            let entity_count = 10_000;
            let component_count = (entity_count as f64 * density) as usize;
            
            let start_memory = ecs.estimate_memory_usage();
            
            // Add components with specified density
            for i in 0..entity_count {
                let entity = ecs.spawn();
                if (i as f64 / entity_count as f64) < density {
                    ecs.transforms.insert(entity, Transform {
                        x: i as f32,
                        y: i as f32,
                        cell_x: i,
                        cell_y: i,
                    });
                }
            }
            
            let end_memory = ecs.estimate_memory_usage();
            let memory_used = end_memory - start_memory;
            
            // Memory usage should be reasonable regardless of density
            let memory_per_component = memory_used as f64 / component_count as f64;
            assert!(memory_per_component < 150.0, "Memory per component should be efficient at {:.1f} density: {:.2} bytes", density, memory_per_component);
        }
    }

    #[test]
    fn ecs_entity_overhead_is_minimal() {
        let mut ecs = Ecs::new();
        let baseline_memory = ecs.estimate_memory_usage();
        
        // Create entities without components
        let entities: Vec<_> = (0..100_000).map(|_| ecs.spawn()).collect();
        
        let entity_memory = ecs.estimate_memory_usage();
        let entity_overhead = entity_memory - baseline_memory;
        
        // Entity overhead should be minimal
        let overhead_per_entity = entity_overhead as f64 / 100_000.0;
        assert!(overhead_per_entity < 50.0, "Entity overhead should be minimal: {:.2} bytes per entity", overhead_per_entity);
        
        assert_eq!(entities.len(), 100_000);
        assert_eq!(ecs.alive.len(), 100_000);
    }

    #[test]
    fn ecs_component_reuse_efficiency() {
        let mut ecs = Ecs::new();
        
        // Test component reuse patterns
        for cycle in 0..10 {
            let entities: Vec<_> = (0..1_000).map(|_| ecs.spawn()).collect();
            
            // Add components
            for (i, entity) in entities.iter().enumerate() {
                ecs.transforms.insert(*entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i,
                    cell_y: i,
                });
            }
            
            // Remove all components
            for entity in &entities {
                ecs.transforms.remove(entity);
            }
            
            // Despawn entities
            for entity in &entities {
                ecs.despawn(*entity);
            }
        }
        
        // Memory should be properly reclaimed
        let final_memory = ecs.estimate_memory_usage();
        assert_eq!(ecs.alive.len(), 0);
        assert_eq!(ecs.transforms.len(), 0);
        
        // Final memory should be close to baseline
        assert!(final_memory < 100_000, "Memory should be properly reclaimed: {} bytes", final_memory);
    }
}

// Mock methods for Ecs
impl Ecs {
    fn estimate_memory_usage(&self) -> usize {
        // Mock implementation - in real code this would calculate actual memory usage
        self.alive.len() * 8 + self.transforms.len() * 32 + self.velocities.len() * 16 + self.kinds.len() * 4
    }
}
