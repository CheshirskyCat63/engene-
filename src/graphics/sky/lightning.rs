//! Phase 8: Procedural lightning bolt generation with flash lighting.

use glam::Vec3;

pub struct LightningStrike {
    pub position: Vec3,
    pub duration: f32,
    pub brightness: f32,
    pub branch_count: u32,
    pub bolt_segments: Vec<[Vec3; 2]>,
    pub timer: f32,
    pub thunder_delay: f32,
    pub thunder_played: bool,
}

pub struct LightningSystem {
    pub active_strikes: Vec<LightningStrike>,
    pub flash_intensity: f32,
}

impl LightningSystem {
    pub fn new() -> Self {
        Self {
            active_strikes: Vec::new(),
            flash_intensity: 0.0,
        }
    }

    pub fn spawn_strike(&mut self, position: Vec3, camera_pos: Vec3) {
        let distance = (position - camera_pos).length();
        let segments = Self::generate_bolt(position, position + Vec3::new(0.0, -8000.0, 0.0), 5);
        self.active_strikes.push(LightningStrike {
            position,
            duration: 0.15,
            brightness: 10.0,
            branch_count: 4,
            bolt_segments: segments,
            timer: 0.0,
            thunder_delay: distance / 343.0,
            thunder_played: false,
        });
    }

    fn generate_bolt(start: Vec3, end: Vec3, depth: u32) -> Vec<[Vec3; 2]> {
        if depth == 0 {
            return vec![[start, end]];
        }
        let mid = (start + end) * 0.5;
        let offset = Vec3::new(
            ((depth as f32) * 127.1).sin() * 500.0,
            0.0,
            ((depth as f32) * 269.5).cos() * 500.0,
        );
        let mid_displaced = mid + offset;
        let mut segs = Self::generate_bolt(start, mid_displaced, depth - 1);
        segs.extend(Self::generate_bolt(mid_displaced, end, depth - 1));
        segs
    }

    pub fn update(&mut self, dt: f32) {
        self.flash_intensity = 0.0;
        for strike in &mut self.active_strikes {
            strike.timer += dt;
            if strike.timer < strike.duration {
                let t = strike.timer / strike.duration;
                self.flash_intensity += strike.brightness * (1.0 - t);
            }
        }
        self.active_strikes.retain(|s| s.timer < s.duration + s.thunder_delay + 1.0);
    }
}
