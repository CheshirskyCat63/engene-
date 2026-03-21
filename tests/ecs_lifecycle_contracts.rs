//! ECS Lifecycle Contracts
//! 
//! Tests for entity lifecycle, spawn/despawn operations, and component management.
//! Ownership: ECS Team
//! Lane: ecs
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod ecs_lifecycle_tests {
    use engene::core::ecs::Ecs;
    use engene::world::components::Transform;

    #[test]
    fn ecs_create_empty() {
        let ecs = Ecs::new();
        assert_eq!(ecs.alive.len(), 0);
        assert!(ecs.transforms.is_empty());
        assert!(ecs.kinds.is_empty());
    }

    #[test]
    fn ecs_spawn_creates_entity() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        assert!(ecs.is_alive(entity));
        assert_eq!(ecs.alive.len(), 1);
        assert!(ecs.alive.contains(&entity));
    }

    #[test]
    fn ecs_spawn_increments_id() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        let e3 = ecs.spawn();
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);
    }

    #[test]
    fn ecs_despawn_removes_entity() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        assert!(ecs.is_alive(entity));
        ecs.despawn(entity);
        assert!(!ecs.is_alive(entity));
        assert_eq!(ecs.alive.len(), 0);
    }

    #[test]
    fn ecs_component_add_and_get() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        let t = Transform {
            x: 10.0,
            y: 20.0,
            cell_x: 1,
            cell_y: 2,
        };
        ecs.transforms.insert(entity, t.clone());
        assert_eq!(ecs.transforms.get(&entity), Some(&t));
    }

    #[test]
    fn ecs_component_remove() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        ecs.transforms.insert(
            entity,
            Transform {
                x: 1.0,
                y: 2.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        assert!(ecs.transforms.contains_key(&entity));
        let removed = ecs.transforms.remove(&entity);
        assert!(removed.is_some());
        assert!(!ecs.transforms.contains_key(&entity));
    }

    #[test]
    fn ecs_sparse_set_integrity_after_insert_remove() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        ecs.transforms.insert(
            e1,
            Transform {
                x: 0.0,
                y: 0.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.transforms.insert(
            e2,
            Transform {
                x: 1.0,
                y: 1.0,
                cell_x: 1,
                cell_y: 1,
            },
        );
        
        // Remove first entity
        ecs.transforms.remove(&e1);
        assert!(!ecs.transforms.contains_key(&e1));
        assert!(ecs.transforms.contains_key(&e2));
        
        // Insert new entity
        let e3 = ecs.spawn();
        ecs.transforms.insert(
            e3,
            Transform {
                x: 2.0,
                y: 2.0,
                cell_x: 2,
                cell_y: 2,
            },
        );
        assert!(ecs.transforms.contains_key(&e2));
        assert!(ecs.transforms.contains_key(&e3));
    }

    #[test]
    fn ecs_bulk_spawn_performance() {
        let mut ecs = Ecs::new();
        let start = std::time::Instant::now();
        
        // Spawn many entities
        let entities: Vec<_> = (0..1000).map(|_| ecs.spawn()).collect();
        
        let duration = start.elapsed();
        assert_eq!(entities.len(), 1000);
        assert_eq!(ecs.alive.len(), 1000);
        assert!(duration.as_millis() < 10, "Bulk spawn should be fast");
    }

    #[test]
    fn ecs_bulk_despawn_performance() {
        let mut ecs = Ecs::new();
        let entities: Vec<_> = (0..1000).map(|_| ecs.spawn()).collect();
        
        let start = std::time::Instant::now();
        
        // Despawn all entities
        for entity in entities {
            ecs.despawn(entity);
        }
        
        let duration = start.elapsed();
        assert_eq!(ecs.alive.len(), 0);
        assert!(duration.as_millis() < 10, "Bulk despawn should be fast");
    }

    #[test]
    fn ecs_entity_uniqueness_guaranteed() {
        let mut ecs = Ecs::new();
        let mut entities = std::collections::HashSet::new();
        
        // Spawn many entities and check uniqueness
        for _ in 0..1000 {
            let entity = ecs.spawn();
            assert!(!entities.contains(&entity), "Entity IDs must be unique");
            entities.insert(entity);
        }
        
        assert_eq!(entities.len(), 1000);
    }

    #[test]
    fn ecs_component_isolation_between_entities() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        
        let t1 = Transform { x: 1.0, y: 2.0, cell_x: 0, cell_y: 0 };
        let t2 = Transform { x: 3.0, y: 4.0, cell_x: 1, cell_y: 1 };
        
        ecs.transforms.insert(e1, t1);
        ecs.transforms.insert(e2, t2);
        
        assert_eq!(ecs.transforms.get(&e1), Some(&t1));
        assert_eq!(ecs.transforms.get(&e2), Some(&t2));
        
        // Modify one component
        let t1_modified = Transform { x: 10.0, y: 20.0, cell_x: 5, cell_y: 5 };
        ecs.transforms.insert(e1, t1_modified);
        
        assert_eq!(ecs.transforms.get(&e1), Some(&t1_modified));
        assert_eq!(ecs.transforms.get(&e2), Some(&t2)); // Should be unchanged
    }

    #[test]
    fn ecs_memory_usage_stable() {
        let mut ecs = Ecs::new();
        let initial_memory = ecs.estimate_memory_usage();
        
        // Add entities and components
        for i in 0..100 {
            let entity = ecs.spawn();
            ecs.transforms.insert(entity, Transform {
                x: i as f32,
                y: i as f32,
                cell_x: i,
                cell_y: i,
            });
        }
        
        let with_entities_memory = ecs.estimate_memory_usage();
        
        // Remove half
        for i in 0..50 {
            let entity = ecs.alive.iter().nth(0).copied().unwrap();
            ecs.despawn(entity);
        }
        
        let after_removal_memory = ecs.estimate_memory_usage();
        
        assert!(with_entities_memory > initial_memory);
        assert!(after_removal_memory < with_entities_memory);
        assert!(after_removal_memory > initial_memory);
    }

    #[test]
    fn ecs_concurrent_spawn_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let ecs = Arc::new(Mutex::new(Ecs::new()));
        let mut handles = vec![];
        
        // Spawn from multiple threads
        for _ in 0..4 {
            let ecs_clone = Arc::clone(&ecs);
            let handle = thread::spawn(move || {
                let mut ecs = ecs_clone.lock().unwrap();
                let mut entities = vec![];
                for _ in 0..100 {
                    entities.push(ecs.spawn());
                }
                entities
            });
            handles.push(handle);
        }
        
        // Collect all entities
        let mut all_entities = vec![];
        for handle in handles {
            all_entities.extend(handle.join().unwrap());
        }
        
        // Verify uniqueness
        let mut unique_entities = std::collections::HashSet::new();
        for entity in all_entities {
            assert!(unique_entities.insert(entity), "Entity IDs must be unique across threads");
        }
        
        let ecs = ecs.lock().unwrap();
        assert_eq!(ecs.alive.len(), 400);
    }
}

