//! World subsystem - terrain, streaming, persistence, and spatial indexing.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit + integration
//!
//! ## Modules
//! - `world`, `heightmap`, `biome` - production, world generation
//! - `streaming`, `chunk_persistence` - production, chunk management
//! - `hierarchical_spatial`, `spatial_index` - production, spatial queries
//! - `components` - production, ECS components (split into sub-modules)
//! - `surface_db`, `material_bridge` - partial, material truth

pub mod api {
    /// Stable crate identifier.
    pub const CRATE: &str = "engine_world";
}

pub mod authored_sets;
pub mod authoring;
pub mod biome;
pub mod cell;
pub mod chunk_package;
pub mod chunk_persistence;
pub mod chunk_schema;
pub mod components;
pub mod damage_profiles;
pub mod extension_components;
pub mod fields;
pub mod heightmap;
pub mod hierarchical_spatial;
pub mod io_budget;
pub mod material_bridge;
pub mod origin_shift;
pub mod persistence;
pub mod population;
pub mod resources;
pub mod spatial_index;
pub mod streaming;
pub mod surface_db;
pub mod surface_state;
pub mod terrain_damage;
pub mod terrain_deformation;
pub mod terrain_masks;
pub mod terrain_truth;
pub mod world;

// Re-export main types
pub use cell::{Cell, CELL_SIZE, GRID_SIZE, WORLD_SIZE};
pub use fields::WorldFields;
pub use world::WorldGrid;

