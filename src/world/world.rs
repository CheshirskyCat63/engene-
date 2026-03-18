use rand::Rng;

use crate::world::biome::Biome;
use crate::world::cell::{Cell, GRID_SIZE};

pub struct WorldGrid {
    pub cells: Vec<Cell>,
}

impl WorldGrid {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = Vec::with_capacity((GRID_SIZE * GRID_SIZE) as usize);

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let center = GRID_SIZE / 2;
                let biome =
                    if x >= center - 1 && x <= center + 1 && y >= center - 1 && y <= center + 1 {
                        Biome::Settlement
                    } else {
                        match rng.gen_range(0u8..10) {
                            0..=3 => Biome::Forest,
                            4..=6 => Biome::Plains,
                            7..=8 => Biome::Hills,
                            _ => Biome::Swamp,
                        }
                    };
                cells.push(Cell::new(x, y, biome));
            }
        }

        Self { cells }
    }

    pub fn get(&self, x: u32, y: u32) -> &Cell {
        &self.cells[crate::world::cell::cell_index(x, y)]
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        let idx = crate::world::cell::cell_index(x, y);
        &mut self.cells[idx]
    }

    pub fn regenerate_food(&mut self, delta: f32) {
        for cell in &mut self.cells {
            cell.regenerate_food(delta);
        }
    }

    pub fn regenerate_food_scaled(&mut self, delta: f32, season_mult: f32) {
        for cell in &mut self.cells {
            cell.regenerate_food(delta * season_mult);
        }
    }
}
