//! Engine World - MINIMUM TRUTH ONLY
//! 
//! Only what engine path needs:
//! - Basic world data/types
//! - Spatial/state surfaces for core needs
//! - No content carnival

pub mod api {
    pub const CRATE: &str = "engine_world";
}

// MINIMUM WORLD TRUTH
pub mod cell {
    /// Basic cell data for engine needs
    #[derive(Debug, Clone, Copy)]
    pub struct Cell {
        pub position: (i32, i32),
        pub biome_id: u8,
    }
    
    pub const CELL_SIZE: f32 = 1.0;
    pub const GRID_SIZE: usize = 100;
}

pub mod world {
    use super::cell::Cell;
    
    /// Minimal world grid for engine path
    #[derive(Debug)]
    pub struct WorldGrid {
        pub cells: Vec<Cell>,
    }
    
    impl WorldGrid {
        pub fn new() -> Self {
            Self { cells: Vec::new() }
        }
        
        pub fn generate() -> Self {
            let mut world = Self::new();
            // Generate some cells for engine path
            for x in 0..10 {
                for y in 0..10 {
                    world.cells.push(Cell {
                        position: (x, y),
                        biome_id: 0,
                    });
                }
            }
            world
        }
    }
}

// Re-exports for engine path
pub use cell::{Cell, CELL_SIZE, GRID_SIZE};
pub use world::WorldGrid;
