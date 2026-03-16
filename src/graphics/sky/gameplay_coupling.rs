//! Phase 13.9: Weather gameplay state for AI, ballistics, movement, stealth.

use glam::Vec3;

pub struct WeatherGameplayState {
    pub surface_friction_mult: f32,
    pub fog_visibility_mult: f32,
    pub rain_audio_masking: f32,
    pub ambient_light_level: f32,
    pub wind_vector: Vec3,
    pub wind_speed: f32,
    pub lightning_flash_active: bool,
    pub thunder_recent: bool,
    pub global_wetness: f32,
    pub rain_intensity: f32,
}

impl Default for WeatherGameplayState {
    fn default() -> Self {
        Self {
            surface_friction_mult: 1.0,
            fog_visibility_mult: 1.0,
            rain_audio_masking: 0.0,
            ambient_light_level: 1.0,
            wind_vector: Vec3::ZERO,
            wind_speed: 0.0,
            lightning_flash_active: false,
            thunder_recent: false,
            global_wetness: 0.0,
            rain_intensity: 0.0,
        }
    }
}
