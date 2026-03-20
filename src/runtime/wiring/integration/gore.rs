// LEGACY IMPORTS - Use canonical crates instead
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_core::events::canonical::*;
use engine_render::gore_mesh::{GoreMeshInstance, GoreMeshSystem};
use crate::physics::damage_pipeline::response_aggregator::BodyZone;
use crate::core::system::EngineSystem as LegacyEngineSystem;

fn u8_to_body_zone(z: u8) -> BodyZone {
    match z {
        0 => BodyZone::Head,
        1 => BodyZone::Neck,
        2 => BodyZone::Torso,
        3 => BodyZone::LeftArm,
        4 => BodyZone::RightArm,
        5 => BodyZone::LeftLeg,
        6 => BodyZone::RightLeg,
        7 => BodyZone::Pelvis,
        _ => BodyZone::Torso,
    }
}

// ---------------------------------------------------------------------------
// 7. GoreWireSystem
// ---------------------------------------------------------------------------

pub struct GoreWireSystem;

impl LegacyEngineSystem for GoreWireSystem {
    fn name(&self) -> &str {
        "GoreWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("GoreWire")
            .reads_resource::<GoreMeshSystem>()
            .reads_event::<BodyZoneDamaged>()
            .emits_event::<GoreMeshSpawn>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(gore) = ctx.resources.get_mut::<GoreMeshSystem>() else {
            return;
        };

        let damaged: Vec<BodyZoneDamaged> = ctx
            .events
            .read::<BodyZoneDamaged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &damaged {
            let body_zone = u8_to_body_zone(ev.zone);
            let intensity = ev.damage.min(1.0);

            gore.add_gore(GoreMeshInstance {
                entity: ev.entity,
                zone: body_zone,
                position: ev.position,
                scale: 1.0,
                blood_intensity: intensity,
            });

            ctx.events.emit(GoreMeshSpawn {
                entity: ev.entity,
                zone: ev.zone,
                position: ev.position,
                intensity,
            });
        }
    }
}