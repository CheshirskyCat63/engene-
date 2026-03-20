use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtDirectionProfile {
    pub name: String,
    pub fog_near: f32,
    pub fog_far: f32,
    pub fog_color_day: [f32; 3],
    pub fog_color_night: [f32; 3],
    pub fog_density: f32,
    pub ambient_intensity_day: f32,
    pub ambient_intensity_night: f32,
    pub sun_intensity: f32,
    pub shadow_softness: f32,
    pub exposure: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
}

impl ArtDirectionProfile {
    pub fn default_outdoor() -> Self {
        Self {
            name: "Outdoor Default".into(),
            fog_near: 200.0,
            fog_far: 2000.0,
            fog_color_day: [0.7, 0.75, 0.8],
            fog_color_night: [0.05, 0.05, 0.1],
            fog_density: 0.002,
            ambient_intensity_day: 0.3,
            ambient_intensity_night: 0.05,
            sun_intensity: 2.0,
            shadow_softness: 0.5,
            exposure: 1.0,
            saturation: 1.0,
            contrast: 1.1,
            bloom_threshold: 1.5,
            bloom_intensity: 0.3,
        }
    }

    pub fn swamp() -> Self {
        Self {
            name: "Swamp".into(),
            fog_near: 50.0,
            fog_far: 500.0,
            fog_color_day: [0.5, 0.55, 0.45],
            fog_color_night: [0.03, 0.04, 0.03],
            fog_density: 0.01,
            ambient_intensity_day: 0.2,
            ambient_intensity_night: 0.03,
            sun_intensity: 1.2,
            shadow_softness: 0.8,
            exposure: 0.9,
            saturation: 0.8,
            contrast: 0.95,
            bloom_threshold: 2.0,
            bloom_intensity: 0.2,
        }
    }

    pub fn anomaly_zone() -> Self {
        Self {
            name: "Anomaly Zone".into(),
            fog_near: 30.0,
            fog_far: 300.0,
            fog_color_day: [0.6, 0.4, 0.3],
            fog_color_night: [0.1, 0.05, 0.02],
            fog_density: 0.015,
            ambient_intensity_day: 0.25,
            ambient_intensity_night: 0.08,
            sun_intensity: 1.5,
            shadow_softness: 0.6,
            exposure: 1.1,
            saturation: 0.7,
            contrast: 1.2,
            bloom_threshold: 1.0,
            bloom_intensity: 0.5,
        }
    }

    pub fn underground() -> Self {
        Self {
            name: "Underground".into(),
            fog_near: 10.0,
            fog_far: 100.0,
            fog_color_day: [0.02, 0.02, 0.03],
            fog_color_night: [0.02, 0.02, 0.03],
            fog_density: 0.03,
            ambient_intensity_day: 0.02,
            ambient_intensity_night: 0.02,
            sun_intensity: 0.0,
            shadow_softness: 0.3,
            exposure: 1.5,
            saturation: 0.6,
            contrast: 1.3,
            bloom_threshold: 0.8,
            bloom_intensity: 0.4,
        }
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        let lerp3 =
            |a: [f32; 3], b: [f32; 3]| [lerp(a[0], b[0]), lerp(a[1], b[1]), lerp(a[2], b[2])];
        Self {
            name: format!("blend({}, {})", self.name, other.name),
            fog_near: lerp(self.fog_near, other.fog_near),
            fog_far: lerp(self.fog_far, other.fog_far),
            fog_color_day: lerp3(self.fog_color_day, other.fog_color_day),
            fog_color_night: lerp3(self.fog_color_night, other.fog_color_night),
            fog_density: lerp(self.fog_density, other.fog_density),
            ambient_intensity_day: lerp(self.ambient_intensity_day, other.ambient_intensity_day),
            ambient_intensity_night: lerp(
                self.ambient_intensity_night,
                other.ambient_intensity_night,
            ),
            sun_intensity: lerp(self.sun_intensity, other.sun_intensity),
            shadow_softness: lerp(self.shadow_softness, other.shadow_softness),
            exposure: lerp(self.exposure, other.exposure),
            saturation: lerp(self.saturation, other.saturation),
            contrast: lerp(self.contrast, other.contrast),
            bloom_threshold: lerp(self.bloom_threshold, other.bloom_threshold),
            bloom_intensity: lerp(self.bloom_intensity, other.bloom_intensity),
        }
    }
}
