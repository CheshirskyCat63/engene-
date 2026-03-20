use glam::Vec3;

pub const OCCLUSION_UPDATE_RADIUS: f32 = 500.0;

#[derive(Clone, Debug)]
pub struct OcclusionBreach {
    pub position: Vec3,
    pub radius: f32,
    pub sound_passthrough: f32,
    pub light_passthrough: f32,
    pub vision_passthrough: f32,
}

pub struct DestructionOcclusionSystem {
    breaches: Vec<OcclusionBreach>,
}

impl DestructionOcclusionSystem {
    pub fn new() -> Self {
        Self {
            breaches: Vec::new(),
        }
    }

    pub fn register_breach(&mut self, breach: OcclusionBreach) {
        self.breaches.push(breach);
    }

    pub fn query_sound_occlusion(&self, from: Vec3, to: Vec3) -> f32 {
        let mut occlusion = 1.0_f32;
        let dir = to - from;
        let len = dir.length();
        if len < 0.01 {
            return 0.0;
        }
        let dir_norm = dir / len;

        for breach in &self.breaches {
            let to_breach = breach.position - from;
            let proj = to_breach.dot(dir_norm);
            if proj < 0.0 || proj > len {
                continue;
            }
            let closest = from + dir_norm * proj;
            let dist = (closest - breach.position).length();
            if dist < breach.radius {
                occlusion *= 1.0 - breach.sound_passthrough;
            }
        }
        occlusion
    }

    pub fn query_vision(&self, from: Vec3, to: Vec3) -> bool {
        let dir = to - from;
        let len = dir.length();
        if len < 0.01 {
            return true;
        }
        let dir_norm = dir / len;

        for breach in &self.breaches {
            let to_breach = breach.position - from;
            let proj = to_breach.dot(dir_norm);
            if proj < 0.0 || proj > len {
                continue;
            }
            let closest = from + dir_norm * proj;
            let dist = (closest - breach.position).length();
            if dist < breach.radius && breach.vision_passthrough > 0.5 {
                return true;
            }
        }
        false
    }

    pub fn breaches_near(&self, pos: Vec3, radius: f32) -> Vec<&OcclusionBreach> {
        self.breaches
            .iter()
            .filter(|b| (b.position - pos).length() < radius)
            .collect()
    }

    pub fn breach_count(&self) -> usize {
        self.breaches.len()
    }
}
