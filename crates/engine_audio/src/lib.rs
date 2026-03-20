pub mod api {
    /// Phase A facade placeholder for engine_audio.
    pub const CRATE: &str = "engine_audio";
}

pub mod audio;

// Public facade exports
pub use audio::{AudioEngine, SoundHandle, SoundKind};