#[cfg(test)]
mod ecs_component_management_tests {
    use engene::core::ecs::Ecs;
    use engene::world::components::{Transform, Velocity, Kind};

    #[test]
    fn ecs_multiple_component_types() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        
        let transform = Transform { x: 1.0, y: 2.0, cell_x: 0, cell_y: 0 };
        let velocity = Velocity { vx: 3.0, vy: 4.0 };
        let kind = Kind { value: 42 };
        
        ecs.transforms.insert(entity, transform.clone());
        ecs.velocities.insert(entity, velocity.clone());
        ecs.kinds.insert(entity, kind.clone());
        
        assert_eq!(ecs.transforms.get(&entity), Some(&transform));
        assert_eq!(ecs.velocities.get(&entity), Some(&velocity));
        assert_eq!(ecs.kinds.get(&entity), Some(&kind));
    }

    #[test]
    fn ecs_partial_component_removal() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        
        ecs.transforms.insert(entity, Transform { x: 1.0, y: 2.0, cell_x: 0, cell_y: 0 });
        ecs.velocities.insert(entity, Velocity { vx: 3.0, vy: 4.0 });
        ecs.kinds.insert(entity, Kind { value: 42 });
        
        // Remove only transform
        ecs.transforms.remove(&entity);
        
        assert!(!ecs.transforms.contains_key(&entity));
        assert!(ecs.velocities.contains_key(&entity));
        assert!(ecs.kinds.contains_key(&entity));
        
        // Entity should still be alive
        assert!(ecs.is_alive(entity));
    }

    #[test]
    fn ecs_component_query_performance() {
        let mut ecs = Ecs::new();
        
        // Create entities with components
        for i in 0..1000 {
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
        
        // Query entities with both Transform and Velocity
        let count = ecs.transforms.iter()
            .filter(|(entity, _)| ecs.velocities.contains_key(entity))
            .count();
        
        let duration = start.elapsed();
        
        assert_eq!(count, 500); // Half of the entities should have velocity
        assert!(duration.as_millis() < 5, "Component query should be fast");
    }

    #[test]
    fn ecs_sparse_set_density_optimization() {
        let mut ecs = Ecs::new();
        
        // Create sparse entities (every 10th entity has components)
        for i in 0..1000 {
            let entity = ecs.spawn();
            if i % 10 == 0 {
                ecs.transforms.insert(entity, Transform {
                    x: i as f32,
                    y: i as f32,
                    cell_x: i / 10,
                    cell_y: i / 10,
                });
            }
        }
        
        // Verify sparse set efficiency
        assert_eq!(ecs.transforms.len(), 100);
        assert_eq!(ecs.alive.len(), 1000);
        
        // Query should be efficient despite sparsity
        let start = std::time::Instant::now();
        let count: usize = ecs.transforms.iter().count();
        let duration = start.elapsed();
        
        assert_eq!(count, 100);
        assert!(duration.as_millis() < 1, "Sparse set iteration should be very fast");
    }

    #[test]
    fn ecs_component_type_safety() {
        let mut ecs = Ecs::new();
        let entity = ecs.spawn();
        
        // Different component types should be independent
        ecs.transforms.insert(entity, Transform { x: 1.0, y: 2.0, cell_x: 0, cell_y: 0 });
        ecs.velocities.insert(entity, Velocity { vx: 3.0, vy: 4.0 });
        
        // Type safety should prevent mixing
        let transform = ecs.transforms.get(&entity);
        let velocity = ecs.velocities.get(&entity);
        
        assert!(transform.is_some());
        assert!(velocity.is_some());
        
        // Should not be able to get velocity from transforms
        assert!(ecs.transforms.get(&entity).unwrap().x == 1.0);
        assert!(ecs.velocities.get(&entity).unwrap().vx == 3.0);
    }
}
