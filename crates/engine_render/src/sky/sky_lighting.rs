//! Phase 2.5: Dynamic sky irradiance, sun+sky energy balance, moonlight ambient.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct WeatherLightingParams {
    pub sun_attenuation: f32,
    pub sky_diffuse_boost: f32,
    pub sky_color_temp: f32,
    pub lightning_flash: f32,
    pub rain_aerial_density: f32,
    pub moon_ambient: f32,
    pub _pad: [f32; 2],
}

impl Default for WeatherLightingParams {
    fn default() -> Self {
        Self {
            sun_attenuation: 1.0,
            sky_diffuse_boost: 1.0,
            sky_color_temp: 6500.0,
            lightning_flash: 0.0,
            rain_aerial_density: 0.0,
            moon_ambient: 0.0,
            _pad: [0.0; 2],
        }
    }
}

/// Phase 2.5: Computes weather lighting from atmosphere + weather state.
pub struct SkyLightingSystem;

impl SkyLightingSystem {
    pub fn new() -> Self {
        Self
    }

    /// Update lighting params from sun direction, cloud coverage, rain, lightning, moon, and day progress.
    pub fn update(
        &mut self,
        sun_dir: Vec3,
        cloud_coverage: f32,
        rain_intensity: f32,
        lightning_flash: f32,
        moon_elevation: f32,
        _day_progress: f32,
    ) -> WeatherLightingParams {
        let sun_attenuation = sun_dir.y.clamp(0.0, 1.0);
        let sky_diffuse_boost = 1.0 + cloud_coverage * 0.5;
        let sky_color_temp = 6500.0 + cloud_coverage * 1500.0;
        let rain_aerial_density = rain_intensity * 0.5;
        let moon_ambient = moon_elevation.max(0.0) * 0.01 * (1.0 - cloud_coverage);

        WeatherLightingParams {
            sun_attenuation,
            sky_diffuse_boost,
            sky_color_temp,
            lightning_flash,
            rain_aerial_density,
            moon_ambient,
            _pad: [0.0; 2],
        }
    }
}
