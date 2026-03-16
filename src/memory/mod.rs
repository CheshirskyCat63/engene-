//! Memory subsystem - asset management, entity pooling, and memory tracking.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit
//!
//! ## Modules
//! - `asset_manager`, `save_chunks` - production, asset persistence
//! - `entity_pool`, `memory_manager` - production, memory management
//! - `streaming_cache` - production, chunk caching
//! - `atomic_saved` - production, crash-safe save operations

pub mod asset_manager;
pub mod atomic_saved;
pub mod component_delta;
pub mod entity_pool;
pub mod memory_manager;
pub mod save_chunks;
pub mod streaming_cache;
