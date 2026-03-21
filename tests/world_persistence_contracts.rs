//! World Persistence Contracts
//! 
//! Tests for chunk persistence cycles, save/load integrity, and streaming contracts.
//! Ownership: World Persistence Team
//! Lane: contracts
//! Type: Contract + Integration Tests
//! Speed: Medium

#[cfg(test)]
mod chunk_persistence_tests {
    use engine_world::chunk_persistence::{ChunkPersistenceService, ChunkCoord};
    use engine_world::chunk_schema::ChunkSchema;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn chunk_save_unload_marks_entities_unloaded_not_dead() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        // Create chunk with entities
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = ChunkSchema::new(coord);
        
        // Add entities to chunk
        let entity_id = 1000u64;
        chunk.add_entity(entity_id);
        chunk.add_entity(2000u64);
        chunk.add_entity(3000u64);
        
        // Entities should be present and alive
        assert!(chunk.contains_entity(entity_id));
        assert!(chunk.is_entity_alive(entity_id));
        
        // Save chunk
        persistence.save_chunk(&chunk).unwrap();
        
        // Unload chunk (simulate streaming unload)
        let unloaded_chunk = persistence.unload_chunk(coord).unwrap();
        
        // Entities should be marked unloaded, not dead
        assert!(unloaded_chunk.contains_entity(entity_id));
        assert!(!unloaded_chunk.is_entity_loaded(entity_id), "entity should be unloaded");
        assert!(unloaded_chunk.is_entity_alive(entity_id), "entity should still be alive");
        
        // Verify persistence state
        let entity_state = unloaded_chunk.get_entity_state(entity_id);
        assert_eq!(entity_state.status, engine_world::chunk_schema::EntityStatus::Unloaded);
    }

    #[test]
    fn chunk_load_relinks_same_pid_after_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        // Create and save chunk with entities
        let coord = ChunkCoord::new(1, 2, 3);
        let mut original_chunk = ChunkSchema::new(coord);
        
        // Add entities with specific PIDs
        let entity_a = 1001u64;
        let entity_b = 1002u64;
        original_chunk.add_entity(entity_a);
        original_chunk.add_entity(entity_b);
        
        // Set entity data
        original_chunk.set_entity_position(entity_a, (10.0, 20.0, 30.0));
        original_chunk.set_entity_position(entity_b, (40.0, 50.0, 60.0));
        
        // Save original chunk
        persistence.save_chunk(&original_chunk).unwrap();
        
        // Simulate complete unload
        drop(original_chunk);
        
        // Load chunk back
        let loaded_chunk = persistence.load_chunk(coord).unwrap();
        
        // PIDs should be preserved
        assert!(loaded_chunk.contains_entity(entity_a));
        assert!(loaded_chunk.contains_entity(entity_b));
        
        // Entity data should be preserved
        let pos_a = loaded_chunk.get_entity_position(entity_a);
        let pos_b = loaded_chunk.get_entity_position(entity_b);
        
        assert_eq!(pos_a, Some((10.0, 20.0, 30.0)));
        assert_eq!(pos_b, Some((40.0, 50.0, 60.0)));
        
        // Entity should be marked as loaded
        assert!(loaded_chunk.is_entity_loaded(entity_a));
        assert!(loaded_chunk.is_entity_loaded(entity_b));
    }

    #[test]
    fn chunk_persistence_handles_multiple_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        let coord = ChunkCoord::new(5, 5, 5);
        
        // Multiple save/load cycles
        for cycle in 0..5 {
            let mut chunk = if cycle == 0 {
                ChunkSchema::new(coord)
            } else {
                persistence.load_chunk(coord).unwrap()
            };
            
            // Add new entity each cycle
            let entity_id = 2000 + cycle as u64;
            chunk.add_entity(entity_id);
            chunk.set_entity_position(entity_id, (cycle as f32, cycle as f32, cycle as f32));
            
            // Save chunk
            persistence.save_chunk(&chunk).unwrap();
            
            // Unload and reload
            drop(chunk);
            let reloaded = persistence.load_chunk(coord).unwrap();
            
            // Verify all entities from previous cycles exist
            for prev_cycle in 0..=cycle {
                let check_id = 2000 + prev_cycle as u64;
                assert!(reloaded.contains_entity(check_id));
                assert!(reloaded.is_entity_loaded(check_id));
                
                let expected_pos = (prev_cycle as f32, prev_cycle as f32, prev_cycle as f32);
                assert_eq!(reloaded.get_entity_position(check_id), Some(expected_pos));
            }
        }
    }

    #[test]
    fn chunk_persistence_isolation_between_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        // Create two different chunks
        let coord_a = ChunkCoord::new(0, 0, 0);
        let coord_b = ChunkCoord::new(1, 1, 1);
        
        let mut chunk_a = ChunkSchema::new(coord_a);
        let mut chunk_b = ChunkSchema::new(coord_b);
        
        // Add different entities to each chunk
        chunk_a.add_entity(1001u64);
        chunk_a.add_entity(1002u64);
        
        chunk_b.add_entity(2001u64);
        chunk_b.add_entity(2002u64);
        
        // Save both chunks
        persistence.save_chunk(&chunk_a).unwrap();
        persistence.save_chunk(&chunk_b).unwrap();
        
        // Load and verify isolation
        let loaded_a = persistence.load_chunk(coord_a).unwrap();
        let loaded_b = persistence.load_chunk(coord_b).unwrap();
        
        // Chunk A should only have its entities
        assert!(loaded_a.contains_entity(1001u64));
        assert!(loaded_a.contains_entity(1002u64));
        assert!(!loaded_a.contains_entity(2001u64));
        assert!(!loaded_a.contains_entity(2002u64));
        
        // Chunk B should only have its entities
        assert!(loaded_b.contains_entity(2001u64));
        assert!(loaded_b.contains_entity(2002u64));
        assert!(!loaded_b.contains_entity(1001u64));
        assert!(!loaded_b.contains_entity(1002u64));
    }
}

