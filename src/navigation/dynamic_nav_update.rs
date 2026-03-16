use std::collections::HashSet;
use glam::Vec3;

const MAX_DIRTY_CELLS_PER_FRAME: usize = 64;
const MAX_CLUSTER_REBUILDS_PER_FRAME: usize = 2;

pub struct NavDirtyTracker {
    dirty_cells: HashSet<(i32, i32)>,
    pending_cluster_rebuilds: Vec<u32>,
}

impl NavDirtyTracker {
    pub fn new() -> Self {
        Self {
            dirty_cells: HashSet::new(),
            pending_cluster_rebuilds: Vec::new(),
        }
    }

    pub fn mark_dirty(&mut self, cell_x: i32, cell_z: i32) {
        self.dirty_cells.insert((cell_x, cell_z));
    }

    pub fn mark_area_dirty(&mut self, center: Vec3, radius: f32, cell_size: f32) {
        let r = (radius / cell_size).ceil() as i32;
        let cx = (center.x / cell_size).floor() as i32;
        let cz = (center.z / cell_size).floor() as i32;

        for dx in -r..=r {
            for dz in -r..=r {
                self.dirty_cells.insert((cx + dx, cz + dz));
            }
        }
    }

    pub fn mark_cluster_rebuild(&mut self, cluster_id: u32) {
        if !self.pending_cluster_rebuilds.contains(&cluster_id) {
            self.pending_cluster_rebuilds.push(cluster_id);
        }
    }

    pub fn drain_dirty_batch(&mut self) -> Vec<(i32, i32)> {
        let count = self.dirty_cells.len().min(MAX_DIRTY_CELLS_PER_FRAME);
        let batch: Vec<_> = self.dirty_cells.iter().copied().take(count).collect();
        for cell in &batch {
            self.dirty_cells.remove(cell);
        }
        batch
    }

    pub fn drain_cluster_rebuilds(&mut self) -> Vec<u32> {
        let count = self.pending_cluster_rebuilds.len().min(MAX_CLUSTER_REBUILDS_PER_FRAME);
        self.pending_cluster_rebuilds.drain(..count).collect()
    }

    pub fn pending_count(&self) -> usize {
        self.dirty_cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dirty_cells.is_empty() && self.pending_cluster_rebuilds.is_empty()
    }
}
