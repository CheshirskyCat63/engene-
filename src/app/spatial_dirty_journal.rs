use crate::core::ecs::Entity;
use crate::world::hierarchical_spatial::SpatialDirtyInput;

#[derive(Debug, Default, Clone)]
pub struct SpatialDirtyJournal {
    pub inserted: Vec<Entity>,
    pub moved: Vec<Entity>,
    pub removed: Vec<Entity>,
    pub force_rebuild: bool,
    pub editor_mutation: bool,
    pub chunk_structural_change: bool,
}

impl SpatialDirtyJournal {
    pub fn mark_inserted(&mut self, e: Entity) {
        self.inserted.push(e);
    }

    pub fn mark_moved(&mut self, e: Entity) {
        self.moved.push(e);
    }

    pub fn mark_removed(&mut self, e: Entity) {
        self.removed.push(e);
    }

    pub fn mark_editor_mutation(&mut self) {
        self.editor_mutation = true;
    }

    pub fn mark_chunk_structural_change(&mut self) {
        self.chunk_structural_change = true;
    }

    pub fn mark_force_rebuild(&mut self) {
        self.force_rebuild = true;
    }

    pub fn clear(&mut self) {
        self.inserted.clear();
        self.moved.clear();
        self.removed.clear();
        self.force_rebuild = false;
        self.editor_mutation = false;
        self.chunk_structural_change = false;
    }

    pub fn is_empty(&self) -> bool {
        self.inserted.is_empty()
            && self.moved.is_empty()
            && self.removed.is_empty()
            && !self.force_rebuild
            && !self.editor_mutation
            && !self.chunk_structural_change
    }

    pub fn to_input(&self) -> SpatialDirtyInput {
        SpatialDirtyInput {
            inserted: self.inserted.clone(),
            moved: self.moved.clone(),
            removed: self.removed.clone(),
            force_rebuild: self.force_rebuild,
            editor_mutation: self.editor_mutation,
            chunk_structural_change: self.chunk_structural_change,
        }
    }
}
