//! Body System — Physical Character Response Stack (Contract 6)
//!
//! Integrates PersonalNeeds.health with BodyState zones, HitLocation->zone damage,
//! pain/bleed/consciousness tracking, broken limbs, and Contract 6 response levels.
//! Low-spec: full response only within 30m of camera.

use std::collections::HashMap;

use crate::body::anatomy::{BleedPoint, BodyState};
use crate::body::body_response::BodyPhysicalResponseCache as BodyResponseCache;
use crate::body::body_store::BodyStateStore;
use crate::body::death_pipeline::CorpseManager;
use crate::core::ecs::Entity;
use crate::core::events::canonical::{BodyZoneDamaged, CombatHit, EntityDied, GoreMeshSpawn, SoundTrigger, SoundTriggerKind};
use crate::core::mutation_policy::FixedTickContext;
use crate::core::registry::Resources;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;
use crate::physics::damage_pipeline::response_aggregator::BodyZone;
use crate::world::components::{SimLevel, SimulationLevel};

const FULL_RESPONSE_RADIUS: f32 = 30.0;
const SIM_DT: f32 = 1.0 / 20.0;
const PAIN_FLEE_THRESHOLD: f32 = 0.7;
const PAIN_TO_FEAR_RATE: f32 = 0.4;
const BROKEN_LIMB_THRESHOLD: f32 = 30.0;
const BLEED_DAMAGE_THRESHOLD: f32 = 3.0;
const HEAL_EPSILON: f32 = 0.02;

/// Camera/observer position for low-spec distance culling.
#[derive(Clone, Debug, Default)]
pub struct ObserverPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ObserverPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Per-entity physical response modifiers for movement and combat.
#[derive(Clone, Debug)]
pub struct BodyPhysicalResponse {
    pub movement_speed_mult: f32,
    pub combat_power_mult: f32,
    pub pain: f32,
    pub consciousness: f32,
    pub response_level: u8, // L0..L3
}

/// Cache of body physical responses for consumers (AI, movement, combat).
#[derive(Clone, Debug, Default)]
pub struct BodyPhysicalResponseCache {
    pub responses: HashMap<Entity, BodyPhysicalResponse>,
}

/// Combined state for BodySystem to avoid multiple resource borrows.
pub struct BodySystemState {
    pub store: BodyStateStore,
    pub cache: BodyPhysicalResponseCache,
}

impl Default for BodySystemState {
    fn default() -> Self {
        Self {
            store: BodyStateStore::new(),
            cache: BodyPhysicalResponseCache::default(),
        }
    }
}

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

/// Map CombatHit hit_zone (0=Head, 1=Torso, 2=Arms, 3=Legs) to BodyZones.
/// Arms -> both arms (half damage each), Legs -> both legs (half damage each).
fn combat_hit_zones(hit_zone: u8) -> Vec<(BodyZone, f32)> {
    match hit_zone {
        0 => vec![(BodyZone::Head, 1.0)],
        1 => vec![(BodyZone::Torso, 1.0)],
        2 => vec![(BodyZone::LeftArm, 0.5), (BodyZone::RightArm, 0.5)],
        3 => vec![(BodyZone::LeftLeg, 0.5), (BodyZone::RightLeg, 0.5)],
        _ => vec![(BodyZone::Torso, 1.0)],
    }
}

fn body_zone_to_store_index(zone: BodyZone) -> usize {
    match zone {
        BodyZone::Head => 0,
        BodyZone::Neck => 1,
        BodyZone::Torso => 2,
        BodyZone::LeftArm => 3,
        BodyZone::RightArm => 4,
        BodyZone::Pelvis => 5,
        BodyZone::LeftLeg => 6,
        BodyZone::RightLeg => 7,
    }
}

/// Apply damage to a zone, update trauma/pain/consciousness, add bleed points, mark broken joints.
fn apply_zone_damage_impl(body: &mut BodyState, zone: BodyZone, raw_damage: f32) {
    let zone_idx = body_zone_to_store_index(zone);
    if zone_idx >= body.zones.len() {
        return;
    }
    let zs = &mut body.zones[zone_idx];
    zs.integrity = (zs.integrity - raw_damage).max(0.0);
    zs.trauma += raw_damage * 0.5;
    body.total_trauma += raw_damage * 0.3;
    body.consciousness = (body.consciousness - raw_damage * 0.005).max(0.0);
    body.pain = (body.pain + raw_damage * 0.01).min(1.0);

    if raw_damage >= BLEED_DAMAGE_THRESHOLD {
        body.bleed_points.push(BleedPoint {
            zone,
            rate: raw_damage * 0.05,
            time_active: 0.0,
        });
    }

    if zs.integrity < BROKEN_LIMB_THRESHOLD {
        for joint in &mut body.joints {
            if joint.zone == zone {
                joint.broken = true;
                joint.integrity = joint.integrity.min(zs.integrity);
            }
        }
    }
}

