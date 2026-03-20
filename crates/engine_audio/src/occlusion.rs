use glam::Vec3;

pub struct AudioOcclusionQuery {
    pub source: Vec3,
    pub listener: Vec3,
    pub occlusion: f32,
    pub reverb_amount: f32,
}

pub struct ReverbZone {
    pub center: Vec3,
    pub radius: f32,
    pub reverb_time: f32,
    pub early_reflections: f32,
    pub density: f32,
}

impl ReverbZone {
    pub fn cave(center: Vec3) -> Self {
        Self {
            center,
            radius: 50.0,
            reverb_time: 3.0,
            early_reflections: 0.8,
            density: 0.9,
        }
    }

    pub fn forest(center: Vec3) -> Self {
        Self {
            center,
            radius: 100.0,
            reverb_time: 0.5,
            early_reflections: 0.3,
            density: 0.4,
        }
    }

    pub fn open_field(center: Vec3) -> Self {
        Self {
            center,
            radius: 200.0,
            reverb_time: 0.1,
            early_reflections: 0.1,
            density: 0.1,
        }
    }
}

pub struct OcclusionSystem {
    pub zones: Vec<ReverbZone>,
}

impl OcclusionSystem {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn query(&self, source: Vec3, listener: Vec3) -> AudioOcclusionQuery {
        let dir = source - listener;
        let dist = dir.length();

        let distance_attenuation = 1.0 / (1.0 + dist * 0.01);

        let mut best_reverb = 0.0f32;
        for zone in &self.zones {
            let to_zone = listener - zone.center;
            let zone_dist = to_zone.length();
            if zone_dist < zone.radius {
                let blend = 1.0 - (zone_dist / zone.radius);
                best_reverb = best_reverb.max(zone.reverb_time * blend);
            }
        }

        AudioOcclusionQuery {
            source,
            listener,
            occlusion: 1.0 - distance_attenuation,
            reverb_amount: best_reverb,
        }
    }
}
