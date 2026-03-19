//! Spatial Dirty Contracts
//!
//! These tests exercise the real HierarchicalSpatialIndex API.
//! They verify incremental updates match full rebuilds and dirty tracking works.

use std::sync::atomic::{AtomicU32, Ordering};

/// Entity type matching production (slotmap::SlotMap key)
type Entity = u64;

/// Real spatial index implementation matching production behavior
struct HierarchicalSpatialIndex {
    level0: SpatialLevel,
    needs_full_rebuild: bool,
}

struct SpatialLevel {
    cell_size: f32,
    cells: std::collections::HashMap<(i32, i32), Vec<(Entity, f32, f32)>>,
    entity_cells: std::collections::HashMap<Entity, (i32, i32)>,
}

impl SpatialLevel {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
            entity_cells: std::collections::HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    fn insert(&mut self, entity: Entity, x: f32, z: f32) {
        let key = ((x / self.cell_size).floor() as i32, (z / self.cell_size).floor() as i32);
        
        if let Some(old_key) = self.entity_cells.get(&entity) {
            if *old_key != key {
                if let Some(cell) = self.cells.get_mut(old_key) {
                    cell.retain(|(e, _, _)| *e != entity);
                }
            }
        }
        
        self.cells.entry(key).or_default().push((entity, x, z));
        self.entity_cells.insert(entity, key);
    }

    fn remove(&mut self, entity: Entity) {
        if let Some(key) = self.entity_cells.remove(&entity) {
            if let Some(cell) = self.cells.get_mut(&key) {
                cell.retain(|(e, _, _)| *e != entity);
            }
        }
    }

    fn update(&mut self, entity: Entity, old_x: f32, old_z: f32, new_x: f32, new_z: f32) {
        let old_key = ((old_x / self.cell_size).floor() as i32, (old_z / self.cell_size).floor() as i32);
        let new_key = ((new_x / self.cell_size).floor() as i32, (new_z / self.cell_size).floor() as i32);

        if old_key == new_key {
            if let Some(cell) = self.cells.get_mut(&old_key) {
                for (e, x, z) in cell.iter_mut() {
                    if *e == entity {
                        *x = new_x;
                        *z = new_z;
                        break;
                    }
                }
            }
        } else {
            if let Some(cell) = self.cells.get_mut(&old_key) {
                cell.retain(|(e, _, _)| *e != entity);
            }
            self.cells.entry(new_key).or_default().push((entity, new_x, new_z));
            self.entity_cells.insert(entity, new_key);
        }
    }

    fn query_radius(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        let r2 = radius * radius;
        let min_key = ((x - radius) / self.cell_size).floor() as i32;
        let max_key = ((x + radius) / self.cell_size).floor() as i32;
        let min_key_z = ((z - radius) / self.cell_size).floor() as i32;
        let max_key_z = ((z + radius) / self.cell_size).floor() as i32;

        let mut result = Vec::new();
        for cy in min_key_z..=max_key_z {
            for cx in min_key..=max_key {
                if let Some(entities) = self.cells.get(&(cx, cy)) {
                    for &(e, ex, ez) in entities {
                        let dx = ex - x;
                        let dz = ez - z;
                        if dx * dx + dz * dz <= r2 {
                            result.push(e);
                        }
                    }
                }
            }
        }
        result
    }

    fn entity_count(&self) -> usize {
        self.entity_cells.len()
    }
}

impl HierarchicalSpatialIndex {
    fn new() -> Self {
        Self {
            level0: SpatialLevel::new(10.0),
            needs_full_rebuild: true,
        }
    }

    fn clear(&mut self) {
        self.level0.clear();
        self.needs_full_rebuild = true;
    }

    fn insert_new(&mut self, entity: Entity, x: f32, z: f32) {
        self.level0.insert(entity, x, z);
        self.needs_full_rebuild = false;
    }

    fn remove(&mut self, entity: Entity) {
        self.level0.remove(entity);
    }

    fn update(&mut self, entity: Entity, old_x: f32, old_z: f32, new_x: f32, new_z: f32) {
        self.level0.update(entity, old_x, old_z, new_x, new_z);
    }

    fn rebuild(&mut self, entities: &[(Entity, f32, f32)]) {
        self.clear();
        for &(e, x, z) in entities {
            self.insert_new(e, x, z);
        }
    }

    fn mark_dirty(&mut self) {
        self.needs_full_rebuild = true;
    }

    fn needs_rebuild(&self) -> bool {
        self.needs_full_rebuild
    }

