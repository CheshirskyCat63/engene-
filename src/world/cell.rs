use crate::core::ecs::Entity;
use crate::world::biome::Biome;

/// Contract 9: World Scale — 40x40 cells * 50m = 2000m = 2km per axis
pub const GRID_SIZE: u32 = 40;
pub const CELL_SIZE: f32 = 50.0;
pub const WORLD_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE; // 2000.0m = 2km

/// Chunk: 4x4 cells = 200m x 200m. Total: 10x10 = 100 chunks.
pub const CHUNK_CELLS: u32 = 4;
pub const CHUNK_SIZE_M: f32 = CHUNK_CELLS as f32 * CELL_SIZE; // 200.0m

/// Budgets (Contract 9)
pub const ACTIVE_LOADED_RADIUS_CHUNKS: u32 = 3; // 600m
pub const PHYSICS_BUBBLE_RADIUS: f32 = 200.0;
pub const MAX_DRAW_CALLS: u32 = 100;
pub const MAX_TRIANGLES: u32 = 500_000;
pub const MAX_TRIANGLES_LOW_SPEC: u32 = 200_000;

#[derive(Clone, Debug)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
    pub biome: Biome,
    pub food: f32,
    pub danger: f32,
    pub entities: Vec<Entity>,
}

impl Cell {
    pub fn new(x: u32, y: u32, biome: Biome) -> Self {
        Self {
            x,
            y,
            biome,
            food: biome.food_density(),
            danger: biome.danger_level(),
            entities: Vec::new(),
        }
    }

    pub fn world_pos(&self) -> (f32, f32) {
        (
            self.x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
            self.y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        )
    }

    pub fn regenerate_food(&mut self, delta: f32) {
        let cap = self.biome.food_density();
        self.food = (self.food + delta * 0.005).min(cap);
    }
}

pub fn cell_index(x: u32, y: u32) -> usize {
    (y * GRID_SIZE + x) as usize
}

pub fn pos_to_cell(x: f32, y: f32) -> (u32, u32) {
    let cx = ((x / CELL_SIZE) as u32).min(GRID_SIZE - 1);
    let cy = ((y / CELL_SIZE) as u32).min(GRID_SIZE - 1);
    (cx, cy)
}