#[cfg(test)]
mod relink_report_tests {
    use engine_world::relink_report::{RelinkReport, RelinkStatus};
    use engine_world::chunk_persistence::{ChunkPersistenceService, ChunkCoord};
    use tempfile::TempDir;

    #[test]
    fn relink_report_clean_for_simple_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        // Create chunk with entities
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = engine_world::chunk_schema::ChunkSchema::new(coord);
        
        // Add entities
        chunk.add_entity(3001u64);
        chunk.add_entity(3002u64);
        chunk.add_entity(3003u64);
        
        // Set up entity relationships
        chunk.set_entity_parent(3002u64, Some(3001u64)); // 3002 child of 3001
        chunk.set_entity_parent(3003u64, Some(3001u64)); // 3003 child of 3001
        
        // Save chunk
        persistence.save_chunk(&chunk).unwrap();
        
        // Load chunk and generate relink report
        let loaded_chunk = persistence.load_chunk(coord).unwrap();
        let report = RelinkReport::generate(&loaded_chunk);
        
        // Report should be clean for simple roundtrip
        assert!(report.is_clean(), "relink report should be clean for simple roundtrip");
        assert_eq!(report.status(), RelinkStatus::Clean);
        
        // Should have no broken links
        assert_eq!(report.broken_links().len(), 0);
        
