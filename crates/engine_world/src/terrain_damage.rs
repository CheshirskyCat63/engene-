use glam::Vec3;

pub struct CraterStamp {
    pub center: Vec3,
    pub radius: f32,
    pub depth: f32,
    pub rim_height: f32,
    pub energy: f32,
}

impl CraterStamp {
    pub fn from_explosion(center: Vec3, energy: f32) -> Self {
        let radius = (energy * 0.01).sqrt().min(10.0);
        let depth = (energy * 0.005).min(3.0);
        Self {
            center,
            radius,
            depth,
            rim_height: depth * 0.3,
            energy,
        }
    }

    pub fn sample_depth(&self, distance: f32) -> f32 {
        if distance >= self.radius {
            return 0.0;
        }
        let t = distance / self.radius;
        if t < 0.7 {
            -self.depth * (1.0 - (t / 0.7).powi(2))
        } else {
            self.rim_height * ((t - 0.7) / 0.3)
        }
    }
}
