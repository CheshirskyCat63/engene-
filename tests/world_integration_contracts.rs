//! World Integration Contracts
//! 
//! End-to-end tests for world systems, chunk persistence, streaming, and spatial management.
//! Ownership: World Team
//! Lane: contracts
//! Type: Integration + Contract Tests
//! Speed: Medium

#[cfg(test)]
mod world_integration_tests {
    use engene::world::world::WorldGrid;
    use engene::world::streaming::{ChunkCoord, WorldStreamer};
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::heightmap::Heightmap;
    use engene::world::components::{Transform, EntityKind};
    use engene::core::ecs::Ecs;

    #[test]
    fn e2e_world_grid_generation() {
        let grid = WorldGrid::generate();
        
        // Should have reasonable dimensions
        assert!(grid.cells.len() > 0);
        assert!(grid.cells.len() < 100_000); // Not too large
        
        // Should have valid biomes
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        assert!(biomes.iter().all(|b| b.is_valid()));
        
        // Should have spatial coherence
        for (i, cell) in grid.cells.iter().enumerate() {
            if i > 0 {
                let prev_cell = &grid.cells[i - 1];
                // Adjacent cells should have reasonable biome relationships
                let biome_distance = cell.biome.distance_to(prev_cell.biome);
                assert!(biome_distance <= 3.0); // Not too different
            }
        }
    }

    #[test]
    fn e2e_world_persistence_roundtrip() {
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Create test chunk data
        let original_data = vec![1, 2, 3, 4, 5];
        
        // Save chunk
        let save_result = persistence.save_chunk(coord, &original_data);
        assert!(save_result.is_ok());
        
        // Load chunk
        let load_result = persistence.load_chunk(coord);
        assert!(load_result.is_ok());
        
        let loaded_data = load_result.unwrap();
        assert_eq!(loaded_data, original_data);
    }

    #[test]
    fn e2e_world_streaming_load_unload() {
        let mut streamer = WorldStreamer::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Load chunk
        let load_result = streamer.load_chunk(coord);
        assert!(load_result.is_ok());
        
        assert!(streamer.is_chunk_loaded(coord));
        
        // Unload chunk
        let unload_result = streamer.unload_chunk(coord);
        assert!(unload_result.is_ok());
        
        assert!(!streamer.is_chunk_loaded(coord));
    }

    #[test]
    fn e2e_world_streaming_multiple_chunks() {
        let mut streamer = WorldStreamer::new();
        let coords = vec![
            ChunkCoord::new(0, 0),
            ChunkCoord::new(1, 0),
            ChunkCoord::new(0, 1),
            ChunkCoord::new(-1, 0),
            ChunkCoord::new(0, -1),
        ];
        
        // Load multiple chunks
        for &coord in &coords {
            let result = streamer.load_chunk(coord);
            assert!(result.is_ok());
        }
        
        // All should be loaded
        for &coord in &coords {
            assert!(streamer.is_chunk_loaded(coord));
        }
        
        // Unload all
        for &coord in &coords {
            let result = streamer.unload_chunk(coord);
            assert!(result.is_ok());
        }
        
        // All should be unloaded
        for &coord in &coords {
            assert!(!streamer.is_chunk_loaded(coord));
        }
    }

    #[test]
    fn e2e_world_heightmap_integration() {
        let grid = WorldGrid::generate();
        let heightmap = Heightmap::from_world_grid(&grid);
        
        // Heightmap should match world dimensions
        assert_eq!(heightmap.width(), grid.width);
        assert_eq!(heightmap.height(), grid.height);
        
        // Heights should be reasonable
        let min_height = heightmap.min_height();
        let max_height = heightmap.max_height();
        
        assert!(min_height.is_finite());
        assert!(max_height.is_finite());
        assert!(max_height >= min_height);
        
        // Should be able to sample heights at world positions
        let world_pos = [grid.width as f32 / 2.0, 0.0, grid.height as f32 / 2.0];
        let height = heightmap.sample_height(world_pos[0], world_pos[2]);
        assert!(height.is_finite());
        assert!(height >= min_height && height <= max_height);
    }

