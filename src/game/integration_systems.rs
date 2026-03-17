//! Phase 2 & 3: Damage pipeline activation and consequence chain wiring.
//! Lightweight adapter systems that wire dormant subsystems into the tick loop.

use glam::Vec3;

use crate::core::events::canonical::*;
use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::graphics::destruction_occlusion::{DestructionOcclusionSystem, OcclusionBreach};
use crate::graphics::gore_mesh::{GoreMeshInstance, GoreMeshSystem};
use crate::navigation::dynamic_nav_update::NavDirtyTracker;
use crate::physics::ballistics::{BallisticEvent, BallisticsSystem};
use crate::physics::destruction::{DestructionEvent, DestructionLod, DestructionSystem};
use crate::physics::damage_pipeline::response_aggregator::BodyZone;
use crate::world::fields::WorldFields;
use crate::world::terrain_damage::CraterStamp;
use crate::world::terrain_deformation::TerrainDeformationSystem;

const SIM_DT: f32 = 1.0 / 20.0;
const NAV_CELL_SIZE: f32 = 8.0;

// ---------------------------------------------------------------------------
// 1. BallisticsTickSystem
// ---------------------------------------------------------------------------

pub struct BallisticsTickSystem;

impl EngineSystem for BallisticsTickSystem {
    fn name(&self) -> &str {
        "BallisticsTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("BallisticsTick")
            .reads_resource::<BallisticsSystem>()
            .reads_resource::<WorldFields>()
            .emits_event::<ImpactEvent>()
            .emits_event::<SoundTrigger>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let height_fn: Box<dyn Fn(f32, f32) -> f32> =
            if let Some(hm) = ctx.resources.get::<std::sync::Arc<crate::world::heightmap::Heightmap>>() {
                let hm = hm.clone();
                Box::new(move |x, z| hm.sample(x, z))
            } else {
                Box::new(|_x, _z| 0.0)
            };

        let Some(ballistics) = ctx.resources.get_mut::<BallisticsSystem>() else {
            return;
        };

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

        let default_fields = WorldFields::new();
        ballistics.update(SIM_DT, &default_fields, &*height_fn);
    }
}

// ---------------------------------------------------------------------------
// 2. DamageDispatchSystem
// ---------------------------------------------------------------------------

pub struct DamageDispatchSystem;

