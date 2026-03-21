//! Engine World - Truth Layer (DATA ONLY)
//!
//! OWNER: engine_world
//! PURPOSE: World data contracts only
//! STATUS: Data layer - NO runtime logic, NO orchestration
//! 
//! This crate provides world DATA layer ONLY:
//! - World coordinates
//! - Chunk data structures
//! - World state contracts
//! - Terrain data
//!
//! WHAT THIS IS NOT:
//! - NOT runtime phase execution
//! - NOT streaming orchestration
//! - NOT persistence orchestration
//! - NOT game logic

// ================================================================================
// CANONICAL PUBLIC API - DATA ONLY
// ================================================================================
pub mod coords;
pub mod chunk;
pub mod world_state;
pub mod terrain;

// ================================================================================
// QUARANTINE - NOT EXPORTED - CHECKING FOR HIDDEN CONSUMERS
// These modules exist but are NOT public API. They are kept for
// potential consumer audit. If no consumers found in 1 week, delete.
// ================================================================================
// NOT exported: chunk_persistence, streaming_owner, persistence, world.rs

mod chunk_persistence_quarantine {
    // pub mod chunk_persistence; // COMMENTED - checking for consumers
    // pub mod streaming_owner;    // COMMENTED - checking for consumers
}
