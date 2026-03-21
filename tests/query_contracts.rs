//! Query Contracts
//! 
//! Tests for query performance, memory usage, and composition.
//! Ownership: ECS Team
//! Lane: ecs
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod query_performance_tests {
    use engene::core::query::Query;
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity, Position};

    #[test]
    fn query_performance_is_optimal() {
        let mut world = World::new();
        
        // Create many entities with components
        for i in 0..1000 {
            let entity = world.spawn();
            world.add_component(entity, Transform::default());
            world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
            if i % 2 == 0 {
                world.add_component(entity, Position::new(i as f32, i as f32, 0.0));
            }
        }
        
        let start = std::time::Instant::now();
        
        // Query should be fast
        let query = Query::<(&Transform, &Velocity)>::new();
        let results: Vec<_> = query.iter(&world).collect();
        
        let duration = start.elapsed();
        
        assert_eq!(results.len(), 1000);
        assert!(duration.as_millis() < 10, "Query should complete in < 10ms, took {:?}", duration);
    }

    #[test]
    fn query_memory_usage_is_efficient() {
        let mut world = World::new();
        
        // Create entities
        for i in 0..100 {
            let entity = world.spawn();
            world.add_component(entity, Transform::default());
            world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
        }
        
        let query = Query::<(&Transform, &Velocity)>::new();
        
        // Query should not significantly increase memory usage
        let memory_before = world.memory_usage();
        let results: Vec<_> = query.iter(&world).collect();
        let memory_after = world.memory_usage();
        
        assert_eq!(results.len(), 100);
        assert!(memory_after - memory_before < 1024, "Query should use minimal additional memory");
    }

    #[test]
    fn query_results_are_consistent() {
        let mut world = World::new();
        
        // Create entities with known values
        for i in 0..10 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            world.add_component(entity, Velocity::new(i as f32 * 2.0, 0.0, 0.0));
        }
        
        let query = Query::<(&Transform, &Velocity)>::new();
        let results: Vec<_> = query.iter(&world).collect();
        
        // Results should be consistent and ordered
        assert_eq!(results.len(), 10);
        
        for (i, (transform, velocity)) in results.iter().enumerate() {
            assert_eq!(transform.position.x, i as f32);
            assert_eq!(velocity.x, i as f32 * 2.0);
        }
    }
}

#[cfg(test)]
mod query_caching_tests {
    use engene::core::query::{Query, QueryCache};
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity};

    #[test]
    fn query_caching_is_effective() {
        let mut world = World::new();
        let mut cache = QueryCache::new();
        
        // Create entities
        for i in 0..100 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
        }
        
        let query = Query::<(&Transform, &Velocity)>::new();
        
        // First query should populate cache
        let start1 = std::time::Instant::now();
        let results1: Vec<_> = query.iter_cached(&mut cache, &world).collect();
        let duration1 = start1.elapsed();
        
        // Second query should use cache
        let start2 = std::time::Instant::now();
        let results2: Vec<_> = query.iter_cached(&mut cache, &world).collect();
        let duration2 = start2.elapsed();
        
        assert_eq!(results1.len(), 100);
        assert_eq!(results2.len(), 100);
        assert!(duration2 < duration1, "Cached query should be faster");
    }
}

#[cfg(test)]
mod query_parallel_tests {
    use engene::core::query::Query;
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity};
    use std::sync::{Arc, Mutex};

    #[test]
    fn query_parallel_execution_is_safe() {
        let mut world = World::new();
        
        // Create entities
        for i in 0..100 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
        }
        
        let world = Arc::new(Mutex::new(world));
        let query = Arc::new(Query::<(&Transform, &Velocity)>::new());
        
        let mut handles = vec![];
        
        // Run queries in parallel
        for thread_id in 0..4 {
            let world_clone = Arc::clone(&world);
            let query_clone = Arc::clone(&query);
            
            let handle = std::thread::spawn(move || {
                let world = world_clone.lock().unwrap();
                let results: Vec<_> = query_clone.iter(&*world).collect();
                
                // Each thread should get all results
                assert_eq!(results.len(), 100);
                
                // Verify thread-specific processing
                for (transform, velocity) in &results {
                    assert_eq!(transform.position.x, velocity.x);
                }
                
                results.len()
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            let result = handle.join().unwrap();
            assert_eq!(result, 100);
        }
    }
}