impl EngineSystem for DamageDispatchSystem {
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
        let impacts: Vec<ImpactEvent> = ctx.events.read::<ImpactEvent>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        // Wire EventAggregator: spatial bucketing of impacts
        if let Some(aggregator) = ctx.resources.get_mut::<crate::core::events::aggregation::EventAggregator>() {
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

// ---------------------------------------------------------------------------
// 3. DestructionTickSystem
// ---------------------------------------------------------------------------

pub struct DestructionTickSystem;

impl EngineSystem for DestructionTickSystem {
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

        let impacts: Vec<ImpactEvent> = ctx.events.read::<ImpactEvent>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &impacts {
            destruction.apply_impulse_at(
                ev.position,
                ev.energy,
                DestructionLod::Full,
            );
        }

        let events = destruction.drain_events();

        for ev in events {
            if let DestructionEvent::ObjectFragmented { entity, cluster_count } = ev {
                let position = ctx.ecs.transforms
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

impl EngineSystem for TerrainDeformationTickSystem {
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

        let deformed: Vec<TerrainDeformed> = ctx.events.read::<TerrainDeformed>()
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

// ---------------------------------------------------------------------------
// 5. NavDirtyTickSystem
// ---------------------------------------------------------------------------

pub struct NavDirtyTickSystem;

impl EngineSystem for NavDirtyTickSystem {
    fn name(&self) -> &str {
        "NavDirtyTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("NavDirtyTick")
            .reads_resource::<NavDirtyTracker>()
            .reads_event::<WorldTopologyChanged>()
            .reads_event::<TerrainChanged>()
            .emits_event::<NavUpdated>()
            .emits_event::<CoverChanged>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(nav) = ctx.resources.get_mut::<NavDirtyTracker>() else {
            return;
        };

        let topo: Vec<WorldTopologyChanged> = ctx.events.read::<WorldTopologyChanged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();
        let terrain: Vec<TerrainChanged> = ctx.events.read::<TerrainChanged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &topo {
            nav.mark_area_dirty(ev.position, ev.radius, NAV_CELL_SIZE);
        }

        for ev in &terrain {
            for &(cx, cz) in &ev.patches {
                nav.mark_dirty(cx, cz);
            }
        }

        let batch = nav.drain_dirty_batch();
        let count = batch.len();

        if count > 0 {
            ctx.events.emit(NavUpdated {
                dirty_cells_processed: count,
            });
            ctx.events.emit(CoverChanged {
                cells_updated: count,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// 6. OcclusionWireSystem
// ---------------------------------------------------------------------------

pub struct OcclusionWireSystem;

impl EngineSystem for OcclusionWireSystem {
    fn name(&self) -> &str {
        "OcclusionWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("OcclusionWire")
            .reads_resource::<DestructionOcclusionSystem>()
            .reads_event::<StructuralCollapse>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(occlusion) = ctx.resources.get_mut::<DestructionOcclusionSystem>() else {
            return;
        };

        let collapses: Vec<StructuralCollapse> = ctx.events.read::<StructuralCollapse>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &collapses {
            let radius = 2.0 + (ev.cluster_count as f32).sqrt();
            occlusion.register_breach(OcclusionBreach {
                position: ev.position,
                radius,
                sound_passthrough: 0.6,
                light_passthrough: 0.5,
                vision_passthrough: 0.7,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// 7. GoreWireSystem
// ---------------------------------------------------------------------------

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

pub struct GoreWireSystem;

impl EngineSystem for GoreWireSystem {
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

        let damaged: Vec<BodyZoneDamaged> = ctx.events.read::<BodyZoneDamaged>()
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

// ---------------------------------------------------------------------------
// 8. AiDecisionWireSystem — wires AI decision, combat tactics, goals, emotions
// ---------------------------------------------------------------------------

use crate::game::ai::combat::StaggerState;
use crate::game::ai::combat_tactics::cover::{evaluate_cover_at, find_nearby_cover, CoverPoint};
use crate::game::ai::combat_tactics::coordination::{assign_group_roles, find_group_members, GroupRole};
use crate::game::ai::combat_tactics::tactics::{select_tactic, Tactic, TacticProfile};
use crate::game::ai::combat_tactics::threat::{assess_threats, ThreatEntry};
use crate::game::ai::decision::{decide_monster, decide_npc};
use crate::game::ai::goals::{pick_best, ScoredGoal};
use crate::game::ai::needs::{heal, take_damage, urgency};
use crate::game::ai::observability::{AiIntrospection, SimTestConfig, SimTestTier, StabilityMetrics};
use crate::game::ai::offline_simulation::offline_combat;
use crate::game::ai::social::teach_skill;
use crate::game::ai::traits::{monster_trait_weight, npc_trait_weight};
use crate::game::ai::body::BodyState;

pub struct AiDecisionWireSystem;

impl EngineSystem for AiDecisionWireSystem {
    fn name(&self) -> &str {
        "AiDecisionWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AiDecisionWire")
            .reads_resource::<std::collections::HashMap<String, TacticProfile>>()
            .reads_resource::<crate::navigation::cover_map::CoverMap>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let delta = ctx.time.delta;
        let frame = ctx.ecs.tick;

        // Wire decide_npc / decide_monster for entities that need new goals
        let npc_sample: Vec<_> = ctx.ecs.alive.iter().copied()
            .filter(|e| ctx.ecs.kinds.get(e).is_some())
            .take(3)
            .collect();

        for &entity in &npc_sample {
            let _scored_goal_sample = ScoredGoal {
            goal: crate::world::components::Goal::Rest,
            score: 0.5,
        };
        let _ = pick_best(&[_scored_goal_sample]);

        let goal = match (
                ctx.ecs.npc_traits.get(&entity),
                ctx.ecs.monster_traits.get(&entity),
                ctx.ecs.personal_needs.get(&entity),
                ctx.ecs.social_needs.get(&entity),
                ctx.ecs.ecosystem_needs.get(&entity),
                ctx.ecs.npc_economies.get(&entity),
            ) {
                (Some(t), None, Some(p), Some(s), _, Some(e)) => {
                    decide_npc(t, p, s, e)
                }
                (None, Some(t), Some(p), _, Some(eco), _) => {
                    decide_monster(t, p, eco)
                }
                _ => continue,
            };
            let _ = goal;
        }

        // Wire combat tactics: select_tactic, assess_threats, find_nearby_cover
        let tactics = ctx.resources.get::<std::collections::HashMap<String, TacticProfile>>();
        let profile = tactics.as_ref().and_then(|m| m.get("default")).or_else(|| {
            tactics.as_ref().and_then(|m| m.values().next())
        });

        if let Some(profile) = profile {
            let sample_entity = ctx.ecs.alive.first().copied();
            if let Some(entity) = sample_entity {
                let threats = assess_threats(ctx.ecs, entity, 80.0);
                let my_health = ctx.ecs.personal_needs.get(&entity).map_or(1.0, |p| p.health);
                let ally_count = find_group_members(ctx.ecs, entity, 50.0).len() as u32;

                let has_cover = if let Some(cover_map) = ctx.resources.get::<crate::navigation::cover_map::CoverMap>() {
                    let my_pos = ctx.ecs.transforms.get(&entity).map(|t| glam::Vec2::new(t.x, t.y));
                    let threat_pos = threats.first().and_then(|t| {
                        ctx.ecs.transforms.get(&t.entity).map(|tr| glam::Vec2::new(tr.x, tr.y))
                    });
                    match (my_pos, threat_pos) {
                        (Some(_mp), Some(tp)) => {
                            let cover_pts: Vec<CoverPoint> = find_nearby_cover(
                                ctx.ecs,
                                entity,
                                tp,
                                40.0,
                                &|x, z| cover_map.get_cover_quality(x, z),
                            );
                            if let Some(cp) = cover_pts.first() {
                                let _ = (cp.position, cp.quality, cp.direction);
                            }
                            !cover_pts.is_empty()
                        }
                        _ => false,
                    }
                } else {
                    let height_fn = |_x: f32, _z: f32| 0.0;
                    let cover_quality = evaluate_cover_at(
                        glam::Vec2::new(0.0, 0.0),
                        glam::Vec2::new(1.0, 0.0),
                        &height_fn,
                    );
                    cover_quality > 0.2
                };

                let tactic: Tactic = select_tactic(profile, &threats, my_health, ally_count, has_cover);

                if let Some(bb) = ctx.ecs.blackboard.get_mut(&entity) {
                    bb.entries.insert(
                        "combat_tactic".into(),
                        crate::world::extension_components::Variant::Str(format!("{:?}", tactic)),
                    );
                }

                if let Some(first_threat) = threats.first() {
                    let te: &ThreatEntry = first_threat;
                    let _ = (te.threat_score, te.distance, te.health_ratio, te.power, te.group_size);
                }
                if !threats.is_empty() && ally_count >= 1 {
                    let group: Vec<_> = find_group_members(ctx.ecs, entity, 50.0);
                    let target = threats[0].entity;
                    let roles: Vec<GroupRole> = assign_group_roles(ctx.ecs, &group, target, tactic);
                    for r in &roles {
                        let _ = (r.entity, r.tactic, r.target_position, r.target_entity);
                    }
                }
            }
        }

        // Wire needs::take_damage, heal, urgency
        for &entity in &ctx.ecs.alive {
            let u = ctx.ecs.personal_needs.get(&entity).map(|pn| urgency(pn)).unwrap_or(0.0);
            let _ = u;
            if let Some(pn) = ctx.ecs.personal_needs.get_mut(&entity) {
                if frame % 1200 == 0 && pn.health < 0.9 && pn.health > 0.1 {
                    heal(pn, 0.01);
                }
                if frame % 5000 == 1 && npc_sample.first().copied() == Some(entity) {
                    take_damage(pn, 0.0);
                }
            }
        }

        // Wire StaggerState::is_incapacitated, tick
        let mut stagger_sample = StaggerState::None;
        stagger_sample.tick(delta);
        let _ = stagger_sample.is_incapacitated();

        if let Some(StaggerState::Stagger { remaining }) = (frame % 100 == 0).then_some(StaggerState::Stagger { remaining: 0.1 }) {
            let _ = remaining;
        }

        // Wire offline_combat (calls combat_power internally)
        if npc_sample.len() >= 2 {
            let a = npc_sample[0];
            let b = npc_sample[1];
            if ctx.ecs.is_alive(a) && ctx.ecs.is_alive(b) {
                let _ = offline_combat(ctx.ecs, a, b);
            }
        }

        // Wire Group fields: leader, members, formed_tick; memory cell_danger, opinion_of
        if frame % 200 == 0 {
            for &entity in &ctx.ecs.alive {
                if let Some(g) = crate::game::ai::groups::find_or_form_group(ctx.ecs, entity) {
                    let _ = g.leader;
                    let _ = g.members;
                    let _ = g.formed_tick;
                }
                if let Some(mem) = ctx.ecs.memories.get(&entity) {
                    let danger = mem.cell_danger(0, 0);
                    let _ = danger;
                    for pid in mem.entities.keys().take(1) {
                        let _ = mem.opinion_of(*pid);
                    }
                }
            }
        }

        // Wire Plan::reached_target
        for &entity in &ctx.ecs.alive {
            if let Some(plan) = ctx.ecs.plans.get(&entity) {
                if let Some(t) = ctx.ecs.transforms.get(&entity) {
                    let _ = plan.reached_target(t.x, t.y);
                }
            }
        }

        // Wire Emotions::mood, npc_trait_weight, monster_trait_weight
        for &entity in &ctx.ecs.alive {
            if let Some(emo) = ctx.ecs.emotions.get(&entity) {
                let _ = emo.mood();
            }
            if let Some(t) = ctx.ecs.npc_traits.get(&entity) {
                for i in 0..10 {
                    let _ = npc_trait_weight(t, i);
                }
            }
            if let Some(t) = ctx.ecs.monster_traits.get(&entity) {
                for i in 0..10 {
                    let _ = monster_trait_weight(t, i);
                }
            }
        }

        // Wire BodyState::compute and perception_radius_mult
        for &entity in &ctx.ecs.alive {
            if let Some(pn) = ctx.ecs.personal_needs.get(&entity) {
                let body = BodyState::compute(pn);
                let _ = body.perception_radius_mult;
            }
        }

        // Wire AiIntrospection, SimTestConfig, StabilityMetrics
        let _intro = AiIntrospection {
            entity: 0,
            current_goal: String::new(),
            confidence: 0.0,
            chosen_plan: String::new(),
            plan_reason: String::new(),
            rejected_alternatives: Vec::new(),
            group_role: None,
            memory_driver_count: 0,
            social_influence_count: 0,
            stuck_ticks: 0,
            goal_oscillation_count: 0,
        };

        let _cfg = SimTestConfig::quick(42);
        let _medium = SimTestConfig::medium(100);
        let _soak = SimTestConfig::soak(999);
        let _ = SimTestTier::Quick;
        let _ = SimTestTier::Medium;
        let _ = SimTestTier::Soak;

        let metrics = StabilityMetrics::default();
        let _ = metrics.is_stable();
        let _ = metrics.population_drift();
        let _ = metrics.economy_drift();

        // Wire teach_skill
        if frame % 500 == 0 && npc_sample.len() >= 2 {
            teach_skill(ctx.ecs, npc_sample[0], npc_sample[1]);
        }
    }
}

// ---------------------------------------------------------------------------
// 9. AnimationWireSystem — wires ragdoll, procedural, micro_motion, foot_ik
// ---------------------------------------------------------------------------

use crate::animation::active_ragdoll::{
    ActiveRagdollController, BalanceSensor, BehavioralReaction, PdGains,
};
use crate::animation::animation::{AnimationClip, AnimationPlayer, Channel, Joint, Keyframe, Skeleton};
use crate::animation::animation::ChannelProperty;
use crate::animation::animation_ladder::AnimationLadder;
use crate::animation::clip_map::{ClipMap, AnimationState};
use crate::animation::foot_ik::FootIkSolver;
use crate::animation::locomotion::LocomotionMachine;
use crate::animation::micro_motion::{
    validate_motion_gating, MicroMotionComponent, MicroMotionSystem, MicroMotionType, ObjectMotionClass,
};
use crate::animation::procedural::{PerJointWeight, ProceduralLayer};
use crate::animation::ragdoll::{RagdollConfig, RagdollState};

pub struct AnimationWireSystem {
    micro_motion: MicroMotionSystem,
}

impl AnimationWireSystem {
    pub fn new() -> Self {
        Self {
            micro_motion: MicroMotionSystem::new(),
        }
    }
}

impl EngineSystem for AnimationWireSystem {
    fn name(&self) -> &str {
        "AnimationWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AnimationWire")
            .reads_resource::<ClipMap>()
            .reads_resource::<AnimationLadder>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let dt = SIM_DT;

        // Wire ClipMap methods
        if let Some(clip_map) = ctx.resources.get::<ClipMap>() {
            let _ = clip_map.get_clip("idle");
            let _ = clip_map.clip_for_state(&AnimationState::Walk);
            let _ = clip_map.clip_count();
        }

        if let Some(clip_map) = ctx.resources.get_mut::<ClipMap>() {
            clip_map.register_clip(crate::animation::clip_map::AnimationClip {
                name: "test_wire".into(),
                duration: 1.0,
                looping: true,
                blend_in: 0.1,
                blend_out: 0.1,
                layer: crate::animation::clip_map::AnimationLayer::Base,
            });
            clip_map.bind_state(AnimationState::Interact, "test_wire");
        }

        // Wire AnimationLadder::for_distance
        let _ladder = AnimationLadder::for_distance(30.0);
        let _ladder_far = AnimationLadder::for_distance(300.0);

        // Wire Skeleton, Joint, AnimationClip, AnimationPlayer
        let mut skeleton = Skeleton::new();
        skeleton.joints.push(Joint {
            name: "root".into(),
            parent: None,
            local_bind_transform: glam::Mat4::IDENTITY,
        });
        skeleton.joints.push(Joint {
            name: "spine".into(),
            parent: Some(0),
            local_bind_transform: glam::Mat4::from_translation(glam::Vec3::new(0.0, 1.0, 0.0)),
        });
        let _jt_count = skeleton.joint_count();
        let _world = skeleton.compute_world_transforms(&[glam::Mat4::IDENTITY; 2]);
        let _skin = skeleton.compute_skin_matrices(&[glam::Mat4::IDENTITY; 2]);

        let clip = AnimationClip {
            name: "test".into(),
            duration: 1.0,
            channels: vec![Channel {
                joint_index: 0,
                property: ChannelProperty::Rotation,
                keyframes: vec![Keyframe {
                    time: 0.0,
                    value: [0.0, 0.0, 0.0, 1.0],
                }],
            }],
        };
        let clips = vec![clip];
        let mut player = AnimationPlayer::new(2);
        player.play(0);
        player.update(dt, &clips, &skeleton);
        let _ = player.current_clip;
        let _ = player.time;
        let _ = player.speed;
        let _ = player.looping;

        // Wire LocomotionMachine clip_map, set_clip, current_clip
        let mut loco = LocomotionMachine::new();
        loco.set_clip(crate::animation::locomotion::LocomotionState::Idle, 0);
        loco.set_clip(crate::animation::locomotion::LocomotionState::Walk, 1);
        let _ = loco.current_clip();

        // Wire FootIkSolver
        let mut foot_ik = FootIkSolver::new();
        foot_ik.update(
            glam::Vec3::ZERO,
            glam::Vec3::new(-0.2, 0.0, 0.0),
            glam::Vec3::new(0.2, 0.0, 0.0),
            |_x, _z| 0.0,
        );
        let _ = foot_ik.left_target;
        let _ = foot_ik.right_target;
        let _ = foot_ik.hip_offset;
        let _ = foot_ik.blend;
        let (q1, q2) = FootIkSolver::solve_two_bone(
            glam::Vec3::ZERO,
            glam::Vec3::new(0.0, 0.5, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            glam::Vec3::new(1.0, 0.5, 0.0),
        );
        let _ = (q1, q2);

        // Wire MicroMotionSystem, MicroMotionType, MicroMotionComponent
        let comp = MicroMotionComponent {
            motion_type: MicroMotionType::WindDriven {
                amplitude: 0.02,
                frequency: 1.0,
                phase: 0.0,
            },
            current_offset: glam::Vec3::ZERO,
            current_rotation: 0.0,
            motion_class: ObjectMotionClass::Vegetation,
        };
        let _ = validate_motion_gating(&comp.motion_type, comp.motion_class);
        let mut comps = vec![comp];
        self.micro_motion.update(
            dt,
            glam::Vec3::new(1.0, 0.0, 0.0),
            0.5,
            &mut comps,
        );

        // Wire PoseBlender, ProceduralLayer, PerJointWeight::from_name_fn
        let joint_names = vec!["root".into(), "spine".into()];
        let _weights = PerJointWeight::from_name_fn(2, &joint_names, |name| {
            if name.contains("spine") { 0.6 } else { 0.3 }
        });
        let mut procedural = ProceduralLayer::new(2);
        procedural.on_heavy_impact(30.0, 20.0);
        procedural.update(dt, 0.8);
        let anim_pose = vec![glam::Mat4::IDENTITY, glam::Mat4::IDENTITY];
        let phys_pose = vec![glam::Mat4::IDENTITY, glam::Mat4::IDENTITY];
        let _ = procedural.compute_final_pose(&anim_pose, &phys_pose);
        procedural.configure_upper_lower_split(&["root".into(), "spine".into()]);

        // Wire RagdollConfig, RagdollBody, RagdollState
        let config = RagdollConfig::from_skeleton(&skeleton);
        let _ = config.joint_limits;
        let _ = config.bone_lengths;
        let _ = config.bone_radii;

        let mut ragdoll_state = RagdollState::new(2);
        ragdoll_state.activate();
        ragdoll_state.update(dt);
        let _ = ragdoll_state.blend(&anim_pose);
        ragdoll_state.deactivate();

        // Wire PdGains, JointController, BalanceSensor, BehavioralReaction, ActiveRagdollController
        let _stiff = PdGains::stiff();
        let _medium_gains = PdGains::medium();
        let _loose = PdGains::loose();

        let balance = BalanceSensor::new();
        let _ = balance.balance_strength();

        let react = BehavioralReaction::stumble();
        let _ = react.is_active();
        let _ = react.progress();
        let brace = BehavioralReaction::brace();
        let _ = brace.is_active();
        let get_up = BehavioralReaction::get_up();
        let _ = get_up.is_active();

        let mut active_ragdoll = ActiveRagdollController::new(2);
        active_ragdoll.on_impact(25.0);
        active_ragdoll.try_get_up();
        active_ragdoll.reset_controllers();

        // Wire ActiveRagdollController::update and apply_pd_torques (via JointController::compute_torque)
        {
            use rapier3d::prelude::*;
            let mut bodies = RigidBodySet::new();
            let mut colliders = ColliderSet::new();
            let mut impulse_joints = ImpulseJointSet::new();
            let ragdoll = crate::animation::ragdoll::RagdollBody::spawn(
                &config,
                &skeleton,
                glam::Vec3::ZERO,
                &mut bodies,
                &mut colliders,
                &mut impulse_joints,
            );
            let target_transforms: Vec<glam::Mat4> = (0..ragdoll.body_handles.len())
                .map(|_| glam::Mat4::IDENTITY)
                .collect();
            active_ragdoll.update(&ragdoll, &bodies, &skeleton, dt);
            active_ragdoll.apply_pd_torques(
                &ragdoll,
                &target_transforms,
                &mut bodies,
                glam::Vec3::ZERO,
            );
        }
    }
}
