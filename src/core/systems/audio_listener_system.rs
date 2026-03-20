//! Audio listener update system.

use engine_audio::audio::AudioEngine;
use glam::Vec3;

/// Updates audio engine listener position and orientation.
pub struct AudioListenerSystem;

impl AudioListenerSystem {
    /// Update audio listener from camera.
    pub fn update(audio: &mut AudioEngine, cam_pos: Vec3, cam_fwd: Vec3, dt: f32) {
        audio.set_listener(cam_pos, cam_fwd);
        audio.update(dt);
    }
}
