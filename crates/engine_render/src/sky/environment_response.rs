//! Phase 11: Surface wetness and environmental weather response.

/// Surface wetness state driven by rain and environmental conditions.
#[derive(Clone, Debug)]
pub struct WetnessState {
    pub global_wetness: f32,
    pub rain_accumulation: f32,
    pub drying_timer: f32,
}

impl Default for WetnessState {
    fn default() -> Self {
        Self {
            global_wetness: 0.0,
            rain_accumulation: 0.0,
            drying_timer: 0.0,
        }
    }
}

impl WetnessState {
    pub fn update(&mut self, rain_intensity: f32, dt: f32, temperature: f32, humidity: f32) {
        if rain_intensity > 0.0 {
            // Accumulate wetness when raining
            let accumulation_rate = rain_intensity * 0.5;
            self.rain_accumulation += accumulation_rate * dt;
            self.global_wetness = (self.global_wetness + accumulation_rate * dt * 0.2).min(1.0);
            self.drying_timer = 0.0;
        } else {
            // Dry when not raining: temperature and humidity affect drying rate
            let temp_factor = (temperature / 30.0).clamp(0.3, 1.5); // warmer = faster drying
            let humidity_factor = 1.0 - humidity; // higher humidity = slower drying
            let drying_rate = 0.2 * temp_factor * humidity_factor * dt;

            self.drying_timer += dt;

            self.global_wetness = (self.global_wetness - drying_rate).max(0.0);
            self.rain_accumulation = (self.rain_accumulation - drying_rate * 2.0).max(0.0);
        }
    }
}

/// WGSL snippet for PBR wetness modification.
pub const PBR_WETNESS_WGSL: &str = r#"
let wet_albedo = albedo * mix(1.0, darkening_curve, wetness);
let wet_roughness = mix(base_roughness, roughness_wet, wetness);
"#;
