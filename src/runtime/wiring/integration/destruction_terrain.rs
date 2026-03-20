use engine_core::system::EngineSystem as LegacyEngineSystem;
use crate::world::terrain_damage::CraterStamp;
use crate::world::terrain_deformation::TerrainDeformationSystem;
use engine_core::events::canonical::*;
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_physics::destruction::{DestructionEvent, DestructionLod, DestructionSystem};
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};

use glam::Vec3;

// ---------------------------------------------------------------------------
// 3. DestructionTickSystem
// ---------------------------------------------------------------------------

pub struct DestructionTickSystem;

impl LegacyEngineSystem for DestructionTickSystem {
    fn name(&self) -> &str {
        "DestructionTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("DestructionTick")
            .reads_resource::<DestructionSystem>()
            .reads_event::<ImpactEvent>()
            .emits_event::<WorldTopologyChanged>()
            .emits_event::<StructuralCollapse>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(destruction) = ctx.resources.get_mut::<DestructionSystem>() else {
            return;
        };

        let impacts: Vec<ImpactEvent> = ctx
            .events
            .read::<ImpactEvent>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &impacts {
            destruction.apply_impulse_at(ev.position, ev.energy, DestructionLod::Full);
        }

        let events = destruction.drain_events();

        for ev in events {
            if let DestructionEvent::ObjectFragmented {
                entity,
                cluster_count,
            } = ev
            {
                let position = ctx
                    .ecs
                    .transforms
                    .get(&entity)
                    .map(|t| Vec3::new(t.x, 0.0, t.y))
                    .unwrap_or(Vec3::ZERO);

                ctx.events.emit(WorldTopologyChanged {
                    position,
                    radius: 10.0,
                    cause: TopologyChangeCause::StructuralCollapse,
                });
                ctx.events.emit(StructuralCollapse {
                    entity,
                    position,
                    cluster_count,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. TerrainDeformationTickSystem
// ---------------------------------------------------------------------------

pub struct TerrainDeformationTickSystem;

impl LegacyEngineSystem for TerrainDeformationTickSystem {
    fn name(&self) -> &str {
        "TerrainDeformationTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("TerrainDeformationTick")
            .reads_resource::<TerrainDeformationSystem>()
            .reads_event::<TerrainDeformed>()
            .emits_event::<TerrainChanged>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(terrain) = ctx.resources.get_mut::<TerrainDeformationSystem>() else {
            return;
        };

        let deformed: Vec<TerrainDeformed> = ctx
            .events
            .read::<TerrainDeformed>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &deformed {
            terrain.submit_crater(CraterStamp {
                center: ev.position,
                radius: ev.radius,
                depth: ev.depth,
                rim_height: ev.depth * 0.3,
                energy: ev.radius * ev.depth * 100.0,
            });
        }

        terrain.process_frame();
        let patches = terrain.drain_dirty_patches();

        if !patches.is_empty() {
            ctx.events.emit(TerrainChanged { patches });
        }
    }
}
