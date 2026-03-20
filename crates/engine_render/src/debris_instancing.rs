use glam::Vec3;

use crate::world::damage_profiles::DebrisProfileId;

#[derive(Clone, Debug)]
pub struct DebrisInstance {
    pub position: Vec3,
    pub velocity: Vec3,
    pub profile: DebrisProfileId,
    pub mass: f32,
    pub age: f32,
    pub lifetime: f32,
}

pub struct DebrisSystem {
    active: Vec<DebrisInstance>,
    max_active: usize,
}

impl DebrisSystem {
    pub fn new(max_active: usize) -> Self {
        Self {
            active: Vec::with_capacity(max_active),
            max_active,
        }
    }

    pub fn spawn(&mut self, position: Vec3, profile: DebrisProfileId, count: u16) {
        let per = count.min(16);
        for i in 0..per {
            if self.active.len() >= self.max_active {
                self.active.remove(0);
            }
            let angle = (i as f32 / per as f32) * std::f32::consts::TAU;
            let spread = Vec3::new(angle.cos() * 3.0, 2.0 + (i as f32) * 0.3, angle.sin() * 3.0);
            self.active.push(DebrisInstance {
                position,
                velocity: spread,
                profile,
                mass: 0.1 + (i as f32) * 0.05,
                age: 0.0,
                lifetime: 5.0,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for d in &mut self.active {
            d.velocity.y -= 9.81 * dt;
            d.position += d.velocity * dt;
            d.age += dt;
        }
        self.active.retain(|d| d.age < d.lifetime);
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}
