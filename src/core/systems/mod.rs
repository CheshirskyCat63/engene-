//! Core game systems extracted from main.rs (Phase A.4).
//!
//! Each system is a self-contained unit that can be registered with the engine.

mod streaming_system;
mod spatial_rebuild_system;
mod audio_listener_system;
mod render_system;

pub use streaming_system::StreamingSystem;
pub use spatial_rebuild_system::SpatialRebuildSystem;
pub use audio_listener_system::AudioListenerSystem;
pub use render_system::{RenderSystem, EntityInstanceCollector};
