use glam::Vec3;
use std::collections::HashSet;

const MAX_COVER_UPDATES_PER_FRAME: usize = 32;

pub struct CoverRefreshQueue {
    dirty_cells: HashSet<(i32, i32)>,
}

impl CoverRefreshQueue {
    pub fn new() -> Self {
        Self {
            dirty_cells: HashSet::new(),
        }
    }

    pub fn mark_dirty(&mut self, cell_x: i32, cell_z: i32) {
        self.dirty_cells.insert((cell_x, cell_z));
    }

    pub fn mark_area(&mut self, center: Vec3, radius: f32, cell_size: f32) {
        let r = (radius / cell_size).ceil() as i32;
        let cx = (center.x / cell_size).floor() as i32;
        let cz = (center.z / cell_size).floor() as i32;
        for dx in -r..=r {
            for dz in -r..=r {
                self.dirty_cells.insert((cx + dx, cz + dz));
            }
        }
    }

    pub fn drain_batch(&mut self) -> Vec<(i32, i32)> {
        let count = self.dirty_cells.len().min(MAX_COVER_UPDATES_PER_FRAME);
        let batch: Vec<_> = self.dirty_cells.iter().copied().take(count).collect();
        for cell in &batch {
            self.dirty_cells.remove(cell);
        }
        batch
    }

    pub fn is_empty(&self) -> bool {
        self.dirty_cells.is_empty()
    }
}