/// Propagate healing from PersonalNeeds.health into BodyState zones proportionally.
fn apply_healing_to_body(body: &mut BodyState, target_aggregate: f32) {
    let current = body.aggregate_health();
    if target_aggregate <= current + HEAL_EPSILON {
        return;
    }
    let heal_amount = target_aggregate - current;
    let total_restorable = body.zones.iter().map(|z| (100.0 - z.integrity).max(0.0)).sum::<f32>()
        + (1.0 - body.blood_level).max(0.0) * 50.0;
    if total_restorable < 0.01 {
        return;
    }
    let frac = (heal_amount / total_restorable).min(1.0);
    for zs in &mut body.zones {
        let space = (100.0 - zs.integrity).max(0.0);
        zs.integrity = (zs.integrity + space * frac).min(100.0);
    }
    if body.blood_level < 1.0 {
        body.blood_level = (body.blood_level + (1.0 - body.blood_level) * frac * 0.5).min(1.0);
    }
}

pub struct BodySystem;

impl BodySystem {
    fn ensure_body(store: &mut BodyStateStore, entity: Entity) -> Option<u32> {
        if let Some(id) = store.store_id_for_entity(entity) {
            return Some(id);
        }
        let body = BodyState::new_humanoid(entity);
        let handle = store.allocate(body);
        Some(handle.store_id)
    }

    fn distance_to_observer(transform_x: f32, transform_y: f32, obs: &ObserverPosition) -> f32 {
        let dx = transform_x - obs.x;
        let dz = transform_y - obs.z;
        (dx * dx + dz * dz).sqrt()
    }

    fn compute_response_level(sim_level: Option<&SimLevel>, in_range: bool) -> u8 {
        if !in_range {
            return 0; // Minimal when out of range
        }
        match sim_level.map(|s| s.level) {
            Some(SimulationLevel::L0) => 0, // L0: hit reactions, full
            Some(SimulationLevel::L1) => 1,   // L1: injury-aware motion
            Some(SimulationLevel::L2) => 2,  // L2: procedural balance
            Some(SimulationLevel::L3) => 3,  // L3: death/ragdoll
            None => 0,
        }
    }

    fn compute_limb_modifiers(body: &BodyState) -> (f32, f32) {
        let mut move_mult = 1.0_f32;
        let mut combat_mult = 1.0_f32;

        for joint in &body.joints {
            if joint.broken {
                match joint.zone {
                    BodyZone::LeftLeg | BodyZone::RightLeg => move_mult *= 0.6,
                    BodyZone::LeftArm | BodyZone::RightArm => combat_mult *= 0.7,
                    _ => {}
                }
            }
        }

        for zs in &body.zones {
            if zs.integrity < 20.0 {
                match zs.zone {
                    BodyZone::LeftLeg | BodyZone::RightLeg => move_mult *= 0.8,
                    BodyZone::LeftArm | BodyZone::RightArm => combat_mult *= 0.85,
                    _ => {}
                }
            }
        }

        (move_mult.max(0.2), combat_mult.max(0.3))
    }
}

impl EngineSystem for BodySystem {
    fn name(&self) -> &str {
        "BodySystem"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("BodySystem")
            .reads_resource::<BodySystemState>()
            .writes_resource::<BodySystemState>()
            .reads_resource::<ObserverPosition>()
            .reads_event::<BodyZoneDamaged>()
            .reads_event::<CombatHit>()
    }

