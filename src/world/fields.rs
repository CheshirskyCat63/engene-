use glam::Vec3;

/// Minimal storm cell data for wind contribution (avoids world depending on graphics).
#[derive(Clone, Debug)]
pub struct StormCellWind {
    pub position: Vec3,
    pub radius: f32,
    pub wind_vector: Vec3,
}

pub struct WindField {
    pub base_dir: Vec3,
    pub strength: f32,
    pub turbulence: f32,
    pub storm_wind: Vec3,
    pub curl_turbulence: f32,
}

impl WindField {
    pub fn new() -> Self {
        Self {
            base_dir: Vec3::new(1.0, 0.0, 0.3).normalize(),
            strength: 5.0,
            turbulence: 0.3,
            storm_wind: Vec3::ZERO,
            curl_turbulence: 0.2,
        }
    }

    /// Sets storm_wind from the nearest active storm cell (by distance from origin).
    pub fn update_from_weather(&mut self, storm_cells: &[StormCellWind], _time: f32) {
        let origin = Vec3::ZERO;
        self.storm_wind = storm_cells
            .iter()
            .min_by(|a, b| {
                let da = (a.position - origin).length_squared();
                let db = (b.position - origin).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|c| c.wind_vector)
            .unwrap_or(Vec3::ZERO);
    }

    /// Curl noise approximation: sin/cos for CPU (no GPU noise).
    fn curl_noise(&self, pos: Vec3, time: f32) -> Vec3 {
        let curl_x = (pos.z * 0.01 + time).sin() * (pos.y * 0.02).cos();
        let curl_y = (pos.x * 0.015 + time * 0.7).cos() * (pos.z * 0.012).sin();
        let curl_z = (pos.y * 0.01 + time * 0.5).sin() * (pos.x * 0.02).cos();
        Vec3::new(curl_x, curl_y, curl_z) * self.curl_turbulence
    }

    /// Terrain influence: valleys funnel (boost horizontal when below avg), ridges deflect (reduce wind at height).
    /// Uses procedural local "avg" as proxy when no heightmap available.
    fn terrain_influence(&self, pos: Vec3, base_wind: Vec3) -> Vec3 {
        let local_avg = (pos.x * 0.005).sin() * 80.0 + (pos.z * 0.007).cos() * 60.0;
        let height_above_avg = pos.y - local_avg;

        if height_above_avg < 0.0 {
            let valley_factor = 1.0 + (-height_above_avg / 100.0).min(0.5);
            let mut horizontal = base_wind;
            horizontal.y = 0.0;
            let len = horizontal.length().max(0.001);
            horizontal = horizontal.normalize() * len * valley_factor;
            Vec3::new(horizontal.x, base_wind.y, horizontal.z)
        } else {
            let ridge_factor = 1.0 / (1.0 + height_above_avg * 0.002);
            base_wind * ridge_factor
        }
    }

    pub fn sample(&self, pos: Vec3, time: f32) -> Vec3 {
        let phase = pos.x * 0.01 + pos.z * 0.013 + time * 0.5;
        let turb = Vec3::new(
            phase.sin() * self.turbulence,
            0.0,
            (phase * 1.3).cos() * self.turbulence,
        );
        let global_wind = self.base_dir * self.strength + turb;
        let wind_with_storm = global_wind + self.storm_wind;
        let curl = self.curl_noise(pos, time);
        let with_curl = wind_with_storm + curl;
        self.terrain_influence(pos, with_curl)
    }
}

pub struct AirDensityField {
    pub base_density: f32,
    pub temp_gradient: f32,
}

impl AirDensityField {
    pub fn new() -> Self {
        Self {
            base_density: 1.225,
            temp_gradient: -0.0065,
        }
    }

    pub fn sample(&self, pos: Vec3, _time: f32) -> f32 {
        let altitude = pos.y.max(0.0);
        (self.base_density + self.temp_gradient * altitude * 0.001).max(0.3)
    }
}

pub struct RainField {
    pub intensity: f32,
    pub coverage: f32,
}

impl RainField {
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            coverage: 0.0,
        }
    }

    pub fn sample(&self, _pos: Vec3, _time: f32) -> f32 {
        self.intensity * self.coverage
    }

    pub fn set_rain(&mut self, intensity: f32, coverage: f32) {
        self.intensity = intensity;
        self.coverage = coverage;
    }
}

pub struct AnomalyZone {
    pub center: Vec3,
    pub radius: f32,
    pub force_strength: f32,
    pub force_type: AnomalyForceType,
}

#[derive(Clone, Copy, Debug)]
pub enum AnomalyForceType {
    Vortex,
    Gravity,
    Repulsion,
    Random,
}

pub struct AnomalyForces {
    pub acceleration: Vec3,
    pub inside_anomaly: bool,
}

pub struct AnomalyField {
    pub zones: Vec<AnomalyZone>,
}

impl AnomalyField {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn sample(&self, pos: Vec3) -> AnomalyForces {
        let mut accel = Vec3::ZERO;
        let mut inside = false;

        for zone in &self.zones {
            let delta = pos - zone.center;
            let dist = delta.length();
            if dist < zone.radius {
                inside = true;
                let falloff = 1.0 - dist / zone.radius;
                let force = match zone.force_type {
                    AnomalyForceType::Vortex => {
                        Vec3::new(-delta.z, 0.0, delta.x).normalize_or_zero() * zone.force_strength * falloff
                    }
                    AnomalyForceType::Gravity => {
                        Vec3::new(0.0, -zone.force_strength * falloff, 0.0)
                    }
                    AnomalyForceType::Repulsion => {
                        delta.normalize_or_zero() * zone.force_strength * falloff
                    }
                    AnomalyForceType::Random => {
                        let seed = (pos.x * 17.3 + pos.z * 31.7) % 6.28;
                        Vec3::new(seed.sin(), 0.0, seed.cos()) * zone.force_strength * falloff
                    }
                };
                accel += force;
            }
        }

        AnomalyForces { acceleration: accel, inside_anomaly: inside }
    }
}

pub struct WorldFields {
    pub wind: WindField,
    pub air_density: AirDensityField,
    pub rain: RainField,
    pub anomaly: AnomalyField,
}

impl WorldFields {
    pub fn new() -> Self {
        Self {
            wind: WindField::new(),
            air_density: AirDensityField::new(),
            rain: RainField::new(),
            anomaly: AnomalyField::new(),
        }
    }
}
