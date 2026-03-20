use crate::core::ecs::Entity;
use std::collections::HashMap;

const LEVEL0_CELL: f32 = 10.0;
const LEVEL1_CELL: f32 = 100.0;
const LEVEL2_CELL: f32 = 1000.0;

fn cell_key(x: f32, z: f32, cell_size: f32) -> (i32, i32) {
    (
        (x / cell_size).floor() as i32,
        (z / cell_size).floor() as i32,
    )
}

struct SpatialLevel {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<(Entity, f32, f32)>>,
    /// Entity -> current cell key (for O(1) updates)
    entity_cells: HashMap<Entity, (i32, i32)>,
}

impl SpatialLevel {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    fn insert(&mut self, entity: Entity, x: f32, z: f32) {
        let key = cell_key(x, z, self.cell_size);

        // Remove from old cell if exists
        if let Some(old_key) = self.entity_cells.get(&entity) {
            if *old_key != key {
                if let Some(cell) = self.cells.get_mut(old_key) {
                    cell.retain(|(e, _, _)| *e != entity);
                }
            }
        }

        // Insert into new cell
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
        let old_key = cell_key(old_x, old_z, self.cell_size);
        let new_key = cell_key(new_x, new_z, self.cell_size);

        if old_key == new_key {
            // Same cell, just update position
            if let Some(cell) = self.cells.get_mut(&old_key) {
                for (_, x, z) in cell.iter_mut() {
                    if *x == old_x && *z == old_z {
                        *x = new_x;
                        *z = new_z;
                        break;
                    }
                }
            }
        } else {
            // Different cell, remove from old, insert into new
            if let Some(cell) = self.cells.get_mut(&old_key) {
                cell.retain(|(e, _, _)| *e != entity);
            }
            self.cells
                .entry(new_key)
                .or_default()
                .push((entity, new_x, new_z));
            self.entity_cells.insert(entity, new_key);
        }
    }