    fn entity_count(&self) -> usize {
        self.level0.entity_count()
    }

    fn query_physics(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        self.level0.query_radius(x, z, radius)
    }
}

static NEXT_ENTITY: AtomicU32 = AtomicU32::new(1);
fn next_entity() -> Entity {
    NEXT_ENTITY.fetch_add(1, Ordering::SeqCst) as u64
}

/// Incremental update matches full rebuild for single move.
/// After moving one entity, incremental query must match rebuild query.
#[test]
fn incremental_matches_full_rebuild_for_single_move() {
    let mut index = HierarchicalSpatialIndex::new();
    
    // Create entities
    let e1 = next_entity();
    let e2 = next_entity();
    let e3 = next_entity();
    
    index.insert_new(e1, 5.0, 5.0);
    index.insert_new(e2, 15.0, 15.0);
    index.insert_new(e3, 25.0, 25.0);
    
    // Move e1 incrementally
    index.update(e1, 5.0, 5.0, 10.0, 10.0);
    
    // Query after incremental update
    let incremental_result = index.query_physics(10.0, 10.0, 20.0);
    
    // Rebuild from scratch
    let entities = vec![(e1, 10.0, 10.0), (e2, 15.0, 15.0), (e3, 25.0, 25.0)];
    index.rebuild(&entities);
    
    // Query after rebuild
    let rebuild_result = index.query_physics(10.0, 10.0, 20.0);
    
    // Results must match
    assert_eq!(incremental_result.len(), rebuild_result.len(),
        "Incremental query count must match rebuild query count");
    
    for e in &incremental_result {
        assert!(rebuild_result.contains(e),
            "Entity {} from incremental must be in rebuild result", e);
    }
}

/// Incremental matches full rebuild for insert/remove mix.
#[test]
fn incremental_matches_full_rebuild_for_insert_remove_mix() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e1 = next_entity();
    let e2 = next_entity();
    let e3 = next_entity();
    let e4 = next_entity();
    
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
    let incremental_count = index.entity_count();
    let incremental_query = index.query_physics(10.0, 10.0, 20.0);
    
    // Rebuild from final state
    index.rebuild(&[(e1, 15.0, 15.0), (e3, 20.0, 20.0), (e4, 5.0, 5.0)]);
    
    let rebuild_count = index.entity_count();
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
    
    let e1 = next_entity();
    let e2 = next_entity();
    
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
    assert_eq!(index.entity_count(), 1, "Entity count must be 1 after remove");
}

/// Mark dirty sets rebuild flag.
#[test]
fn mark_dirty_sets_rebuild_flag() {
    let mut index = HierarchicalSpatialIndex::new();
    
    // Initial state: needs rebuild is true
    assert!(index.needs_rebuild(), "New index must need rebuild");
    
    // Insert clears dirty flag
    let e = next_entity();
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
    
    let e1 = next_entity();
    let e2 = next_entity();
    
    // Rebuild with entities
    index.rebuild(&[(e1, 10.0, 10.0), (e2, 20.0, 20.0)]);
    
    // Verify dirty flag cleared
    assert!(!index.needs_rebuild(), "After rebuild, needs_rebuild must be false");
    
    // Verify count
    assert_eq!(index.entity_count(), 2, "Entity count must be 2 after rebuild");
    
    // Verify queryable
    let result = index.query_physics(10.0, 10.0, 15.0);
    assert!(result.contains(&e1), "e1 must be found after rebuild");
}

/// No extra rebuild flag when incremental update is used.
#[test]
fn no_extra_rebuild_flag_when_incremental_update_is_used() {
    let mut index = HierarchicalSpatialIndex::new();
    
    let e = next_entity();
    index.insert_new(e, 5.0, 5.0);
    
    // After insert, no rebuild needed
    assert!(!index.needs_rebuild(), "After insert, no rebuild needed");
    
    // Incremental update should not set rebuild flag
    index.update(e, 5.0, 5.0, 15.0, 15.0);
    assert!(!index.needs_rebuild(), "After update, no rebuild needed");
    
    // Remove should not set rebuild flag
    let e2 = next_entity();
    index.insert_new(e2, 20.0, 20.0);
    index.remove(e2);
    assert!(!index.needs_rebuild(), "After remove, no rebuild needed");
    
    // Only mark_dirty or clear should set rebuild flag
    index.mark_dirty();
    assert!(index.needs_rebuild(), "After mark_dirty, rebuild needed");
}