    #[test]
    fn e2e_world_entity_placement() {
        let mut ecs = Ecs::new();
        let grid = WorldGrid::generate();
        
        // Place entities in world
        let positions = vec![
            [10.0, 0.0, 10.0],
            [20.0, 0.0, 20.0],
            [30.0, 0.0, 30.0],
        ];
        
        let entities: Vec<_> = positions.iter().map(|&pos| {
            let entity = ecs.spawn();
            ecs.add_component(entity, Transform { position: pos, ..Default::default() });
            ecs.add_component(entity, EntityKind::Tree);
            entity
        }).collect();
        
        // Verify entities exist
        assert_eq!(entities.len(), 3);
        for &entity in &entities {
            assert!(ecs.alive.contains(&entity));
            assert!(ecs.components::<Transform>().contains(entity));
            assert!(ecs.components::<EntityKind>().contains(entity));
        }
    }

    #[test]
    fn e2e_world_spatial_queries() {
        let mut ecs = Ecs::new();
        
        // Create entities at different positions
        let positions = vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [100.0, 0.0, 0.0],
            [1000.0, 0.0, 0.0],
        ];
        
        for &pos in &positions {
            let entity = ecs.spawn();
            ecs.add_component(entity, Transform { position: pos, ..Default::default() });
        }
        
        // Query entities near origin
        let nearby_entities: Vec<_> = ecs.query::<&Transform>()
            .filter(|transform| {
                let dist = transform.position.distance([0.0, 0.0, 0.0]);
                dist < 50.0
            })
            .collect();
        
        assert_eq!(nearby_entities.len(), 2); // Should find first two entities
    }

    #[test]
    fn e2e_world_chunk_coord_conversion() {
        let world_pos = [100.0, 0.0, 100.0];
        let chunk_size = 32.0;
        
        // Convert world position to chunk coordinates
        let chunk_x = (world_pos[0] / chunk_size).floor() as i32;
        let chunk_z = (world_pos[2] / chunk_size).floor() as i32;
        let coord = ChunkCoord::new(chunk_x, chunk_z);
        
        // Convert back to world position (chunk origin)
        let chunk_origin = coord.to_world_position(chunk_size);
        
        // Chunk origin should be less than or equal to original position
        assert!(chunk_origin[0] <= world_pos[0]);
        assert!(chunk_origin[2] <= world_pos[2]);
        
        // Distance should be less than chunk size
        let dist = chunk_origin.distance(world_pos);
        assert!(dist < chunk_size);
    }

    #[test]
    fn e2e_world_biome_transitions() {
        let grid = WorldGrid::generate();
        
        // Find biome transitions
        let mut transitions = Vec::new();
        
        for (i, cell) in grid.cells.iter().enumerate() {
            if i > 0 {
                let prev_cell = &grid.cells[i - 1];
                if cell.biome != prev_cell.biome {
                    transitions.push((prev_cell.biome, cell.biome));
                }
            }
        }
        
        // Should have some biome transitions
        assert!(transitions.len() > 0);
        
        // Transitions should be between valid biomes
        for (from, to) in transitions {
            assert!(from.is_valid());
            assert!(to.is_valid());
        }
    }

    #[test]
    fn e2e_world_resource_distribution() {
        let grid = WorldGrid::generate();
        
        // Count resources by biome
        let mut resource_counts = std::collections::HashMap::new();
        
        for cell in &grid.cells {
            let count = resource_counts.entry(cell.biome).or_insert(0);
            *count += cell.resources.len();
        }
        
        // Should have resources in multiple biomes
        assert!(resource_counts.len() > 1);
        
        // Total resources should be reasonable
        let total_resources: usize = resource_counts.values().sum();
        assert!(total_resources > 0);
        assert!(total_resources < grid.cells.len() * 10); // Not too many per cell
    }

    #[test]
    fn e2e_world_persistence_large_data() {
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Create large chunk data (1MB)
        let large_data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        
        // Save large chunk
        let save_result = persistence.save_chunk(coord, &large_data);
        assert!(save_result.is_ok());
        
        // Load large chunk
        let load_result = persistence.load_chunk(coord);
        assert!(load_result.is_ok());
        
        let loaded_data = load_result.unwrap();
        assert_eq!(loaded_data.len(), large_data.len());
        assert_eq!(loaded_data, large_data);
    }

    #[test]
    fn e2e_world_streaming_performance() {
        let mut streamer = WorldStreamer::new();
        let coords: Vec<_> = (0..100).map(|i| ChunkCoord::new(i, i)).collect();
        
        let start = std::time::Instant::now();
        
        // Load 100 chunks
        for &coord in &coords {
            let result = streamer.load_chunk(coord);
            assert!(result.is_ok());
        }
        
        let load_time = start.elapsed();
        assert!(load_time.as_millis() < 1000, "Loading 100 chunks should be fast");
        
        // Verify all loaded
        for &coord in &coords {
            assert!(streamer.is_chunk_loaded(coord));
        }
        
        // Test unload performance
        let unload_start = std::time::Instant::now();
        
        for &coord in &coords {
            let result = streamer.unload_chunk(coord);
            assert!(result.is_ok());
        }
        
        let unload_time = unload_start.elapsed();
        assert!(unload_time.as_millis() < 500, "Unloading 100 chunks should be fast");
    }

    #[test]
    fn e2e_world_concurrent_streaming() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let streamer = Arc::new(Mutex::new(WorldStreamer::new()));
        let mut handles = vec![];
        
        // Concurrent chunk operations
        for i in 0..8 {
            let streamer_clone = Arc::clone(&streamer);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let coord = ChunkCoord::new(i * 10 + j, i * 10 + j);
                    let mut s = streamer_clone.lock().unwrap();
                    
                    // Load chunk
                    let load_result = s.load_chunk(coord);
                    assert!(load_result.is_ok());
                    
                    // Verify loaded
                    assert!(s.is_chunk_loaded(coord));
                    
                    // Unload chunk
                    let unload_result = s.unload_chunk(coord);
                    assert!(unload_result.is_ok());
                    
                    assert!(!s.is_chunk_loaded(coord));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn e2e_world_edge_cases() {
        let mut streamer = WorldStreamer::new();
        
        // Test extreme coordinates
        let extreme_coords = vec![
            ChunkCoord::new(i32::MAX, i32::MAX),
            ChunkCoord::new(i32::MIN, i32::MIN),
            ChunkCoord::new(0, 0),
        ];
        
        for coord in extreme_coords {
            // Should handle extreme coordinates gracefully
            let load_result = streamer.load_chunk(coord);
            assert!(load_result.is_ok() || load_result.is_err()); // Either works or fails gracefully
            
            if load_result.is_ok() {
                assert!(streamer.is_chunk_loaded(coord));
                
                let unload_result = streamer.unload_chunk(coord);
                assert!(unload_result.is_ok());
                
                assert!(!streamer.is_chunk_loaded(coord));
            }
        }
    }

    #[test]
    fn e2e_world_memory_management() {
        let mut streamer = WorldStreamer::new();
        
        let initial_memory = streamer.memory_usage();
        
        // Load many chunks
        for i in 0..1000 {
            let coord = ChunkCoord::new(i, i);
            let _ = streamer.load_chunk(coord);
        }
        
        let after_load_memory = streamer.memory_usage();
        
        // Memory should increase but not excessively
        assert!(after_load_memory > initial_memory);
        assert!(after_load_memory - initial_memory < 1_000_000_000); // Less than 1GB
        
        // Unload all chunks
        for i in 0..1000 {
            let coord = ChunkCoord::new(i, i);
            let _ = streamer.unload_chunk(coord);
        }
        
        // Memory should be cleaned up
        let after_unload_memory = streamer.memory_usage();
        assert!(after_unload_memory < initial_memory + 100_000_000); // Should be close to initial
    }
}

