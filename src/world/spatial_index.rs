use crate::core::ecs::Entity;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};

pub struct SpatialIndex {
    cells: Vec<Vec<Entity>>,
    grid_w: u32,
}

impl SpatialIndex {
    pub fn new() -> Self {
        let n = (GRID_SIZE * GRID_SIZE) as usize;
        Self {
            cells: vec![Vec::new(); n],
            grid_w: GRID_SIZE,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let idx = self.pos_to_idx(x, y);
        self.cells[idx].push(entity);
    }

    pub fn candidates_in_radius(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let cells_r = (radius / CELL_SIZE).ceil() as i32;
        let cx = (x / CELL_SIZE) as i32;
        let cy = (y / CELL_SIZE) as i32;
        let w = self.grid_w as i32;
        let estimated = ((2 * cells_r + 1) * (2 * cells_r + 1)) as usize * 4;
        let mut result = Vec::with_capacity(estimated);
        for dy in -cells_r..=cells_r {
            for dx in -cells_r..=cells_r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && ny >= 0 && nx < w && ny < w {
                    let idx = (ny * w + nx) as usize;
                    result.extend_from_slice(&self.cells[idx]);
                }
            }
        }
        result
    }

    fn pos_to_idx(&self, x: f32, y: f32) -> usize {
        let cx = ((x / CELL_SIZE) as u32).min(self.grid_w - 1);
        let cy = ((y / CELL_SIZE) as u32).min(self.grid_w - 1);
        (cy * self.grid_w + cx) as usize
    }
}
