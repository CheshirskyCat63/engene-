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
//!
//! NOTE: Streaming moved to engine_runtime::phase::streaming

// Core world modules
pub mod coords;
pub mod chunk;
pub mod world_state;
pub mod terrain;
pub mod chunk_persistence;
pub mod streaming_owner;

// Re-exports for public API
pub use coords::*;
pub use chunk::*;
pub use world_state::*;
pub use terrain::*;
pub use chunk_persistence::*;
pub use streaming_owner::*;
