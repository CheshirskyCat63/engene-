//! Audio subsystem - sound playback, spatial audio, and occlusion.
//!
//! # Status: partial
//! # Integration: enabled
//! # Tests: integration only
//!
//! ## Modules
//! - `audio`, `playback`, `spatial_audio` - partial, core audio
//! - `occlusion` - partial, sound occlusion
//! - `sound_bank` - partial, asset management

pub mod ambience;
pub mod audio;
pub mod audio_debug;
pub mod audio_integration;
pub mod occlusion;
pub mod playback;
pub mod sound_bank;
pub mod spatial_audio;
