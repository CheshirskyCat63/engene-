//! Spatial index rebuild system.

use crate::core::ecs::Ecs;
use crate::world::hierarchical_spatial::HierarchicalSpatialIndex;

/// Rebuilds hierarchical spatial index each frame.
pub struct SpatialRebuildSystem;

impl SpatialRebuildSystem {
    /// Rebuild spatial index from current entity positions.
    pub fn rebuild(spatial: &mut HierarchicalSpatialIndex, ecs: &Ecs) {
        spatial.clear();
        for &e in &ecs.alive {
            if let Some(t) = ecs.transforms.get(&e) {
                spatial.insert(e, t.x, t.y);
            }
        }
    }
}
