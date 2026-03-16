//! Phase 11.1: Per-material weather response profiles.

#[derive(Clone, Debug)]
pub struct WeatherResponseProfile {
    pub absorption_rate: f32,
    pub darkening_curve: f32,
    pub roughness_wet: f32,
    pub puddle_eligible: bool,
    pub drip_formation: bool,
    pub drying_time: f32,
    pub specular_boost: f32,
}

impl WeatherResponseProfile {
    pub fn soil() -> Self { Self { absorption_rate: 0.8, darkening_curve: 0.4, roughness_wet: 0.95, puddle_eligible: true, drip_formation: false, drying_time: 300.0, specular_boost: 0.02 } }
    pub fn asphalt() -> Self { Self { absorption_rate: 0.3, darkening_curve: 0.6, roughness_wet: 0.7, puddle_eligible: true, drip_formation: false, drying_time: 120.0, specular_boost: 0.05 } }
    pub fn concrete() -> Self { Self { absorption_rate: 0.2, darkening_curve: 0.7, roughness_wet: 0.75, puddle_eligible: true, drip_formation: false, drying_time: 150.0, specular_boost: 0.04 } }
    pub fn metal() -> Self { Self { absorption_rate: 0.0, darkening_curve: 0.95, roughness_wet: 0.3, puddle_eligible: false, drip_formation: true, drying_time: 30.0, specular_boost: 0.15 } }
    pub fn wood() -> Self { Self { absorption_rate: 0.5, darkening_curve: 0.5, roughness_wet: 0.8, puddle_eligible: false, drip_formation: true, drying_time: 240.0, specular_boost: 0.03 } }
    pub fn glass() -> Self { Self { absorption_rate: 0.0, darkening_curve: 1.0, roughness_wet: 0.1, puddle_eligible: false, drip_formation: true, drying_time: 15.0, specular_boost: 0.2 } }
    pub fn cloth() -> Self { Self { absorption_rate: 0.9, darkening_curve: 0.3, roughness_wet: 0.9, puddle_eligible: false, drip_formation: false, drying_time: 600.0, specular_boost: 0.01 } }
    pub fn foliage() -> Self { Self { absorption_rate: 0.1, darkening_curve: 0.8, roughness_wet: 0.6, puddle_eligible: false, drip_formation: true, drying_time: 60.0, specular_boost: 0.06 } }
}