    fn register_resources(&mut self, res: &mut Resources) {
        if !res.contains::<BodySystemState>() {
            res.insert(BodySystemState::default());
        }
        if !res.contains::<ObserverPosition>() {
            res.insert(ObserverPosition::default());
        }
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let obs = ctx
            .resources
            .get::<ObserverPosition>()
            .map(|o| (o.x, o.y, o.z))
            .unwrap_or((0.0, 0.0, 0.0));

        let Some(state) = ctx.resources.get_mut::<BodySystemState>() else {
            return;
        };

        let (store, cache) = (&mut state.store, &mut state.cache);
        cache.responses.clear();

        // 1. Process BodyZoneDamaged (physics/ballistics)
        let damaged: Vec<BodyZoneDamaged> = ctx
            .events
            .read::<BodyZoneDamaged>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &damaged {
            let _ = Self::ensure_body(store, ev.entity);
            if let Some(body) = store.get_mut_by_entity(ev.entity) {
                let zone = u8_to_body_zone(ev.zone);
                let raw = ev.damage * 0.01_f32;
                apply_zone_damage_impl(body, zone, raw);
            }
        }

        // 2. Process CombatHit (HitLocation -> zone integrity)
        let combat_hits: Vec<CombatHit> = ctx
            .events
            .read::<CombatHit>()
            .iter()
            .map(|r| (*r).clone())
            .collect();

        for ev in &combat_hits {
            let _ = Self::ensure_body(store, ev.entity);
            let zones = combat_hit_zones(ev.hit_zone);
            for (zone, frac) in zones {
                if let Some(body) = store.get_mut_by_entity(ev.entity) {
                    let raw = ev.damage * frac * 10.0; // Scale to zone integrity
                    apply_zone_damage_impl(body, zone, raw);
                }
            }
        }

        // 3. Tick body store (bleed, pain decay)
        store.tick(SIM_DT);

        // 4. Sync PersonalNeeds <-> BodyState, apply pain->fear, compute modifiers
        let entities: Vec<Entity> = ctx.ecs.alive.clone();
        for entity in entities {
let (transform_x, transform_y) = ctx
 .ecs
 .get_transform(entity)
 .map(|t| (t.x, t.y))
                .unwrap_or((0.0, 0.0));

            let obs_pos = ObserverPosition { x: obs.0, y: obs.1, z: obs.2 };
            let dist = Self::distance_to_observer(transform_x, transform_y, &obs_pos);
            let in_range = dist <= FULL_RESPONSE_RADIUS;

            let has_personal = ctx.ecs.get_needs(entity).is_some();

            if !has_personal {
                continue;
            }

            let store_id = if in_range {
                Self::ensure_body(store, entity)
            } else {
                store.store_id_for_entity(entity)
            };

            if let Some(sid) = store_id {
                if let Some(body) = store.get_mut(sid) {
                    // Read PersonalNeeds.health changes: propagate healing to BodyState
                    if let Some(pn) = ctx.ecs.get_needs(entity) {
                        if pn.health > body.aggregate_health() + HEAL_EPSILON {
                            apply_healing_to_body(body, pn.health);
                        }
                    }
                    // Sync aggregate health -> PersonalNeeds
                    let agg = body.aggregate_health();
                    if let Some(pn) = ctx.ecs.get_needs_mut(entity) {
                        pn.health = agg;
                        // Pain > 0.7 -> increase fear for flee (Contract 6: NPC AI)
                        if body.pain > PAIN_FLEE_THRESHOLD {
                            pn.fear = (pn.fear + PAIN_TO_FEAR_RATE * (body.pain - PAIN_FLEE_THRESHOLD))
                                .min(1.0);
                        }
                    }

                    let (move_mult, combat_mult) = Self::compute_limb_modifiers(body);
                    let health_response = BodyResponseCache::compute_from_health(
                        body.aggregate_health(),
                        body.blood_level,
                        body.pain,
                    );
                    let move_mult = move_mult.min(health_response.movement_speed_mult);
                    let combat_mult = combat_mult.min(health_response.combat_power_mult);
                    let sim_level = ctx.ecs.get_sim_level(entity);
                    let response_level = Self::compute_response_level(sim_level, in_range);

                    cache.responses.insert(
                        entity,
                        BodyPhysicalResponse {
                            movement_speed_mult: move_mult,
                            combat_power_mult: combat_mult,
                            pain: body.pain,
                            consciousness: body.consciousness,
                            response_level,
                        },
                    );
                }
            } else if in_range {
                let body = BodyState::new_humanoid(entity);
                let agg = body.aggregate_health();
                if let Some(pn) = ctx.ecs.get_needs_mut(entity) {
                    pn.health = agg;
                }
                let _ = store.allocate(body);
            }
        }

        // 5. Detect deaths (health <= 0) and register corpses
        let mut dead_entities = Vec::new();
        for &entity in &ctx.ecs.alive {
            if let Some(pn) = ctx.ecs.get_needs(entity) {
                if pn.health <= 0.0 {
let pos = ctx
 .ecs
 .get_transform(entity)
 .map(|t| [t.x, t.y])
                        .unwrap_or([0.0, 0.0]);
let items: Vec<String> = ctx
 .ecs
 .get_inventory(entity)
 .map(|inv| inv.items.iter().map(|it| it.name.clone()).collect())
                        .unwrap_or_default();
                    dead_entities.push((entity, pos, items));
                }
            }
        }

        if !dead_entities.is_empty() {
            if let Some(corpse_mgr) = ctx.resources.get_mut::<CorpseManager>() {
                for (entity, pos, items) in &dead_entities {
                    corpse_mgr.register_death(*entity as u64, *pos, "combat", items.clone());
                }
            }
            for (entity, pos, _) in &dead_entities {
                let world_pos = glam::Vec3::new(pos[0], 0.0, pos[1]);
                ctx.events.emit(EntityDied {
                    entity: *entity,
                    position: world_pos,
                    killer: None,
                });
                ctx.events.emit(GoreMeshSpawn {
                    entity: *entity,
                    zone: 2, // torso
                    position: world_pos,
                    intensity: 1.0,
                });
                ctx.events.emit(SoundTrigger {
                    position: world_pos,
                    kind: SoundTriggerKind::Pain,
                    volume: 0.8,
                });
                ctx.ecs.despawn(*entity);
            }
        }

        // 6. Tick corpse decay
        if let Some(corpse_mgr) = ctx.resources.get_mut::<CorpseManager>() {
            corpse_mgr.tick_all(SIM_DT);
        }
    }
}
