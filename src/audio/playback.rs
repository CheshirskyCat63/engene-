use crate::audio::audio::{AudioEngine, SoundKind};
use crate::audio::sound_bank::{SoundBank, SoundEvent};
use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;

fn sound_kind_to_event(kind: SoundKind) -> SoundEvent {
    match kind {
        SoundKind::Footstep => SoundEvent::FootstepDirt,
        SoundKind::Ambient => SoundEvent::AmbientWind,
        SoundKind::Impact => SoundEvent::BulletImpactMetal,
        SoundKind::Voice => SoundEvent::AmbientWind,
        SoundKind::Wind => SoundEvent::AmbientWind,
        SoundKind::Water => SoundEvent::AmbientWind,
        SoundKind::Explosion => SoundEvent::ExplosionSmall,
        SoundKind::Gunshot => SoundEvent::WeaponFirePistol,
    }
}

pub struct AudioPlaybackBridge {
    pending_log: Vec<(SoundKind, String)>,
}

impl AudioPlaybackBridge {
    pub fn new() -> Self {
        Self {
            pending_log: Vec::new(),
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending_log.len()
    }

    pub fn resolve_event(&self, kind: SoundKind) -> SoundEvent {
        sound_kind_to_event(kind)
    }
}

impl EngineSystem for AudioPlaybackBridge {
    fn name(&self) -> &str {
        "AudioPlaybackBridge"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AudioPlaybackBridge")
            .reads_resource::<AudioEngine>()
            .reads_resource::<SoundBank>()
            .with_headless(false)
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(audio) = ctx.resources.get::<AudioEngine>() else {
            return;
        };
        let Some(bank) = ctx.resources.get::<SoundBank>() else {
            return;
        };

        self.pending_log.clear();
        let count = audio.active_sound_count();
        if count == 0 {
            return;
        }

        #[cfg(feature = "audio_playback")]
        {
            let _ = (bank, count);
        }

        #[cfg(not(feature = "audio_playback"))]
        {
            let _ = (bank, count);
        }
    }
}
