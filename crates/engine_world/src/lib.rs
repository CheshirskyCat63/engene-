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

// Core world modules
pub mod coords;
pub mod chunk;
pub mod world_state;
pub mod terrain;

// Streaming subsystem
pub mod streaming;

// Re-exports for public API
pub use coords::*;
pub use chunk::*;
pub use world_state::*;
pub use terrain::*;
pub use streaming::*;