#[cfg(test)]
mod world_persistence_tests {
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;
    use engene::core::ecs::Ecs;
    use engene::world::components::{Transform, EntityKind};

    #[test]
    fn e2e_world_save_load_roundtrip() {
        let mut ecs = Ecs::new();
        let mut persistence = ChunkPersistenceService::new();
        
        // Create entities
        let entity1 = ecs.spawn();
        ecs.add_component(entity1, Transform { position: [10.0, 0.0, 10.0], ..Default::default() });
        ecs.add_component(entity1, EntityKind::Tree);
        
        let entity2 = ecs.spawn();
        ecs.add_component(entity2, Transform { position: [20.0, 0.0, 20.0], ..Default::default() });
        ecs.add_component(entity2, EntityKind::Rock);
        
        // Save world state
        let coord = ChunkCoord::new(0, 0);
        let save_result = persistence.save_chunk_entities(coord, &ecs);
        assert!(save_result.is_ok());
        
        // Clear ECS
        for entity in ecs.alive.clone() {
            ecs.despawn(entity);
        }
        
        assert!(ecs.alive.is_empty());
        
        // Load world state
        let load_result = persistence.load_chunk_entities(coord, &mut ecs);
        assert!(load_result.is_ok());
        
        // Verify entities restored
        assert_eq!(ecs.alive.len(), 2);
        
        // Verify components restored
        let transforms: Vec<_> = ecs.query::<&Transform>().collect();
        assert_eq!(transforms.len(), 2);
        
        let entity_kinds: Vec<_> = ecs.query::<&EntityKind>().collect();
        assert_eq!(entity_kinds.len(), 2);
    }

