// LEGACY IMPORTS - Use canonical crates instead
use crate::core::system::EngineSystem as LegacyEngineSystem;
use crate::world::fields::WorldFields;
use engine_core::events::canonical::*;
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_physics::ballistics::{BallisticEvent, BallisticsSystem};
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};

const SIM_DT: f32 = 1.0 / 20.0;

// ---------------------------------------------------------------------------
// 1. BallisticsTickSystem
// ---------------------------------------------------------------------------

pub struct BallisticsTickSystem;

impl LegacyEngineSystem for BallisticsTickSystem {
    fn name(&self) -> &str {
        "BallisticsTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("BallisticsTick")
            .writes_resource::<BallisticsSystem>()
            .reads_resource::<WorldFields>()
            .emits_event::<ImpactEvent>()
            .emits_event::<SoundTrigger>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        if !ctx.resources.contains::<WorldFields>() {
            return;
        }
        if !ctx.resources.contains::<BallisticsSystem>() {
            return;
        }

        let height_fn: Box<dyn Fn(f32, f32) -> f32> = if let Some(hm) = ctx
            .resources
            .get::<std::sync::Arc<crate::world::heightmap::Heightmap>>()
        {
            let hm = hm.clone();
            Box::new(move |x, z| hm.sample(x, z))
        } else {
            Box::new(|_x, _z| 0.0)
        };

        let mut ballistics = ctx
            .resources
            .take::<BallisticsSystem>()
            .expect("BallisticsTick requires BallisticsSystem resource");
        let fields = ctx
            .resources
            .get::<WorldFields>()
            .expect("BallisticsTick requires canonical WorldFields resource");

        let events = ballistics.drain_events();

        for ev in &events {
            match ev {
                BallisticEvent::Impact {
                    hit_pos,
                    normal,
                    material,
                    damage,
                    ..
                } => {
                    ctx.events.emit(ImpactEvent {
                        position: *hit_pos,
                        direction: -*normal,
                        energy: *damage,
                        material_hit: *material,
                        instigator: None,
                        target_entity: None,
                    });
                }
                BallisticEvent::EntityHit {
                    entity,
                    damage,
                    hit_pos,
                    projectile_vel,
                } => {
                    let energy = 0.5 * projectile_vel.length_squared();
                    ctx.events.emit(ImpactEvent {
                        position: *hit_pos,
                        direction: projectile_vel.normalize(),
                        energy: *damage + energy * 0.01,
                        material_hit: 0,
                        instigator: None,
                        target_entity: Some(*entity),
                    });
                }
                BallisticEvent::ShotFired { origin, .. } => {
                    ctx.events.emit(SoundTrigger {
                        position: *origin,
                        kind: SoundTriggerKind::GunShot,
                        volume: 1.0,
                    });
                }
            }
        }

        ballistics.update(SIM_DT, fields, &*height_fn);
        ctx.resources.insert_runtime(ballistics);
    }
}

// ---------------------------------------------------------------------------
// 2. DamageDispatchSystem
// ---------------------------------------------------------------------------

pub struct DamageDispatchSystem;

impl LegacyEngineSystem for DamageDispatchSystem {
    fn name(&self) -> &str {
        "DamageDispatch"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("DamageDispatch")
            .reads_event::<ImpactEvent>()
            .emits_event::<BodyZoneDamaged>()
            .emits_event::<TerrainDeformed>()
            .emits_event::<SurfaceDamaged>()
            .emits_event::<SoundTrigger>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let impacts: Vec<ImpactEvent> = ctx
            .events
            .read::<ImpactEvent>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        // Wire EventAggregator: spatial bucketing of impacts
        if let Some(aggregator) = ctx
            .resources
            .get_mut::<crate::core::events::aggregation::EventAggregator>()
        {
            for ev in &impacts {
                aggregator.submit(ev);
            }
            let _aggregated = aggregator.drain();
        }

        for ev in impacts {
            let is_entity_hit = ev.target_entity.is_some();

            if is_entity_hit {
                if let Some(target) = ev.target_entity {
                    ctx.events.emit(BodyZoneDamaged {
                        entity: target,
                        zone: 2, // Torso
                        damage: ev.energy,
                        position: ev.position,
                    });
                }
            } else {
                ctx.events.emit(TerrainDeformed {
                    position: ev.position,
                    radius: (ev.energy * 0.01).sqrt().min(2.0),
                    depth: (ev.energy * 0.005).min(1.0),
                });
            }

            ctx.events.emit(SurfaceDamaged {
                position: ev.position,
                material: ev.material_hit,
                intensity: (ev.energy * 0.01).min(1.0),
            });
            ctx.events.emit(SoundTrigger {
                position: ev.position,
                kind: SoundTriggerKind::Impact {
                    material: ev.material_hit,
                },
                volume: (ev.energy * 0.001).min(1.0),
            });
        }
    }
}
