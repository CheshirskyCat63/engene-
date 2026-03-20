// World cell constants - migrated from root src/world/cell.rs

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