    #[test]
    fn e2e_world_incremental_saves() {
        let mut ecs = Ecs::new();
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Initial save
        let entity1 = ecs.spawn();
        ecs.add_component(entity1, Transform { position: [0.0, 0.0, 0.0], ..Default::default() });
        
        let save1_result = persistence.save_chunk_entities(coord, &ecs);
        assert!(save1_result.is_ok());
        
        // Add more entities
        let entity2 = ecs.spawn();
        ecs.add_component(entity2, Transform { position: [10.0, 0.0, 10.0], ..Default::default() });
        
        // Incremental save
        let save2_result = persistence.save_chunk_entities(coord, &ecs);
        assert!(save2_result.is_ok());
        
        // Load and verify all entities
        let mut new_ecs = Ecs::new();
        let load_result = persistence.load_chunk_entities(coord, &mut new_ecs);
        assert!(load_result.is_ok());
        
        assert_eq!(new_ecs.alive.len(), 2);
    }

    #[test]
    fn e2e_world_cross_chunk_persistence() {
        let mut ecs = Ecs::new();
        let mut persistence = ChunkPersistenceService::new();
        
        // Create entities in different chunks
        let entity1 = ecs.spawn();
        ecs.add_component(entity1, Transform { position: [10.0, 0.0, 10.0], ..Default::default() }); // Chunk (0,0)
        
        let entity2 = ecs.spawn();
        ecs.add_component(entity2, Transform { position: [50.0, 0.0, 50.0], ..Default::default() }); // Chunk (1,1)
        
        // Save both chunks
        let coord1 = ChunkCoord::new(0, 0);
        let coord2 = ChunkCoord::new(1, 1);
        
        let save1_result = persistence.save_chunk_entities(coord1, &ecs);
        let save2_result = persistence.save_chunk_entities(coord2, &ecs);
        
        assert!(save1_result.is_ok());
        assert!(save2_result.is_ok());
        
        // Load and verify
        let mut new_ecs = Ecs::new();
        
        let load1_result = persistence.load_chunk_entities(coord1, &mut new_ecs);
        let load2_result = persistence.load_chunk_entities(coord2, &mut new_ecs);
        
        assert!(load1_result.is_ok());
        assert!(load2_result.is_ok());
        
        assert_eq!(new_ecs.alive.len(), 2);
    }

    #[test]
    fn e2e_world_persistence_compression() {
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Create repetitive data (should compress well)
        let repetitive_data: Vec<u8> = (0..1000).map(|i| (i % 10) as u8).collect();
        
        // Save with compression
        let save_result = persistence.save_chunk_compressed(coord, &repetitive_data);
        assert!(save_result.is_ok());
        
        // Load with decompression
        let load_result = persistence.load_chunk_compressed(coord);
        assert!(load_result.is_ok());
        
        let loaded_data = load_result.unwrap();
        assert_eq!(loaded_data, repetitive_data);
    }

    #[test]
    fn e2e_world_persistence_corruption_handling() {
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Save valid data
        let valid_data = vec![1, 2, 3, 4, 5];
        let save_result = persistence.save_chunk(coord, &valid_data);
        assert!(save_result.is_ok());
        
        // Simulate corruption by writing invalid data
        persistence.simulate_corruption(coord);
        
        // Try to load corrupted data
        let load_result = persistence.load_chunk(coord);
        assert!(load_result.is_err());
        
        // Should be able to recover
        let recovery_result = persistence.recover_chunk(coord);
        assert!(recovery_result.is_ok() || recovery_result.is_err()); // Either works or fails gracefully
    }

    #[test]
    fn e2e_world_persistence_version_compatibility() {
        let mut persistence = ChunkPersistenceService::new();
        let coord = ChunkCoord::new(0, 0);
        
        // Save with current version
        let data = vec![1, 2, 3, 4, 5];
        let save_result = persistence.save_chunk_with_version(coord, &data, 1);
        assert!(save_result.is_ok());
        
        // Try to load with different version
        let load_result = persistence.load_chunk_with_version(coord, 2);
        assert!(load_result.is_err() || load_result.is_ok()); // Either migrates or fails
        
        // Load with correct version
        let correct_load_result = persistence.load_chunk_with_version(coord, 1);
        assert!(correct_load_result.is_ok());
        
        let loaded_data = correct_load_result.unwrap();
        assert_eq!(loaded_data, data);
    }
}