    fn query_radius(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        let r2 = radius * radius;
        let min_key = cell_key(x - radius, z - radius, self.cell_size);
        let max_key = cell_key(x + radius, z + radius, self.cell_size);

        let mut result = Vec::new();
        for cy in min_key.1..=max_key.1 {
            for cx in min_key.0..=max_key.0 {
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

pub struct HierarchicalSpatialIndex {
    level0: SpatialLevel,
    level1: SpatialLevel,
    level2: SpatialLevel,
    /// Track if we need full rebuild (e.g., after origin shift)
    needs_full_rebuild: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SpatialDirtyInput {
    pub inserted: Vec<Entity>,
    pub moved: Vec<Entity>,
    pub removed: Vec<Entity>,
    pub force_rebuild: bool,
    pub editor_mutation: bool,
    pub chunk_structural_change: bool,
}

impl SpatialDirtyInput {
    pub fn mark_editor_mutation(&mut self) {
        self.editor_mutation = true;
    }

    pub fn mark_chunk_structural_change(&mut self) {
        self.chunk_structural_change = true;
    }

    pub fn mark_origin_shift(&mut self) {
        self.force_rebuild = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialUpdatePath {
    NoWork,
    Incremental,
    FullRebuild,
}

pub fn select_spatial_update_path(dirty: &SpatialDirtyInput) -> SpatialUpdatePath {
    if dirty.force_rebuild || dirty.chunk_structural_change {
        SpatialUpdatePath::FullRebuild
    } else if dirty.inserted.is_empty()
        && dirty.moved.is_empty()
        && dirty.removed.is_empty()
        && !dirty.editor_mutation
    {
        SpatialUpdatePath::NoWork
    } else {
        SpatialUpdatePath::Incremental
    }
}

impl HierarchicalSpatialIndex {
    pub fn new() -> Self {
        Self {
            level0: SpatialLevel::new(LEVEL0_CELL),
            level1: SpatialLevel::new(LEVEL1_CELL),
            level2: SpatialLevel::new(LEVEL2_CELL),
            needs_full_rebuild: true,
        }
    }

    pub fn clear(&mut self) {
        self.level0.clear();
        self.level1.clear();
        self.level2.clear();
        self.needs_full_rebuild = true;
    }

    /// Insert a new entity (for spawn)
    pub fn insert_new(&mut self, entity: Entity, x: f32, z: f32) {
        self.level0.insert(entity, x, z);
        self.level1.insert(entity, x, z);
        self.level2.insert(entity, x, z);
        self.needs_full_rebuild = false;
    }

    /// Remove an entity (for despawn)
    pub fn remove(&mut self, entity: Entity) {
        self.level0.remove(entity);
        self.level1.remove(entity);
        self.level2.remove(entity);
    }

    /// Update entity position (incremental, O(1) for same cell, O(2) for cell change)
    pub fn update(&mut self, entity: Entity, old_x: f32, old_z: f32, new_x: f32, new_z: f32) {
        self.level0.update(entity, old_x, old_z, new_x, new_z);
        self.level1.update(entity, old_x, old_z, new_x, new_z);
        self.level2.update(entity, old_x, old_z, new_x, new_z);
    }

    /// Legacy insert (calls insert_new internally)
    pub fn insert(&mut self, entity: Entity, x: f32, z: f32) {
        self.insert_new(entity, x, z);
    }

    pub fn query_physics(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        self.level0.query_radius(x, z, radius)
    }

    pub fn query_ai(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        self.level1.query_radius(x, z, radius)
    }

    pub fn query_world(&self, x: f32, z: f32, radius: f32) -> Vec<Entity> {
        self.level2.query_radius(x, z, radius)
    }

    /// Full rebuild (use only for origin shift or chunk load)
    pub fn rebuild(&mut self, entities: &[(Entity, f32, f32)]) {
        self.clear();
        for &(e, x, z) in entities {
            self.insert_new(e, x, z);
        }
    }

    /// Mark for full rebuild on next frame
    pub fn mark_dirty(&mut self) {
        self.needs_full_rebuild = true;
    }

    /// Check if full rebuild is needed
    pub fn needs_rebuild(&self) -> bool {
        self.needs_full_rebuild
    }

    /// Get entity count at each level
    pub fn entity_count(&self) -> (usize, usize, usize) {
        (
            self.level0.entity_count(),
            self.level1.entity_count(),
            self.level2.entity_count(),
        )
    }
}

impl Default for HierarchicalSpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut index = HierarchicalSpatialIndex::new();
        let e1: Entity = 1;
        let e2: Entity = 2;

        index.insert_new(e1, 5.0, 5.0);
        index.insert_new(e2, 15.0, 15.0);

        let result = index.query_physics(5.0, 5.0, 10.0);
        assert!(result.contains(&e1));
        assert!(!result.contains(&e2));
    }

    #[test]
    fn test_update_same_cell() {
        let mut index = HierarchicalSpatialIndex::new();
        let e: Entity = 1;

        index.insert_new(e, 5.0, 5.0);
        index.update(e, 5.0, 5.0, 7.0, 7.0);

        let result = index.query_physics(7.0, 7.0, 5.0);
        assert!(result.contains(&e));
    }

    #[test]
    fn test_update_different_cell() {
        let mut index = HierarchicalSpatialIndex::new();
        let e: Entity = 1;

        index.insert_new(e, 5.0, 5.0);
        index.update(e, 5.0, 5.0, 25.0, 25.0);

        let result = index.query_physics(25.0, 25.0, 5.0);
        assert!(result.contains(&e));

        let old_result = index.query_physics(5.0, 5.0, 5.0);
        assert!(!old_result.contains(&e));
    }

    #[test]
    fn test_remove() {
        let mut index = HierarchicalSpatialIndex::new();
        let e: Entity = 1;

        index.insert_new(e, 5.0, 5.0);
        index.remove(e);

        let result = index.query_physics(5.0, 5.0, 10.0);
        assert!(!result.contains(&e));
    }

    #[test]
    fn test_rebuild() {
        let mut index = HierarchicalSpatialIndex::new();
        let e1: Entity = 1;
        let e2: Entity = 2;

        index.rebuild(&[(e1, 10.0, 10.0), (e2, 20.0, 20.0)]);

        let (c0, c1, c2) = index.entity_count();
        assert_eq!(c0, 2);
        assert_eq!(c1, 2);
        assert_eq!(c2, 2);
    }
}