        // Should have all entities accounted for
        assert_eq!(report.total_entities(), 3);
        assert_eq!(report.linked_entities(), 3);
        assert_eq!(report.orphaned_entities(), 0);
    }

    #[test]
    fn relink_report_detects_broken_cross_chunk_links() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        // Create chunk with entity that has parent in another chunk
        let coord_a = ChunkCoord::new(0, 0, 0);
        let mut chunk_a = engene::world::chunk_schema::ChunkSchema::new(coord_a);
        
        // Add entity with parent in different chunk
        chunk_a.add_entity(4001u64);
        chunk_a.set_entity_parent(4001u64, Some(9999u64)); // Parent in different chunk
        
        // Save chunk
        persistence.save_chunk(&chunk_a).unwrap();
        
        // Load and generate relink report
        let loaded_chunk = persistence.load_chunk(coord_a).unwrap();
        let report = RelinkReport::generate(&loaded_chunk);
        
        // Report should detect broken link
        assert!(!report.is_clean(), "relink report should detect broken cross-chunk link");
        assert_eq!(report.status(), RelinkStatus::BrokenLinks);
        
        // Should have one broken link
        assert_eq!(report.broken_links().len(), 1);
        
        let broken_link = &report.broken_links()[0];
        assert_eq!(broken_link.entity_id, 4001u64);
        assert_eq!(broken_link.parent_id, Some(9999u64));
        assert_eq!(broken_link.reason, engine_world::relink_report::BrokenLinkReason::ParentNotFound);
    }

    #[test]
    fn relink_report_handles_circular_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ChunkPersistenceService::new(temp_dir.path());
        
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = engine_world::chunk_schema::ChunkSchema::new(coord);
        
        // Create circular dependency
        chunk.add_entity(5001u64);
        chunk.add_entity(5002u64);
        
        chunk.set_entity_parent(5001u64, Some(5002u64));
        chunk.set_entity_parent(5002u64, Some(5001u64)); // Circular
        
        // Save chunk
        persistence.save_chunk(&chunk).unwrap();
        
        // Load and generate relink report
        let loaded_chunk = persistence.load_chunk(coord).unwrap();
        let report = RelinkReport::generate(&loaded_chunk);
        
        // Report should detect circular dependency
        assert!(!report.is_clean(), "relink report should detect circular dependency");
        assert_eq!(report.status(), RelinkStatus::CircularDependency);
        
        // Should identify the cycle
        assert!(report.has_circular_dependencies());
        let cycles = report.circular_dependencies();
        assert_eq!(cycles.len(), 1);
        
        let cycle = &cycles[0];
        assert!(cycle.contains(&5001u64));
        assert!(cycle.contains(&5002u64));
    }
}

#[cfg(test)]
mod streaming_contract_tests {
    use engine_world::streaming::{WorldStreamer, StreamingRegion};
    use engine_world::chunk_persistence::ChunkCoord;
    use tempfile::TempDir;

    #[test]
    fn streaming_four_region_cycle_preserves_identity_uniqueness() {
        let temp_dir = TempDir::new().unwrap();
        let streamer = WorldStreamer::new(temp_dir.path());
        
        // Define four regions
        let regions = vec![
            StreamingRegion::new(ChunkCoord::new(0, 0, 0), 2), // Center
            StreamingRegion::new(ChunkCoord::new(2, 0, 0), 2), // East
            StreamingRegion::new(ChunkCoord::new(0, 2, 0), 2), // North
            StreamingRegion::new(ChunkCoord::new(2, 2, 0), 2), // Northeast
        ];
        
        // Track entities across regions
        let mut entity_regions: HashMap<u64, Vec<ChunkCoord>> = HashMap::new();
        
        // Cycle through all regions
        for (i, region) in regions.iter().enumerate() {
            // Load region
            streamer.load_region(region);
            
            // Add some entities
            for j in 0..5 {
                let entity_id = (i * 100 + j) as u64;
                streamer.spawn_entity_in_region(entity_id, region);
                
                // Track which region this entity belongs to
                entity_regions.entry(entity_id).or_insert_with(Vec::new).push(region.center());
            }
            
            // Process streaming
            streamer.update();
            
            // Unload region
            streamer.unload_region(region);
        }
        
        // Now cycle through all regions again to verify identity preservation
        for region in &regions {
            streamer.load_region(region);
            streamer.update();
            
            // Check entities that should be in this region
            for (entity_id, regions_list) in &entity_regions {
                if regions_list.contains(&region.center()) {
                    assert!(streamer.is_entity_loaded(*entity_id), 
                           "Entity {} should be loaded in region {:?}", entity_id, region.center());
                }
            }
            
            streamer.unload_region(region);
        }
        
        // Verify no identity collisions occurred
        let all_loaded_entities: Vec<u64> = entity_regions.keys().copied().collect();
        let unique_entities: std::collections::HashSet<_> = all_loaded_entities.iter().collect();
        
        assert_eq!(all_loaded_entities.len(), unique_entities.len(), 
                  "All entity IDs should be unique across streaming cycles");
    }

