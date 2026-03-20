//! Phase 10: Climate zone system for localized weather modulation.

use glam::Vec3;

pub struct ClimateZone {
    pub center: Vec3,
    pub radius: f32,
    pub base_humidity: f32,
    pub base_temperature: f32,
    pub wind_bias: Vec3,
    pub fog_density_mult: f32,
    pub precipitation_mult: f32,
    pub storm_probability: f32,
}

pub struct ClimateSystem {
    pub zones: Vec<ClimateZone>,
}

impl ClimateSystem {
    pub fn chernobyl_default() -> Self {
        Self {
            zones: vec![
                ClimateZone {
                    center: Vec3::new(0.0, 0.0, 0.0),
                    radius: 5000.0,
                    base_humidity: 0.7,
                    base_temperature: 18.0,
                    wind_bias: Vec3::new(1.0, 0.0, 0.5),
                    fog_density_mult: 1.5,
                    precipitation_mult: 1.2,
                    storm_probability: 0.15,
                },
                ClimateZone {
                    center: Vec3::new(3000.0, 0.0, 0.0),
                    radius: 3000.0,
                    base_humidity: 0.85,
                    base_temperature: 16.0,
                    wind_bias: Vec3::new(0.5, 0.0, 0.2),
                    fog_density_mult: 2.5,
                    precipitation_mult: 1.0,
                    storm_probability: 0.05,
                },
                ClimateZone {
                    center: Vec3::new(-2000.0, 0.0, 2000.0),
                    radius: 4000.0,
                    base_humidity: 0.4,
                    base_temperature: 22.0,
                    wind_bias: Vec3::new(3.0, 0.0, 1.0),
                    fog_density_mult: 0.5,
                    precipitation_mult: 0.8,
                    storm_probability: 0.1,
                },
            ],
        }
    }

    pub fn sample_at(&self, pos: Vec3) -> ClimateBlend {
        let mut humidity = 0.5;
        let mut temperature = 20.0;
        let mut fog_mult = 1.0;
        let mut total_weight = 0.0;

        for zone in &self.zones {
            let dist = (pos - zone.center).length();
            if dist > zone.radius {
                continue;
            }
            let w = 1.0 - dist / zone.radius;
            let w = w * w;
            humidity += zone.base_humidity * w;
            temperature += zone.base_temperature * w;
            fog_mult += zone.fog_density_mult * w;
            total_weight += w;
        }

        if total_weight > 0.0 {
            humidity /= total_weight + 1.0;
            temperature /= total_weight + 1.0;
            fog_mult /= total_weight + 1.0;
        }

        ClimateBlend {
            humidity,
            temperature,
            fog_mult,
        }
    }
}

pub struct ClimateBlend {
    pub humidity: f32,
    pub temperature: f32,
    pub fog_mult: f32,
}