#[cfg(test)]
mod query_filtering_tests {
    use engene::core::query::{Query, Filter};
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity, Position};

    #[test]
    fn query_filtering_is_accurate() {
        let mut world = World::new();
        
        // Create entities with different components
        for i in 0..20 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            
            if i % 2 == 0 {
                world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
            }
            
            if i % 3 == 0 {
                world.add_component(entity, Position::new(i as f32, i as f32, 0.0));
            }
        }
        
        // Query entities with Transform and Velocity but not Position
        let filter = Filter::new()
            .with_component::<Transform>()
            .with_component::<Velocity>()
            .without_component::<Position>();
        
        let query = Query::<(&Transform, &Velocity)>::new().with_filter(filter);
        let results: Vec<_> = query.iter(&world).collect();
        
        // Should get entities with Transform + Velocity but not Position
        // Numbers: 2, 4, 8, 10, 14, 16 (even numbers not divisible by 3)
        assert_eq!(results.len(), 6);
        
        for (transform, velocity) in &results {
            assert_eq!(transform.position.x, velocity.x);
        }
    }
}

#[cfg(test)]
mod query_sorting_tests {
    use engene::core::query::{Query, SortBy};
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity};

    #[test]
    fn query_sorting_is_stable() {
        let mut world = World::new();
        
        // Create entities with specific positions
        let positions = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        
        for pos in &positions {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(*pos, 0.0, 0.0));
            world.add_component(entity, Velocity::new(*pos * 2.0, 0.0, 0.0));
        }
        
        // Sort by Transform position
        let query = Query::<(&Transform, &Velocity)>::new()
            .sort_by(SortBy::component::<Transform>(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap()));
        
        let results: Vec<_> = query.iter(&world).collect();
        
        assert_eq!(results.len(), 6);
        
        // Should be sorted by position: 1.0, 2.0, 3.0, 5.0, 8.0, 9.0
        let expected_positions = vec![1.0, 2.0, 3.0, 5.0, 8.0, 9.0];
        
        for (i, (transform, velocity)) in results.iter().enumerate() {
            assert_eq!(transform.position.x, expected_positions[i]);
            assert_eq!(velocity.x, expected_positions[i] * 2.0);
        }
    }
}

#[cfg(test)]
mod query_aggregation_tests {
    use engene::core::query::{Query, Aggregator};
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity};

    #[test]
    fn query_aggregation_is_correct() {
        let mut world = World::new();
        
        // Create entities with known values
        for i in 0..10 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            world.add_component(entity, Velocity::new(i as f32 * 2.0, 0.0, 0.0));
        }
        
        let query = Query::<(&Transform, &Velocity)>::new();
        
        // Aggregate positions
        let position_sum = query.aggregate(&world, Aggregator::sum(|(transform, _)| transform.position.x));
        let velocity_sum = query.aggregate(&world, Aggregator::sum(|(_, velocity)| velocity.x));
        
        assert_eq!(position_sum, 45.0); // Sum of 0..9
        assert_eq!(velocity_sum, 90.0); // Sum of 0..18 (2x)
    }
}

#[cfg(test)]
mod query_composition_tests {
    use engene::core::query::{Query, Filter};
    use engene::core::world::World;
    use engene::core::components::{Transform, Velocity, Position, Health};

    #[test]
    fn query_composition_is_flexible() {
        let mut world = World::new();
        
        // Create entities with various component combinations
        for i in 0..30 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new(i as f32, 0.0, 0.0));
            
            if i % 2 == 0 {
                world.add_component(entity, Velocity::new(i as f32, 0.0, 0.0));
            }
            
            if i % 3 == 0 {
                world.add_component(entity, Position::new(i as f32, i as f32, 0.0));
            }
            
            if i % 5 == 0 {
                world.add_component(entity, Health::new(100.0));
            }
        }
        
        // Complex query: Transform + Velocity, no Position, optional Health
        let filter = Filter::new()
            .with_component::<Transform>()
            .with_component::<Velocity>()
            .without_component::<Position>();
        
        let query = Query::<(&Transform, &Velocity, Option<&Health>)>::new().with_filter(filter);
        let results: Vec<_> = query.iter(&world).collect();
        
        // Count matches manually: even numbers not divisible by 3
        // 2, 4, 8, 10, 14, 16, 20, 22, 26, 28
        assert_eq!(results.len(), 10);
        
        // Check that some have Health (multiples of 10)
        let with_health: Vec<_> = results.iter().filter(|(_, _, health)| health.is_some()).collect();
        assert_eq!(with_health.len(), 3); // 10, 20, 30 (but 30 is excluded by filter)
    }
}


use engene::world::components::Transform;

#[derive(Clone)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

impl Velocity {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone)]
struct Health {
    value: f32,
}

impl Health {
    fn new(value: f32) -> Self {
        Self { value }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Vec3 {
    const NEG_Y: Self = Self::new(0.0, -1.0, 0.0);
}
