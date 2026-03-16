//! Phase 6.5: Vegetation physical response to weather.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct VegetationWeatherParams {
    pub wetness: f32,
    pub wind_gust_strength: f32,
    pub mass_multiplier: f32,
    pub damping_multiplier: f32,
}

impl Default for VegetationWeatherParams {
    fn default() -> Self {
        Self {
            wetness: 0.0,
            wind_gust_strength: 0.0,
            mass_multiplier: 1.0,
            damping_multiplier: 1.0,
        }
    }
}

impl VegetationWeatherParams {
    pub fn from_weather(rain_intensity: f32, wind_speed: f32) -> Self {
        let wetness = rain_intensity.clamp(0.0, 1.0);
        Self {
            wetness,
            wind_gust_strength: wind_speed,
            mass_multiplier: 1.0 + wetness * 0.5,
            damping_multiplier: 1.0 + wetness * 1.5,
        }
    }
}
