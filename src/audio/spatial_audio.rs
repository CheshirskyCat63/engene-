use glam::Vec3;

use crate::audio::audio::{AudioEngine, SoundKind};
use crate::core::ecs::Ecs;

pub struct SpatialAudioSystem {
    pub max_audible_distance: f32,
    cooldown: f32,
    timer: f32,
}

impl SpatialAudioSystem {
    pub fn new() -> Self {
        Self {
            max_audible_distance: 200.0,
            cooldown: 0.5,
            timer: 0.0,
        }
    }

    pub fn update(
        &mut self,
        engine: &mut AudioEngine,
        ecs: &Ecs,
        dt: f32,
    ) {
        self.timer += dt;
        if self.timer < self.cooldown {
            return;
        }
        self.timer = 0.0;

        let listener = engine.listener_pos;

        for &e in &ecs.alive {
            if let Some(t) = ecs.get_transform(e) {
                let pos = Vec3::new(t.x, 0.0, t.y);
                let dist = (pos - listener).length();
                if dist > self.max_audible_distance {
                    continue;
                }

                if let Some(ai) = ecs.get_ai_state(e) {
                    if matches!(ai, crate::world::components::AiState::Executing(_)) {
                        let vol = (1.0 - dist / self.max_audible_distance) * 0.3;
                        engine.play_3d(SoundKind::Footstep, pos, vol, 0.3);
                    }
                }
            }
        }
    }
}
