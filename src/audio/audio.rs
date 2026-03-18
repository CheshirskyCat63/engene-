use glam::Vec3;

pub struct AudioEngine {
    pub listener_pos: Vec3,
    pub listener_forward: Vec3,
    pub master_volume: f32,
    active_sounds: Vec<ActiveSound>,
    next_id: u64,
}

struct ActiveSound {
    id: u64,
    kind: SoundKind,
    position: Option<Vec3>,
    volume: f32,
    looping: bool,
    time: f32,
    duration: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundKind {
    Footstep,
    Ambient,
    Impact,
    Voice,
    Wind,
    Water,
    Explosion,
    Gunshot,
}

pub struct SoundHandle(u64);

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            listener_pos: Vec3::ZERO,
            listener_forward: Vec3::NEG_Z,
            master_volume: 1.0,
            active_sounds: Vec::new(),
            next_id: 0,
        }
    }

    pub fn set_listener(&mut self, pos: Vec3, forward: Vec3) {
        self.listener_pos = pos;
        self.listener_forward = forward.normalize_or_zero();
    }

    pub fn play_3d(
        &mut self,
        kind: SoundKind,
        position: Vec3,
        volume: f32,
        duration: f32,
    ) -> SoundHandle {
        let id = self.next_id;
        self.next_id += 1;
        self.active_sounds.push(ActiveSound {
            id,
            kind,
            position: Some(position),
            volume,
            looping: false,
            time: 0.0,
            duration,
        });
        SoundHandle(id)
    }

    pub fn play_ambient(&mut self, kind: SoundKind, volume: f32) -> SoundHandle {
        let id = self.next_id;
        self.next_id += 1;
        self.active_sounds.push(ActiveSound {
            id,
            kind,
            position: None,
            volume,
            looping: true,
            time: 0.0,
            duration: f32::MAX,
        });
        SoundHandle(id)
    }

    pub fn stop(&mut self, handle: &SoundHandle) {
        self.active_sounds.retain(|s| s.id != handle.0);
    }

    pub fn update(&mut self, dt: f32) {
        let listener = self.listener_pos;
        self.active_sounds.retain_mut(|sound| {
            sound.time += dt;
            if !sound.looping && sound.time >= sound.duration {
                return false;
            }
            if let Some(pos) = sound.position {
                let dist = (pos - listener).length();
                let spatial_vol = sound.volume / (1.0 + dist * 0.05);
                let _kind = sound.kind;
                if spatial_vol < 0.001 {
                    return false;
                }
            }
            true
        });
    }

    pub fn compute_spatial_volume(&self, world_pos: Vec3, base_volume: f32) -> (f32, f32) {
        let offset = world_pos - self.listener_pos;
        let distance = offset.length();

        let attenuation = 1.0 / (1.0 + distance * 0.05 + distance * distance * 0.001);
        let volume = base_volume * attenuation * self.master_volume;

        let right = self.listener_forward.cross(Vec3::Y).normalize_or_zero();
        let pan = if distance > 0.1 {
            offset.normalize().dot(right) * 0.5
        } else {
            0.0
        };

        (volume, pan)
    }

    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }
}
