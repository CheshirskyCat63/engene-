//! Engine World - Truth Layer
//!
//! OWNER: engine_world
//! PURPOSE: World state, spatial indexing, and terrain generation
//! STATUS: Production truth layer
//! 
//! This crate provides the canonical world layer including:
//! - World coordinates and chunk systems
//! - Biome and terrain generation
//! - Spatial indexing and queries
//! - Persistence interfaces
//! - Streaming state management

use std::collections::HashMap;

// Re-export streaming owner for runtime integration
pub use streaming_owner::{StreamingOwner, StreamingConfig, StreamingUpdateResult, ChunkResidency};

// Core world modules
pub mod coords;
pub mod chunk;
pub mod biome;
pub mod heightmap;
pub mod world_state;

// Spatial systems
pub mod hierarchical_spatial;
pub mod spatial_index;

// Persistence systems
pub mod chunk_persistence;
pub mod chunk_schema;

// Terrain systems
pub mod terrain;
pub mod terrain_masks;

// Component systems
pub mod components;

// Resource management
pub mod resources;

// Surface and material systems
pub mod surface_db;
pub mod material_bridge;

// Re-exports for public API
pub use coords::*;
pub use chunk::*;
pub use biome::*;
pub use heightmap::*;
pub use world_state::*;
pub use hierarchical_spatial::*;
pub use spatial_index::*;
pub use chunk_persistence::*;
pub use chunk_schema::*;
pub use terrain::*;
pub use terrain_masks::*;
pub use components::*;
pub use resources::*;
pub use surface_db::*;
pub use material_bridge::*;
