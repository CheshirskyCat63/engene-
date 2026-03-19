//! Spatial Dirty Contracts
//!
//! These tests verify the production HierarchicalSpatialIndex behavior.
//! They test incremental updates, rebuild consistency, and dirty flag management.
//!
//! IMPORTANT: Uses real production code from engene::world::hierarchical_spatial.

use engene::world::hierarchical_spatial::HierarchicalSpatialIndex;
use engene::core::ecs::Entity;

/// Helper to create entity from number
fn entity(n: u64) -> Entity {
    n
}

/// Incremental update (single move) should produce same result as full rebuild.
#[test]
fn incremental_matches_full_rebuild_for_single_move() {
    let mut index = HierarchicalSpatialIndex::new();
    
    // Insert initial entities
    let e1 = entity(1);
    let e2 = entity(2);
    let e3 = entity(3);
    
    index.insert_new(e1, 5.0, 5.0);
    index.insert_new(e2, 15.0, 15.0);
    index.insert_new(e3, 25.0, 25.0);
    
    // Move e1 using update
    index.update(e1, 5.0, 5.0, 10.0, 10.0);
    
    // Query after update
    let result_update = index.query_physics(10.0, 10.0, 20.0);
    
    // Now do full rebuild with same positions
    index.rebuild(&[(e1, 10.0, 10.0), (e2, 15.0, 15.0), (e3, 25.0, 25.0)]);
    
    // Query after rebuild
    let result_rebuild = index.query_physics(10.0, 10.0, 20.0);
    
    // Results should match
    assert_eq!(result_update.len(), result_rebuild.len(),
        "Incremental query count must match rebuild query count");
    
    for e in &result_update {
        assert!(result_rebuild.contains(e),
            "Entity {} from incremental must be in rebuild result", e);
    }
}

/// Incremental matches full rebuild for insert/remove mix.
#[test]
fn incremental_matches_full_rebuild_for_insert_remove_mix() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e1 = entity(1);
    let e2 = entity(2);
    let e3 = entity(3);
    let e4 = entity(4);
    
    // Initial insert
    index.insert_new(e1, 0.0, 0.0);
    index.insert_new(e2, 10.0, 10.0);
    index.insert_new(e3, 20.0, 20.0);
    
    // Remove one
    index.remove(e2);
    
    // Insert another
    index.insert_new(e4, 5.0, 5.0);
    
    // Update one
    index.update(e1, 0.0, 0.0, 15.0, 15.0);
    
    // Get incremental state
    let (c0, _, _) = index.entity_count();
    let incremental_count = c0;
    let incremental_query = index.query_physics(10.0, 10.0, 20.0);
    
    // Rebuild from final state
    index.rebuild(&[(e1, 15.0, 15.0), (e3, 20.0, 20.0), (e4, 5.0, 5.0)]);
    
    let (c0r, _, _) = index.entity_count();
    let rebuild_count = c0r;
    let rebuild_query = index.query_physics(10.0, 10.0, 20.0);
    
    assert_eq!(incremental_count, rebuild_count,
        "Entity count must match after insert/remove mix");
    assert_eq!(incremental_query.len(), rebuild_query.len(),
        "Query results must match after insert/remove mix");
}

/// Remove updates queries and counts consistently.
#[test]
fn remove_updates_queries_and_counts_consistently() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e1 = entity(1);
    let e2 = entity(2);
    
    index.insert_new(e1, 5.0, 5.0);
    index.insert_new(e2, 10.0, 10.0);
    
    // Verify both are queryable
    let before_remove = index.query_physics(7.0, 7.0, 10.0);
    assert!(before_remove.contains(&e1), "e1 must be found before remove");
    assert!(before_remove.contains(&e2), "e2 must be found before remove");
    
    // Remove e1
    index.remove(e1);
    
    // Verify e1 is no longer found
    let after_remove = index.query_physics(7.0, 7.0, 10.0);
    assert!(!after_remove.contains(&e1), "e1 must NOT be found after remove");
    assert!(after_remove.contains(&e2), "e2 must still be found after remove");
    
    // Verify count
    let (c0, _, _) = index.entity_count();
    assert_eq!(c0, 1, "Entity count must be 1 after remove");
}

