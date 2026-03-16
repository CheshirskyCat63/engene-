//! Phase 12.5: Weather audio state hooks for AudioEngine.

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RainSurface {
    Metal,
    Leaves,
    Concrete,
    Water,
    Glass,
    Soil,
}

pub struct ThunderEvent {
    pub position: Vec3,
    pub intensity: f32,
    pub delay_seconds: f32,
}

pub struct WindPortal {
    pub opening_area: f32,
    pub wind_alignment: f32,
}

pub struct WeatherAudioState {
    pub rain_intensity: f32,
    pub rain_surface: RainSurface,
    pub thunder_events: Vec<ThunderEvent>,
    pub wind_speed: f32,
    pub wind_portal: Option<WindPortal>,
    pub is_indoor: bool,
    pub shelter_factor: f32,
}

impl Default for WeatherAudioState {
    fn default() -> Self {
        Self {
            rain_intensity: 0.0,
            rain_surface: RainSurface::Concrete,
            thunder_events: Vec::new(),
            wind_speed: 0.0,
            wind_portal: None,
            is_indoor: false,
            shelter_factor: 0.0,
        }
    }
}
