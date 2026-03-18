//! Phase 5: Audio wiring - connects SoundTrigger events to AudioEngine with occlusion.

use crate::audio::audio::{AudioEngine, SoundKind};
use crate::audio::occlusion::OcclusionSystem;
use crate::core::events::canonical::{SoundTrigger, SoundTriggerKind};
use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::graphics::destruction_occlusion::DestructionOcclusionSystem;

fn map_trigger_to_sound(kind: &SoundTriggerKind) -> SoundKind {
    match kind {
        SoundTriggerKind::Impact { .. } => SoundKind::Impact,
        SoundTriggerKind::Collapse => SoundKind::Explosion,
        SoundTriggerKind::Fire => SoundKind::Ambient,
        SoundTriggerKind::Pain => SoundKind::Impact,
        SoundTriggerKind::Footstep { .. } => SoundKind::Footstep,
        SoundTriggerKind::GunShot => SoundKind::Gunshot,
    }
}

fn default_duration(kind: &SoundTriggerKind) -> f32 {
    match kind {
        SoundTriggerKind::Impact { .. } | SoundTriggerKind::Pain => 0.2,
        SoundTriggerKind::Collapse => 1.5,
        SoundTriggerKind::Fire => 2.0,
        SoundTriggerKind::Footstep { .. } => 0.3,
        SoundTriggerKind::GunShot => 0.5,
    }
}

pub struct AudioIntegrationSystem;

impl AudioIntegrationSystem {
    pub fn new() -> Self {
        Self
    }
}

impl EngineSystem for AudioIntegrationSystem {
    fn name(&self) -> &str {
        "AudioIntegration"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AudioIntegration")
            .reads_resource::<AudioEngine>()
            .reads_resource::<DestructionOcclusionSystem>()
            .reads_resource::<OcclusionSystem>()
            .reads_event::<SoundTrigger>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let listener_pos = ctx
            .resources
            .get::<AudioEngine>()
            .map(|a| a.listener_pos)
            .unwrap_or(glam::Vec3::ZERO);

        let triggers: Vec<SoundTrigger> = ctx
            .events
            .read::<SoundTrigger>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        let occlusion = ctx.resources.get::<DestructionOcclusionSystem>();
        let reverb_sys = ctx.resources.get::<OcclusionSystem>();

        let plays: Vec<(SoundKind, glam::Vec3, f32, f32, f32)> = triggers
            .iter()
            .map(|ev| {
                let sound_kind = map_trigger_to_sound(&ev.kind);
                let base_volume = ev.volume;
                let duration = default_duration(&ev.kind);
                let volume = if let Some(occl) = occlusion {
                    let occl_factor = occl.query_sound_occlusion(ev.position, listener_pos);
                    base_volume * (1.0 - occl_factor * 0.5).max(0.3)
                } else {
                    base_volume
                };

                let reverb = if let Some(rsys) = reverb_sys {
                    let q = rsys.query(ev.position, listener_pos);
                    q.reverb_amount
                } else {
                    0.0
                };

                (sound_kind, ev.position, volume, duration, reverb)
            })
            .collect();

        if let Some(audio) = ctx.resources.get_mut::<AudioEngine>() {
            for (kind, pos, volume, duration, reverb) in plays {
                let adjusted_duration = duration * (1.0 + reverb * 0.3);
                audio.play_3d(kind, pos, volume, adjusted_duration);
            }
        }
    }
}