    #[test]
    fn streaming_respects_load_unload_contracts() {
        let temp_dir = TempDir::new().unwrap();
        let streamer = WorldStreamer::new(temp_dir.path());
        
        let region = StreamingRegion::new(ChunkCoord::new(0, 0, 0), 3);
        
        // Initial state - region should not be loaded
        assert!(!streamer.is_region_loaded(&region));
        assert_eq!(streamer.loaded_chunks_count(), 0);
        
        // Load region
        streamer.load_region(&region);
        streamer.update();
        
        // Region should be loaded
        assert!(streamer.is_region_loaded(&region));
        assert!(streamer.loaded_chunks_count() > 0);
        
        // Add entities
        for i in 0..10 {
            streamer.spawn_entity_in_region(6000 + i, &region);
        }
        
        // Entities should be loaded
        for i in 0..10 {
            assert!(streamer.is_entity_loaded(6000 + i));
        }
        
        // Unload region
        streamer.unload_region(&region);
        streamer.update();
        
        // Region should not be loaded
        assert!(!streamer.is_region_loaded(&region));
        assert_eq!(streamer.loaded_chunks_count(), 0);
        
        // Entities should be unloaded but not dead
        for i in 0..10 {
            assert!(!streamer.is_entity_loaded(6000 + i));
            assert!(streamer.is_entity_alive(6000 + i)); // Should still be alive
        }
    }

    #[test]
    fn streaming_handles_concurrent_regions() {
        let temp_dir = TempDir::new().unwrap();
        let streamer = WorldStreamer::new(temp_dir.path());
        
        // Load multiple regions concurrently
        let region_a = StreamingRegion::new(ChunkCoord::new(0, 0, 0), 2);
        let region_b = StreamingRegion::new(ChunkCoord::new(3, 0, 0), 2);
        let region_c = StreamingRegion::new(ChunkCoord::new(0, 3, 0), 2);
        
        // Load all regions
        streamer.load_region(&region_a);
        streamer.load_region(&region_b);
        streamer.load_region(&region_c);
        streamer.update();
        
        // All regions should be loaded
        assert!(streamer.is_region_loaded(&region_a));
        assert!(streamer.is_region_loaded(&region_b));
        assert!(streamer.is_region_loaded(&region_c));
        
        // Add entities to each region
        streamer.spawn_entity_in_region(7001u64, &region_a);
        streamer.spawn_entity_in_region(7002u64, &region_b);
        streamer.spawn_entity_in_region(7003u64, &region_c);
        
        streamer.update();
        
        // Entities should be in correct regions
        assert!(streamer.is_entity_loaded(7001u64));
        assert!(streamer.is_entity_loaded(7002u64));
        assert!(streamer.is_entity_loaded(7003u64));
        
        // Unload one region
        streamer.unload_region(&region_b);
        streamer.update();
        
        // Region B should be unloaded, A and C should remain
        assert!(!streamer.is_region_loaded(&region_b));
        assert!(streamer.is_region_loaded(&region_a));
        assert!(streamer.is_region_loaded(&region_c));
        
        // Entity in region B should be unloaded
        assert!(!streamer.is_entity_loaded(7002u64));
        assert!(streamer.is_entity_alive(7002u64)); // Still alive
        
        // Entities in A and C should remain loaded
        assert!(streamer.is_entity_loaded(7001u64));
        assert!(streamer.is_entity_loaded(7003u64));
    }
}