/// Mark dirty sets rebuild flag.
#[test]
fn mark_dirty_sets_rebuild_flag() {
    let mut index = HierarchicalSpatialIndex::new();
    
    // Initial state: needs rebuild is true
    assert!(index.needs_rebuild(), "New index must need rebuild");
    
    // Insert clears dirty flag
    let e = entity(1);
    index.insert_new(e, 5.0, 5.0);
    assert!(!index.needs_rebuild(), "After insert, needs_rebuild must be false");
    
    // Mark dirty sets flag
    index.mark_dirty();
    assert!(index.needs_rebuild(), "After mark_dirty, needs_rebuild must be true");
}

/// Rebuild clears dirty flag and restores counts.
#[test]
fn rebuild_clears_dirty_flag_and_restores_counts() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e1 = entity(1);
    let e2 = entity(2);
    
    // Rebuild with entities
    index.rebuild(&[(e1, 10.0, 10.0), (e2, 20.0, 20.0)]);
    
    // Verify dirty flag cleared
    assert!(!index.needs_rebuild(), "After rebuild, needs_rebuild must be false");
    
    // Verify count
    let (c0, c1, c2) = index.entity_count();
    assert_eq!(c0, 2, "Level 0 entity count must be 2 after rebuild");
    assert_eq!(c1, 2, "Level 1 entity count must be 2 after rebuild");
    assert_eq!(c2, 2, "Level 2 entity count must be 2 after rebuild");
    
    // Verify queryable
    let result = index.query_physics(10.0, 10.0, 15.0);
    assert!(result.contains(&e1), "e1 must be found after rebuild");
}

/// No extra rebuild flag when incremental update is used.
#[test]
fn no_extra_rebuild_flag_when_incremental_update_is_used() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e = entity(1);
    index.insert_new(e, 5.0, 5.0);
    
    // After insert, no rebuild needed
    assert!(!index.needs_rebuild(), "After insert, no rebuild needed");
    
    // Incremental update should not set rebuild flag
    index.update(e, 5.0, 5.0, 15.0, 15.0);
    assert!(!index.needs_rebuild(), "After update, no rebuild needed");
    
    // Remove should not set rebuild flag
    let e2 = entity(2);
    index.insert_new(e2, 20.0, 20.0);
    index.remove(e2);
    assert!(!index.needs_rebuild(), "After remove, no rebuild needed");
    
    // Only mark_dirty or clear should set rebuild flag
    index.mark_dirty();
    assert!(index.needs_rebuild(), "After mark_dirty, rebuild needed");
}

/// Query methods work correctly at different LOD levels.
#[test]
fn query_methods_work_at_different_lod_levels() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e1 = entity(1);
    let e2 = entity(2);
    let e3 = entity(3);
    
    // Place entities at different distances
    index.insert_new(e1, 5.0, 5.0);    // Close
    index.insert_new(e2, 50.0, 50.0);  // Medium
    index.insert_new(e3, 500.0, 500.0); // Far
    
    // Physics query (level 0, small cell size = 10)
    let physics = index.query_physics(5.0, 5.0, 10.0);
    assert!(physics.contains(&e1), "e1 should be found in physics query");
    assert!(!physics.contains(&e2), "e2 should NOT be found in physics query");
    assert!(!physics.contains(&e3), "e3 should NOT be found in physics query");
    
    // AI query (level 1, medium cell size = 100)
    let ai = index.query_ai(50.0, 50.0, 100.0);
    assert!(!ai.contains(&e1), "e1 should NOT be found in AI query");
    assert!(ai.contains(&e2), "e2 should be found in AI query");
    assert!(!ai.contains(&e3), "e3 should NOT be found in AI query");
    
    // World query (level 2, large cell size = 1000)
    let world = index.query_world(500.0, 500.0, 1000.0);
    assert!(!world.contains(&e1), "e1 should NOT be found in world query");
    assert!(!world.contains(&e2), "e2 should NOT be found in world query");
    assert!(world.contains(&e3), "e3 should be found in world query");
}
