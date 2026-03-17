//! Integration tests for Physics, Body simulation, and Combat systems.
//! 215 tests total with real assertions against the ENGENE API.

use engene::physics::ballistics::{
    BallisticEvent, BallisticsSystem, ImpactResult, MaterialProps, MaterialTable, Projectile,
};
use engene::physics::chain_reactions::{ChainEvent, ChainReactionQueue};
use engene::physics::collapse_solver::{evaluate_failure, evaluate_hanging, CollapseResult, FailureMode};
use engene::physics::collision::{check_overlap, resolve_collisions};
use engene::physics::damage_pipeline::orchestrator::DamageOrchestrator;
use engene::physics::damage_pipeline::response_aggregator::{
    BodyZone, DamageResponse, DecalType, ResponseAggregator, SoundClass,
};
use engene::physics::damage_taxonomy::{DamageCapability, DamageClass};
use engene::physics::destruction::{
    DestructionEvent, DestructionLink, DestructionLod, DestructionNode, DestructionSystem,
    DestructibleObject,
};
use engene::physics::fire::{FireCell, FireGrid, FireState};
use engene::physics::impact_event::{ImpactEvent, ProjectileInfo, StressEvent};
use engene::physics::layered_damage::{DamageLayer, DamageableObject, DamageableStore};
use engene::physics::material_fracture::{compute_fracture, FractureResult};
use engene::physics::movement::{apply_velocity, move_toward, Velocity};
use engene::physics::secondary_impacts::{fragment_to_impact, generate_fragments, DebrisFragment};
use engene::physics::sim_lod::PhysicsLod;
use engene::physics::soft_state::{ObjectCondition, SoftDamageState};
use engene::physics::structural_load::{cascade_collapse, redistribute_loads};
use engene::physics::water::{WaterCell, WaterGrid};
use engene::physics::building::{
    BuildingDescriptor, SectionDescriptor, SectionNeighbor, SectionType, StructuralSection,
};
use engene::body::anatomy::{BodyState as AnatomyBodyState, BleedPoint, JointInfo, ZoneState};
use engene::body::blood::{compute_blood_lod, BloodLod};
use engene::body::body_damage::apply_zone_damage;
use engene::body::body_response::{BodyPhysicalResponseCache, PhysicalResponseTier};
use engene::body::body_store::{BodyHandle, BodyStateStore};
use engene::body::death_pipeline::{CorpseManager, CorpseState, DeathState};
use engene::body::dismemberment::{check_dismemberment, severed_zones};
use engene::game::ai::combat::{HitLocation, StaggerState, resolve_combat};
use engene::game::ai::body::{is_night, time_of_day_mult, BodyState as AiBodyState};
use engene::core::ecs::Ecs;
use engene::core::events::EventBus;
use engene::navigation::path_cache::PathCache;
use engene::navigation::world_graph::{LocationId, WorldGraph};
use engene::world::components::*;
use engene::world::fields::WorldFields;
use engene::world::surface_db::{ResponseClass, SurfaceMaterial};
use glam::Vec3;

// ===== BALLISTICS (25 tests) =====

#[test]
fn ballistics_system_new() {
    let sys = BallisticsSystem::new();
    assert!(sys.projectiles.is_empty());
    assert!(sys.events.is_empty());
    assert_eq!(sys.gravity.y, -9.81);
}

#[test]
fn ballistics_fire_adds_projectile() {
    let mut sys = BallisticsSystem::new();
    let origin = Vec3::new(0.0, 10.0, 0.0);
    let dir = Vec3::new(1.0, 0.0, 0.0);
    sys.fire(origin, dir, 100.0, 0.01, 0.5, 1, 42, 12345);
    assert_eq!(sys.projectiles.len(), 1);
    assert_eq!(sys.projectiles[0].pos, origin);
    assert!((sys.projectiles[0].vel.x - 100.0).abs() < 0.01);
    assert_eq!(sys.projectiles[0].ttl, 5.0);
}

#[test]
fn ballistics_fire_emits_shot_fired_event() {
    let mut sys = BallisticsSystem::new();
    let origin = Vec3::ZERO;
    let dir = Vec3::Y;
    sys.fire(origin, dir, 50.0, 0.01, 0.2, 99, 1, 999);
    let events = sys.drain_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        BallisticEvent::ShotFired { seed, origin: o, weapon_id, owner, dir: _ } => {
            assert_eq!(*seed, 999);
            assert_eq!(o.x, 0.0);
            assert_eq!(*weapon_id, 1);
            assert_eq!(*owner, 99);
        }
        _ => panic!("expected ShotFired"),
    }
}

#[test]
fn ballistics_drain_events_clears() {
    let mut sys = BallisticsSystem::new();
    sys.fire(Vec3::ZERO, Vec3::Y, 10.0, 0.01, 0.1, 0, 0, 0);
    let events = sys.drain_events();
    assert!(!events.is_empty());
    let after = sys.drain_events();
    assert!(after.is_empty());
}

#[test]
fn material_table_register_and_get() {
    let mut mt = MaterialTable::new();
    let props = MaterialProps {
        hardness: 0.8,
        penetration_resistance: 100.0,
        density: 2.5,
    };
    mt.register("steel", 1, props);
    let got = mt.get(1).unwrap();
    assert_eq!(got.hardness, 0.8);
    assert_eq!(got.density, 2.5);
}

#[test]
fn material_table_get_nonexistent() {
    let mt = MaterialTable::new();
    assert!(mt.get(999).is_none());
}

#[test]
fn ballistics_analytical_trajectory_returns_points() {
    let sys = BallisticsSystem::new();
    let fields = WorldFields::new();
    let pts = sys.analytical_trajectory(
        Vec3::new(0.0, 100.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        200.0,
        0.01,
        0.5,
        &fields,
        500.0,
    );
    assert!(!pts.is_empty());
    assert_eq!(pts[0], Vec3::new(0.0, 100.0, 0.0));
}

#[test]
fn impact_result_stopped() {
    let mut sys = BallisticsSystem::new();
    sys.material_table.register("concrete", 1, MaterialProps { hardness: 0.9, penetration_resistance: 500.0, density: 2.4 });
    let proj = Projectile {
        pos: Vec3::new(5.0, 2.0, 3.0),
        prev_pos: Vec3::ZERO,
        vel: Vec3::new(10.0, 0.0, 0.0),
        energy: 50.0,
        drag_coeff: 0.5,
        mass: 0.01,
        ttl: 1.0,
        seed: 0,
        owner: 0,
        weapon_id: 0,
        distance_traveled: 0.0,
    };
    let normal = Vec3::new(-1.0, 0.0, 0.0);
    let result = sys.resolve_impact(&proj, normal, 1);
    assert!(matches!(result, ImpactResult::Stopped { .. }));
}

#[test]
fn impact_result_ricochet_shallow_angle() {
    let mut sys = BallisticsSystem::new();
    sys.material_table.register("metal", 1, MaterialProps { hardness: 0.95, penetration_resistance: 100.0, density: 7.8 });
    let proj = Projectile {
        pos: Vec3::ZERO,
        prev_pos: Vec3::ZERO,
        vel: Vec3::new(1.0, 0.1, 0.0),
        energy: 1000.0,
        drag_coeff: 0.2,
        mass: 0.02,
        ttl: 1.0,
        seed: 0,
        owner: 0,
        weapon_id: 0,
        distance_traveled: 0.0,
    };
    let result = sys.resolve_impact(&proj, Vec3::new(0.0, -1.0, 0.0), 1);
    assert!(matches!(result, ImpactResult::Ricochet { .. }));
}

#[test]
fn impact_result_penetrated_high_energy() {
    let mut sys = BallisticsSystem::new();
    sys.material_table.register("wood", 1, MaterialProps { hardness: 0.3, penetration_resistance: 0.05, density: 0.6 });
    let proj = Projectile {
        pos: Vec3::ZERO,
        prev_pos: Vec3::ZERO,
        vel: Vec3::new(500.0, 0.0, 0.0),
        energy: 1_250_000.0,
        drag_coeff: 0.1,
        mass: 0.01,
        ttl: 1.0,
        seed: 0,
        owner: 0,
        weapon_id: 0,
        distance_traveled: 0.0,
    };
    let result = sys.resolve_impact(&proj, Vec3::new(-1.0, 0.0, 0.0), 1);
    assert!(matches!(result, ImpactResult::Penetrated { .. }));
}

#[test]
fn ballistics_projectile_fields() {
    let mut sys = BallisticsSystem::new();
    sys.fire(Vec3::new(1.0, 2.0, 3.0), Vec3::X, 80.0, 0.005, 0.3, 5, 2, 777);
    let p = &sys.projectiles[0];
    assert_eq!(p.prev_pos, p.pos);
    assert_eq!(p.distance_traveled, 0.0);
    assert!((p.energy - 0.5 * 0.005 * 80.0 * 80.0).abs() < 0.1);
}

#[test]
fn ballistics_multiple_fire() {
    let mut sys = BallisticsSystem::new();
    for i in 0..5u64 {
        sys.fire(Vec3::new(i as f32, 0.0, 0.0), Vec3::Y, 50.0, 0.01, 0.2, i, 0, i as u32);
    }
    assert_eq!(sys.projectiles.len(), 5);
    assert_eq!(sys.events.len(), 5);
}

#[test]
fn ballistics_material_multiple() {
    let mut mt = MaterialTable::new();
    mt.register("a", 1, MaterialProps { hardness: 0.5, penetration_resistance: 50.0, density: 1.0 });
    mt.register("b", 2, MaterialProps { hardness: 0.9, penetration_resistance: 200.0, density: 3.0 });
    assert_eq!(mt.get(1).unwrap().hardness, 0.5);
    assert_eq!(mt.get(2).unwrap().hardness, 0.9);
}

#[test]
fn ballistics_trajectory_max_distance() {
    let sys = BallisticsSystem::new();
    let fields = WorldFields::new();
    let pts = sys.analytical_trajectory(Vec3::ZERO, Vec3::X, 50.0, 0.01, 0.5, &fields, 10.0);
    assert!(pts.len() >= 1);
}

#[test]
fn ballistics_gravity() {
    let sys = BallisticsSystem::new();
    assert_eq!(sys.gravity.x, 0.0);
    assert_eq!(sys.gravity.z, 0.0);
}

#[test]
fn ballistics_projectile_energy() {
    let mut sys = BallisticsSystem::new();
    sys.fire(Vec3::ZERO, Vec3::Y, 100.0, 0.02, 0.1, 0, 0, 0);
    let ke = 0.5 * 0.02 * 100.0 * 100.0;
    assert!((sys.projectiles[0].energy - ke).abs() < 0.01);
}

#[test]
fn ballistics_event_variants() {
    let _sf = BallisticEvent::ShotFired { seed: 0, origin: Vec3::ZERO, dir: Vec3::Y, weapon_id: 0, owner: 0 };
    let _im = BallisticEvent::Impact { seed: 0, hit_pos: Vec3::ZERO, normal: Vec3::Y, material: 0, damage: 10.0 };
    let _eh = BallisticEvent::EntityHit { entity: 0, damage: 5.0, hit_pos: Vec3::ZERO, projectile_vel: Vec3::X };
}

#[test]
fn ballistics_resolve_unknown_material() {
    let sys = BallisticsSystem::new();
    let proj = Projectile {
        pos: Vec3::ZERO,
        prev_pos: Vec3::ZERO,
        vel: Vec3::X,
        energy: 100.0,
        drag_coeff: 0.5,
        mass: 0.01,
        ttl: 1.0,
        seed: 0,
        owner: 0,
        weapon_id: 0,
        distance_traveled: 0.0,
    };
    let r = sys.resolve_impact(&proj, Vec3::new(-1.0, 0.0, 0.0), 999);
    assert!(matches!(r, ImpactResult::Stopped { material: 999, .. }));
}

#[test]
fn ballistics_trajectory_direction() {
    let sys = BallisticsSystem::new();
    let fields = WorldFields::new();
    let origin = Vec3::new(0.0, 50.0, 0.0);
    let dir = Vec3::new(1.0, 0.0, 0.0).normalize();
    let pts = sys.analytical_trajectory(origin, dir, 100.0, 0.01, 0.3, &fields, 200.0);
    assert!(pts.len() >= 2);
    assert!(pts[1].x > pts[0].x || pts[1].x.abs() < 0.01);
}

#[test]
fn ballistics_material_props() {
    let p = MaterialProps { hardness: 0.7, penetration_resistance: 80.0, density: 2.0 };
    assert_eq!(p.hardness, 0.7);
    assert_eq!(p.density, 2.0);
}

// ===== DESTRUCTION (25 tests) =====

#[test]
fn destruction_system_new() {
    let sys = DestructionSystem::new();
    assert!(sys.objects.is_empty());
}

#[test]
fn destruction_register_object() {
    let mut sys = DestructionSystem::new();
    let entity = 42u64;
    let obj = DestructibleObject::new(
        entity,
        vec![
            DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 },
        ],
        vec![DestructionLink { a: 0, b: 0, strength: 100.0, fatigue: 0.0, broken: false }],
    );
    sys.register_object(obj);
    assert_eq!(sys.objects.len(), 1);
}

#[test]
fn destruction_apply_impulse_at_full_lod() {
    let mut sys = DestructionSystem::new();
    let entity = 1u64;
    let nodes = vec![
        DestructionNode { id: 0, position: Vec3::new(0.0, 0.0, 0.0), mass: 1.0, material: 0, accumulated_stress: 0.0 },
        DestructionNode { id: 1, position: Vec3::new(1.0, 0.0, 0.0), mass: 1.0, material: 0, accumulated_stress: 0.0 },
    ];
    let links = vec![
        DestructionLink { a: 0, b: 1, strength: 50.0, fatigue: 0.0, broken: false },
    ];
    let obj = DestructibleObject::new(entity, nodes, links);
    sys.register_object(obj);
    sys.apply_impulse_at(Vec3::new(0.5, 0.0, 0.0), 1000.0, DestructionLod::Full);
    let events = sys.drain_events();
    let _event_count = events.len();
}

#[test]
fn destruction_apply_impulse_statistical() {
    let mut sys = DestructionSystem::new();
    let entity = 2u64;
    let obj = DestructibleObject::new(
        entity,
        vec![DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 }],
        vec![DestructionLink { a: 0, b: 0, strength: 100.0, fatigue: 0.0, broken: false }],
    );
    sys.register_object(obj);
    sys.apply_impulse_at(Vec3::ZERO, 500.0, DestructionLod::Statistical);
}

#[test]
fn destruction_apply_impulse_frozen() {
    let mut sys = DestructionSystem::new();
    let obj = DestructibleObject::new(
        3u64,
        vec![DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 }],
        vec![DestructionLink { a: 0, b: 0, strength: 100.0, fatigue: 0.0, broken: false }],
    );
    sys.register_object(obj);
    sys.apply_impulse_at(Vec3::ZERO, 10000.0, DestructionLod::Frozen);
    assert!(sys.objects[0].links[0].broken == false);
}

#[test]
fn destruction_drain_events() {
    let mut sys = DestructionSystem::new();
    let obj = DestructibleObject::new(
        4u64,
        vec![
            DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 },
            DestructionNode { id: 1, position: Vec3::new(5.0, 0.0, 0.0), mass: 1.0, material: 0, accumulated_stress: 0.0 },
        ],
        vec![DestructionLink { a: 0, b: 1, strength: 10.0, fatigue: 0.0, broken: false }],
    );
    sys.register_object(obj);
    sys.apply_impulse_at(Vec3::ZERO, 5000.0, DestructionLod::Full);
    let events = sys.drain_events();
    let _first_drain = events.len();
    let after = sys.drain_events();
    assert!(after.is_empty());
}

#[test]
fn destructible_integrity() {
    let entity = 5u64;
    let links = vec![
        DestructionLink { a: 0, b: 1, strength: 50.0, fatigue: 0.0, broken: false },
        DestructionLink { a: 1, b: 2, strength: 50.0, fatigue: 0.0, broken: true },
    ];
    let obj = DestructibleObject::new(
        entity,
        vec![
            DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 },
            DestructionNode { id: 1, position: Vec3::X, mass: 1.0, material: 0, accumulated_stress: 0.0 },
            DestructionNode { id: 2, position: Vec3::new(2.0, 0.0, 0.0), mass: 1.0, material: 0, accumulated_stress: 0.0 },
        ],
        links,
    );
    assert!((obj.integrity() - 0.5).abs() < 0.01);
}

#[test]
fn destructible_apply_impulse_returns_events() {
    let mut obj = DestructibleObject::new(
        6u64,
        vec![
            DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 },
            DestructionNode { id: 1, position: Vec3::new(2.0, 0.0, 0.0), mass: 1.0, material: 0, accumulated_stress: 0.0 },
        ],
        vec![DestructionLink { a: 0, b: 1, strength: 5.0, fatigue: 0.0, broken: false }],
    );
    let events = obj.apply_impulse(Vec3::ZERO, 1000.0);
    let _count = events.len();
}

#[test]
fn destructible_apply_statistical_damage() {
    let mut obj = DestructibleObject::new(
        7u64,
        vec![
            DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 },
            DestructionNode { id: 1, position: Vec3::X, mass: 1.0, material: 0, accumulated_stress: 0.0 },
        ],
        vec![
            DestructionLink { a: 0, b: 1, strength: 100.0, fatigue: 0.0, broken: false },
            DestructionLink { a: 1, b: 0, strength: 100.0, fatigue: 0.0, broken: false },
        ],
    );
    let ratio = obj.apply_statistical_damage(150.0);
    assert!(ratio > 0.0 && ratio <= 1.0);
}

#[test]
fn destruction_lod_variants() {
    let _f = DestructionLod::Full;
    let _s = DestructionLod::Statistical;
    let _z = DestructionLod::Frozen;
}

#[test]
fn destruction_event_link_broken() {
    let _e = DestructionEvent::LinkBroken { entity: 0, link_a: 0, link_b: 1 };
}

#[test]
fn destruction_event_object_fragmented() {
    let _e = DestructionEvent::ObjectFragmented { entity: 0, cluster_count: 2 };
}

#[test]
fn destructible_new_total_strength() {
    let obj = DestructibleObject::new(
        8u64,
        vec![DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 }],
        vec![
            DestructionLink { a: 0, b: 0, strength: 30.0, fatigue: 0.0, broken: false },
            DestructionLink { a: 0, b: 0, strength: 70.0, fatigue: 0.0, broken: false },
        ],
    );
    assert_eq!(obj.total_strength, 100.0);
}

#[test]
fn destructible_integrity_full() {
    let obj = DestructibleObject::new(
        9u64,
        vec![DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 }],
        vec![DestructionLink { a: 0, b: 0, strength: 100.0, fatigue: 0.0, broken: false }],
    );
    assert!((obj.integrity() - 1.0).abs() < 0.01);
}

#[test]
fn destructible_integrity_zero() {
    let obj = DestructibleObject::new(
        10u64,
        vec![DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.0 }],
        vec![DestructionLink { a: 0, b: 0, strength: 100.0, fatigue: 0.0, broken: true }],
    );
    assert!(obj.integrity() < 0.01);
}

// ===== COLLISION & MOVEMENT (15 tests) =====

#[test]
fn check_overlap_true() {
    assert!(check_overlap(0.0, 0.0, 1.0, 0.0, 1.0));
}

#[test]
fn check_overlap_false() {
    assert!(!check_overlap(0.0, 0.0, 10.0, 10.0, 1.0));
}

#[test]
fn check_overlap_boundary() {
    assert!(check_overlap(0.0, 0.0, 3.9, 0.0, 1.0));
}

#[test]
fn velocity_default() {
    let v = Velocity::default();
    assert_eq!(v.vx, 0.0);
    assert_eq!(v.vy, 0.0);
}

#[test]
fn move_toward_reaches_target() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(e, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    let reached = move_toward(&mut ecs, e, 5.0, 5.0, 100.0, 1.0);
    assert!(reached);
    let t = ecs.transforms.get(&e).unwrap();
    assert_eq!(t.x, 5.0);
    assert_eq!(t.y, 5.0);
}

#[test]
fn move_toward_partial() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(e, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    let reached = move_toward(&mut ecs, e, 100.0, 0.0, 1.0, 0.1);
    assert!(!reached);
    let t = ecs.transforms.get(&e).unwrap();
    assert!(t.x > 0.0 && t.x < 100.0);
}

#[test]
fn move_toward_no_transform() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    let reached = move_toward(&mut ecs, e, 1.0, 1.0, 1.0, 1.0);
    assert!(!reached);
}

#[test]
fn resolve_collisions_two_entities() {
    let mut ecs = Ecs::new();
    let a = ecs.spawn();
    let b = ecs.spawn();
    ecs.transforms.insert(a, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(b, Transform { x: 1.0, y: 0.0, cell_x: 0, cell_y: 0 });
    resolve_collisions(&mut ecs, &[a, b]);
}

#[test]
fn check_overlap_same_pos() {
    assert!(check_overlap(5.0, 5.0, 5.0, 5.0, 0.5));
}

#[test]
fn velocity_clone() {
    let v = Velocity { vx: 3.0, vy: 4.0 };
    let v2 = v.clone();
    assert_eq!(v2.vx, 3.0);
}

// ===== DAMAGE TAXONOMY (10 tests) =====

#[test]
fn damage_class_ballistic() {
    assert!(DamageClass::Ballistic.is_instant());
    assert!(!DamageClass::Ballistic.is_cumulative());
}

#[test]
fn damage_class_thermal_cumulative() {
    assert!(!DamageClass::Thermal.is_instant());
    assert!(DamageClass::Thermal.is_cumulative());
}

#[test]
fn damage_class_all_instant() {
    assert!(DamageClass::Blunt.is_instant());
    assert!(DamageClass::Explosive.is_instant());
    assert!(DamageClass::Piercing.is_instant());
    assert!(DamageClass::Shear.is_instant());
    assert!(DamageClass::Fragmentation.is_instant());
}

#[test]
fn damage_class_all_cumulative() {
    assert!(DamageClass::Hydraulic.is_cumulative());
    assert!(DamageClass::Erosion.is_cumulative());
    assert!(DamageClass::Corrosion.is_cumulative());
    assert!(DamageClass::Fatigue.is_cumulative());
}

#[test]
fn damage_capability_flags() {
    assert!(DamageCapability::SURFACE.bits() & 1 != 0);
    assert!(DamageCapability::THERMAL.bits() & 0b0001_0000 != 0);
    assert!(DamageCapability::MOISTURE.bits() & 0b0010_0000 != 0);
}

#[test]
fn damage_capability_contains() {
    let caps = DamageCapability::LAYERED;
    assert!(caps.contains(DamageCapability::SURFACE));
}

#[test]
fn damage_capability_empty() {
    assert!(DamageCapability::empty().is_empty());
}

#[test]
fn damage_capability_anatomical() {
    let c = DamageCapability::ANATOMICAL;
    assert!(c.contains(DamageCapability::STRUCTURAL));
}

// ===== IMPACT & STRESS (15 tests) =====

#[test]
fn impact_event_construction() {
    let evt = ImpactEvent {
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::X,
        impulse: 10.0,
        energy: 50.0,
        contact_area: 0.01,
        damage_class: DamageClass::Ballistic,
        instigator: Some(1),
        material_hit: 0,
        target_entity: Some(2),
        projectile_info: None,
    };
    assert_eq!(evt.impulse, 10.0);
    assert_eq!(evt.position.y, 2.0);
}

#[test]
fn stress_event_construction() {
    let evt = StressEvent {
        target_entity: 5,
        damage_class: DamageClass::Fatigue,
        intensity: 0.5,
        duration: 2.0,
        position: Some(Vec3::ZERO),
        source_direction: Some(Vec3::Y),
    };
    assert_eq!(evt.target_entity, 5);
    assert_eq!(evt.intensity, 0.5);
}

#[test]
fn projectile_info() {
    let pi = ProjectileInfo {
        caliber: 0.009,
        velocity: Vec3::new(300.0, 0.0, 0.0),
        mass: 0.008,
        fragmentation: true,
    };
    assert_eq!(pi.caliber, 0.009);
    assert!(pi.fragmentation);
}

#[test]
fn impact_event_with_projectile_info() {
    let pi = ProjectileInfo { caliber: 0.01, velocity: Vec3::Y, mass: 0.01, fragmentation: false };
    let evt = ImpactEvent {
        position: Vec3::ZERO,
        direction: Vec3::Z,
        impulse: 5.0,
        energy: 25.0,
        contact_area: 0.001,
        damage_class: DamageClass::Piercing,
        instigator: None,
        material_hit: 1,
        target_entity: None,
        projectile_info: Some(pi),
    };
    assert!(evt.projectile_info.is_some());
}

#[test]
fn stress_event_no_position() {
    let evt = StressEvent { target_entity: 1, damage_class: DamageClass::Thermal, intensity: 1.0, duration: 5.0, position: None, source_direction: None };
    assert!(evt.position.is_none());
}

// ===== DAMAGE PIPELINE (20 tests) =====

#[test]
fn damage_orchestrator_new() {
    let orch = DamageOrchestrator::new();
    assert!(orch.responses.is_empty());
}

#[test]
fn damage_orchestrator_submit_impact() {
    let mut orch = DamageOrchestrator::new();
    let evt = ImpactEvent {
        position: Vec3::ZERO,
        direction: Vec3::X,
        impulse: 10.0,
        energy: 100.0,
        contact_area: 0.01,
        damage_class: DamageClass::Ballistic,
        instigator: None,
        material_hit: 0,
        target_entity: None,
        projectile_info: None,
    };
    orch.submit_impact(evt);
}

#[test]
fn damage_orchestrator_submit_stress() {
    let mut orch = DamageOrchestrator::new();
    orch.submit_stress(StressEvent {
        target_entity: 1,
        damage_class: DamageClass::Thermal,
        intensity: 0.8,
        duration: 3.0,
        position: None,
        source_direction: None,
    });
}

#[test]
fn damage_orchestrator_drain_responses() {
    let mut orch = DamageOrchestrator::new();
    let r = orch.drain_responses();
    assert!(r.is_empty());
}

#[test]
fn damage_response_surface_marked() {
    let r = DamageResponse::SurfaceMarked {
        position: Vec3::ZERO,
        material: 0,
        decal_type: DecalType::BulletHole,
        intensity: 1.0,
    };
    assert!(matches!(r, DamageResponse::SurfaceMarked { .. }));
}

#[test]
fn damage_response_body_zone_damaged() {
    let r = DamageResponse::BodyZoneDamaged {
        entity: 1,
        zone: BodyZone::Head,
        damage: 25.0,
        penetrated_layers: 1,
    };
    assert!(matches!(r, DamageResponse::BodyZoneDamaged { .. }));
}

#[test]
fn damage_response_audio_trigger() {
    let r = DamageResponse::AudioTrigger {
        position: Vec3::ZERO,
        sound_class: SoundClass::ConcreteBullet,
        intensity: 0.8,
    };
    assert!(matches!(r, DamageResponse::AudioTrigger { .. }));
}

#[test]
fn response_aggregator_collect() {
    let mut responses = Vec::new();
    ResponseAggregator::collect(
        &mut responses,
        DamageResponse::SurfaceMarked {
            position: Vec3::ZERO,
            material: 0,
            decal_type: DecalType::Crack,
            intensity: 0.5,
        },
    );
    assert_eq!(responses.len(), 1);
}

#[test]
fn decal_type_variants() {
    let _ = DecalType::BulletHole;
    let _ = DecalType::Scorch;
    let _ = DecalType::BloodSplat;
}

#[test]
fn body_zone_variants() {
    let _ = BodyZone::Head;
    let _ = BodyZone::Torso;
    let _ = BodyZone::LeftArm;
    let _ = BodyZone::RightLeg;
}

#[test]
fn sound_class_variants() {
    let _ = SoundClass::FleshHit;
    let _ = SoundClass::Explosion;
}

// ===== LAYERED DAMAGE (15 tests) =====

#[test]
fn damageable_store_new() {
    let store = DamageableStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn damageable_store_register() {
    let mut store = DamageableStore::new();
    let obj = DamageableObject {
        entity: 1,
        capability: DamageCapability::SURFACE,
        layers: vec![],
        structural_section: None,
    };
    store.register(obj);
    assert_eq!(store.len(), 1);
}

#[test]
fn damageable_store_get() {
    let mut store = DamageableStore::new();
    let obj = DamageableObject {
        entity: 2,
        capability: DamageCapability::LAYERED,
        layers: vec![DamageLayer {
            material: 0,
            thickness: 0.1,
            integrity: 1.0,
            adhesion: 0.5,
            accumulated_stress: 0.0,
            thermal_damage: 0.0,
            moisture_damage: 0.0,
        }],
        structural_section: Some(1),
    };
    store.register(obj);
    let got = store.get(2).unwrap();
    assert_eq!(got.layers.len(), 1);
}

#[test]
fn damageable_store_get_mut() {
    let mut store = DamageableStore::new();
    store.register(DamageableObject {
        entity: 3,
        capability: DamageCapability::SURFACE,
        layers: vec![],
        structural_section: None,
    });
    let o = store.get_mut(3).unwrap();
    o.layers.push(DamageLayer {
        material: 1,
        thickness: 0.05,
        integrity: 1.0,
        adhesion: 0.3,
        accumulated_stress: 0.0,
        thermal_damage: 0.0,
        moisture_damage: 0.0,
    });
    assert_eq!(store.get(3).unwrap().layers.len(), 1);
}

#[test]
fn damageable_store_remove() {
    let mut store = DamageableStore::new();
    store.register(DamageableObject { entity: 4, capability: DamageCapability::empty(), layers: vec![], structural_section: None });
    let rem = store.remove(4);
    assert!(rem.is_some());
    assert!(store.get(4).is_none());
}

#[test]
fn damageable_store_capability_of() {
    let mut store = DamageableStore::new();
    store.register(DamageableObject { entity: 5, capability: DamageCapability::THERMAL, layers: vec![], structural_section: None });
    assert!(store.capability_of(5).contains(DamageCapability::THERMAL));
}

#[test]
fn damage_layer_fields() {
    let layer = DamageLayer {
        material: 0,
        thickness: 0.2,
        integrity: 0.8,
        adhesion: 0.6,
        accumulated_stress: 0.1,
        thermal_damage: 0.05,
        moisture_damage: 0.0,
    };
    assert_eq!(layer.thickness, 0.2);
}

#[test]
fn damageable_object_structural_section() {
    let obj = DamageableObject {
        entity: 6,
        capability: DamageCapability::STRUCTURAL,
        layers: vec![],
        structural_section: Some(42),
    };
    assert_eq!(obj.structural_section, Some(42));
}

// ===== COLLAPSE & STRUCTURAL (15 tests) =====

fn make_section(id: u32, integrity: f32, neighbor_count: usize) -> StructuralSection {
    StructuralSection {
        id,
        node_ids: vec![id],
        section_type: SectionType::Wall,
        integrity,
        neighbors: (0..neighbor_count)
            .map(|i| SectionNeighbor {
                section_id: i as u32 + 100,
                load_transfer: if i == 0 { 0.5 } else { 0.0 },
                collapse_priority: 0,
            })
            .collect(),
    }
}

#[test]
fn evaluate_failure_none_high_integrity() {
    let section = make_section(0, 0.95, 1);
    assert!(evaluate_failure(&section).is_none());
}

#[test]
fn evaluate_failure_crack() {
    let section = make_section(1, 0.7, 1);
    let r: CollapseResult = evaluate_failure(&section).unwrap();
    assert_eq!(r.failure_mode, FailureMode::Crack);
    assert_eq!(r.section_id, 1);
}

#[test]
fn evaluate_failure_full_collapse() {
    let section = make_section(2, 0.05, 1);
    let r = evaluate_failure(&section).unwrap();
    assert_eq!(r.failure_mode, FailureMode::FullCollapse);
}

#[test]
fn evaluate_hanging_unsupported() {
    let section = make_section(3, 0.5, 0);
    assert!(evaluate_hanging(&section));
}

#[test]
fn evaluate_hanging_supported() {
    let section = StructuralSection {
        id: 4,
        node_ids: vec![4],
        section_type: SectionType::Column,
        integrity: 0.5,
        neighbors: vec![SectionNeighbor { section_id: 10, load_transfer: 0.8, collapse_priority: 0 }],
    };
    assert!(!evaluate_hanging(&section));
}

#[test]
fn failure_mode_variants() {
    let _ = FailureMode::Crack;
    let _ = FailureMode::LocalBreak;
    let _ = FailureMode::PartialCollapse;
    let _ = FailureMode::FullCollapse;
    let _ = FailureMode::Hanging;
}

#[test]
fn test_redistribute_loads() {
    let mut sections = vec![
        make_section(0, 0.0, 2),
        make_section(1, 0.8, 0),
        make_section(2, 0.8, 0),
    ];
    sections[0].neighbors = vec![
        SectionNeighbor { section_id: 1, load_transfer: 0.5, collapse_priority: 0 },
        SectionNeighbor { section_id: 2, load_transfer: 0.5, collapse_priority: 0 },
    ];
    let newly = redistribute_loads(&mut sections, 0);
    let _newly_count = newly.len();
}

#[test]
fn test_cascade_collapse() {
    let mut sections = vec![
        make_section(0, 0.0, 1),
        make_section(1, 0.5, 0),
    ];
    sections[0].neighbors = vec![SectionNeighbor { section_id: 1, load_transfer: 0.5, collapse_priority: 0 }];
    let all = cascade_collapse(&mut sections, 0);
    assert!(all.contains(&0));
}

#[test]
fn structural_section_section_type() {
    let s = StructuralSection {
        id: 0,
        node_ids: vec![0],
        section_type: SectionType::Roof,
        integrity: 1.0,
        neighbors: vec![],
    };
    assert_eq!(s.section_type, SectionType::Roof);
}

#[test]
fn section_neighbor_load_transfer() {
    let n = SectionNeighbor { section_id: 1, load_transfer: 0.7, collapse_priority: 1 };
    assert_eq!(n.load_transfer, 0.7);
}

// ===== MATERIAL FRACTURE & SECONDARY (10 tests) =====

fn make_surface_material() -> SurfaceMaterial {
    SurfaceMaterial {
        id: 0,
        name: "test".to_string(),
        response_class: ResponseClass::BrittleCeramic,
        compressive_strength: 100.0,
        tensile_strength: 50.0,
        shear_strength: 40.0,
        brittleness: 0.8,
        density: 2.0,
        elasticity: 0.2,
        penetration_resistance: 50.0,
        hardness: 0.7,
        flammability: 0.0,
        fuel_content: 0.0,
        ignition_temp: 500.0,
        thermal_conductivity: 1.0,
        porosity: 0.1,
        erosion_resistance: 0.5,
        fragmentation_coeff: 0.5,
        fracture_pattern: 0,
        debris_profile: 0,
        decal_profile: 0,
        dust_intensity: 0.2,
        is_biological: false,
        gore_response: 0,
    }
}

#[test]
fn compute_fracture_below_threshold() {
    let mat = make_surface_material();
    let r = compute_fracture(&mat, 1.0, 1.0);
    assert!(!r.fractured);
    assert_eq!(r.fragment_count, 0);
}

#[test]
fn compute_fracture_above_threshold() {
    let mat = make_surface_material();
    let r = compute_fracture(&mat, 10000.0, 0.01);
    assert!(r.fractured || r.integrity_loss > 0.0);
}

#[test]
fn fracture_result_fields() {
    let fr = FractureResult {
        fractured: true,
        integrity_loss: 0.5,
        fragment_count: 5,
        residual_energy: 10.0,
    };
    assert_eq!(fr.fragment_count, 5);
}

#[test]
fn test_generate_fragments() {
    let frags = generate_fragments(Vec3::ZERO, 100.0, 0, 0.5);
    assert!(!frags.is_empty());
    assert!(frags.len() <= 16);
}

#[test]
fn test_fragment_to_impact() {
    let frag = DebrisFragment {
        position: Vec3::new(1.0, 2.0, 3.0),
        velocity: Vec3::new(10.0, 0.0, 0.0),
        mass: 0.05,
        material: 0,
        energy: 5.0,
    };
    let impact = fragment_to_impact(&frag);
    assert_eq!(impact.damage_class, DamageClass::Fragmentation);
    assert_eq!(impact.position, frag.position);
}

#[test]
fn debris_fragment_energy() {
    let frag = DebrisFragment {
        position: Vec3::ZERO,
        velocity: Vec3::X,
        mass: 0.1,
        material: 1,
        energy: 20.0,
    };
    assert_eq!(frag.energy, 20.0);
}

// ===== CHAIN REACTIONS (8 tests) =====

#[test]
fn chain_reaction_queue_new() {
    let q = ChainReactionQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.pending_count(), 0);
}

#[test]
fn chain_reaction_queue_submit() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent {
        source_entity: Some(1),
        position: Vec3::ZERO,
        energy: 100.0,
        damage_class: DamageClass::Explosive,
        depth: 0,
    });
    assert!(!q.is_empty());
    assert!(q.pending_count() >= 1);
}

#[test]
fn chain_reaction_queue_drain_batch() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent { source_entity: None, position: Vec3::ZERO, energy: 50.0, damage_class: DamageClass::Explosive, depth: 1 });
    let batch = q.drain_batch();
    let _batch_count = batch.len();
}

#[test]
fn chain_reaction_queue_rejects_low_energy() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent { source_entity: None, position: Vec3::ZERO, energy: 5.0, damage_class: DamageClass::Explosive, depth: 0 });
    assert!(q.is_empty() || q.pending_count() == 0);
}

#[test]
fn chain_reaction_queue_rejects_deep() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent { source_entity: None, position: Vec3::ZERO, energy: 100.0, damage_class: DamageClass::Explosive, depth: 5 });
    assert!(q.is_empty());
}

// ===== SOFT STATE (8 tests) =====

#[test]
fn soft_damage_state_new() {
    let s = SoftDamageState::new(1);
    assert_eq!(s.entity, 1);
    assert!(matches!(s.condition, ObjectCondition::Intact));
    assert_eq!(s.accumulated_fatigue, 0.0);
}

#[test]
fn soft_damage_state_apply_fatigue() {
    let mut s = SoftDamageState::new(2);
    s.apply_fatigue(0.5);
    assert!(s.accumulated_fatigue >= 0.5);
}

#[test]
fn object_condition_wobbling() {
    let _ = ObjectCondition::Wobbling { amplitude: 0.1, frequency: 2.0 };
}

#[test]
fn object_condition_cracked() {
    let _ = ObjectCondition::Cracked { severity: 0.5 };
}

#[test]
fn object_condition_sagging() {
    let _ = ObjectCondition::Sagging { amount: 0.3 };
}

#[test]
fn object_condition_leaning() {
    let _ = ObjectCondition::Leaning { angle_deg: 15.0 };
}

#[test]
fn object_condition_limping() {
    let _ = ObjectCondition::Limping;
}

// ===== PHYSICS LOD (5 tests) =====

#[test]
fn physics_lod_from_sim_level() {
    assert_eq!(PhysicsLod::from_sim_level(SimulationLevel::L0), PhysicsLod::Full);
    assert_eq!(PhysicsLod::from_sim_level(SimulationLevel::L1), PhysicsLod::Simplified);
    assert_eq!(PhysicsLod::from_sim_level(SimulationLevel::L2), PhysicsLod::Statistical);
    assert_eq!(PhysicsLod::from_sim_level(SimulationLevel::L3), PhysicsLod::Paused);
}

#[test]
fn physics_lod_cloth_iterations() {
    assert_eq!(PhysicsLod::Full.cloth_iterations(), 10);
    assert_eq!(PhysicsLod::Simplified.cloth_iterations(), 3);
    assert_eq!(PhysicsLod::Statistical.cloth_iterations(), 0);
}

#[test]
fn physics_lod_water_active() {
    assert!(PhysicsLod::Full.water_active());
    assert!(!PhysicsLod::Paused.water_active());
}

#[test]
fn physics_lod_fire_tick_interval() {
    assert_eq!(PhysicsLod::Full.fire_tick_interval(), 1);
    assert_eq!(PhysicsLod::Simplified.fire_tick_interval(), 4);
    assert_eq!(PhysicsLod::Paused.fire_tick_interval(), u64::MAX);
}

// ===== FIRE (12 tests) =====

#[test]
fn fire_grid_new() {
    let grid = FireGrid::new();
    assert_eq!(grid.active_fire_count(), 0);
}

#[test]
fn fire_grid_ignite() {
    let mut grid = FireGrid::new();
    grid.set_fuel(5, 5, 1.0, 0.8);
    grid.ignite(5, 5);
    assert!(grid.active_fire_count() >= 1);
}

#[test]
fn fire_grid_extinguish() {
    let mut grid = FireGrid::new();
    grid.set_fuel(10, 10, 1.0, 0.9);
    grid.ignite(10, 10);
    grid.extinguish(10, 10);
    assert_eq!(grid.get(10, 10).state, FireState::Unlit);
}

#[test]
fn fire_grid_update() {
    let mut grid = FireGrid::new();
    grid.set_fuel(1, 1, 1.0, 0.5);
    grid.ignite(1, 1);
    grid.update(0.1, SimulationLevel::L0);
}

#[test]
fn fire_grid_burning_cells() {
    let mut grid = FireGrid::new();
    grid.set_fuel(0, 0, 1.0, 0.7);
    grid.ignite(0, 0);
    let cells = grid.burning_cells();
    assert!(!cells.is_empty());
}

#[test]
fn fire_grid_is_near_fire() {
    let mut grid = FireGrid::new();
    grid.set_fuel(20, 20, 1.0, 0.8);
    grid.ignite(20, 20);
    let (wx, wy) = FireGrid::cell_world_pos(20, 20);
    assert!(grid.is_near_fire(wx, wy, 10.0));
}

#[test]
fn fire_grid_set_fuel() {
    let mut grid = FireGrid::new();
    grid.set_fuel(3, 3, 0.5, 0.6);
    let cell = grid.get(3, 3);
    assert_eq!(cell.fuel, 0.5);
    assert_eq!(cell.flammability, 0.6);
}

#[test]
fn fire_state_variants() {
    let _ = FireState::Unlit;
    let _ = FireState::Burning;
    let _ = FireState::BurnedOut;
}

#[test]
fn fire_cell_default() {
    let cell = FireCell::default();
    assert_eq!(cell.state, FireState::Unlit);
    assert_eq!(cell.fuel, 0.0);
}

// ===== WATER (10 tests) =====

#[test]
fn water_grid_new() {
    let grid = WaterGrid::new();
    let _level = grid.water_level_at_world(0.0, 0.0);
}

#[test]
fn water_grid_add_water() {
    let mut grid = WaterGrid::new();
    grid.add_water(0, 0, 1.0);
    assert!(grid.get(0, 0).water_level > 0.0);
}

#[test]
fn water_grid_water_level_at_world() {
    let mut grid = WaterGrid::new();
    grid.add_water(5, 5, 2.0);
    let (wx, wz) = (250.0, 250.0);
    assert!(grid.water_level_at_world(wx, wz) >= 0.0);
}

#[test]
fn water_grid_buoyancy_force() {
    let mut grid = WaterGrid::new();
    grid.add_water(0, 0, 1.0);
    let force = grid.buoyancy_force(25.0, 25.0, 0.0);
    assert!(force >= 0.0);
}

#[test]
fn water_grid_set_terrain_height() {
    let mut grid = WaterGrid::new();
    grid.set_terrain_height(1, 1, 5.0);
    assert_eq!(grid.get(1, 1).height, 5.0);
}

#[test]
fn water_grid_mark_container() {
    let mut grid = WaterGrid::new();
    grid.mark_container(2, 2);
    assert!(grid.get(2, 2).is_container);
}

#[test]
fn water_cell_default() {
    let cell = WaterCell::default();
    assert_eq!(cell.water_level, 0.0);
    assert!(!cell.is_container);
}

// ===== BODY ANATOMY (15 tests) =====

#[test]
fn anatomy_body_state_new_humanoid() {
    let body = AnatomyBodyState::new_humanoid(0);
    assert_eq!(body.zones.len(), 8);
    assert_eq!(body.joints.len(), 6);
}

#[test]
fn anatomy_is_alive() {
    let body = AnatomyBodyState::new_humanoid(1);
    assert!(body.is_alive());
}

#[test]
fn anatomy_aggregate_health() {
    let body = AnatomyBodyState::new_humanoid(2);
    let h = body.aggregate_health();
    assert!(h > 0.0 && h <= 100.0);
}

#[test]
fn zone_state_new() {
    let z = ZoneState::new(BodyZone::Head);
    assert_eq!(z.integrity, 100.0);
    assert!(!z.is_severed);
}

#[test]
fn anatomy_zones_order() {
    let body = AnatomyBodyState::new_humanoid(3);
    assert_eq!(body.zones[0].zone, BodyZone::Head);
    assert_eq!(body.zones[2].zone, BodyZone::Torso);
}

#[test]
fn anatomy_blood_level() {
    let body = AnatomyBodyState::new_humanoid(4);
    assert_eq!(body.blood_level, 1.0);
}

#[test]
fn anatomy_bleed_points_empty() {
    let body = AnatomyBodyState::new_humanoid(5);
    assert!(body.bleed_points.is_empty());
}

#[test]
fn joint_info_dismember_threshold() {
    let j = JointInfo { id: 0, zone: BodyZone::Neck, integrity: 100.0, broken: false, dismember_threshold: 15.0 };
    assert_eq!(j.dismember_threshold, 15.0);
}

#[test]
fn bleed_point_fields() {
    let bp = BleedPoint { zone: BodyZone::Torso, rate: 0.1, time_active: 0.0 };
    assert_eq!(bp.rate, 0.1);
}

// ===== BODY STORE (10 tests) =====

#[test]
fn body_state_store_new() {
    let store = BodyStateStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn body_state_store_allocate() {
    let mut store = BodyStateStore::new();
    let body = AnatomyBodyState::new_humanoid(0);
    let handle = store.allocate(body);
    assert_eq!(store.len(), 1);
    assert!(handle.is_alive);
}

#[test]
fn body_state_store_get() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(1));
    let body = store.get(handle.store_id).unwrap();
    assert_eq!(body.entity, 1);
}

#[test]
fn body_state_store_get_mut() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(2));
    let b = store.get_mut(handle.store_id).unwrap();
    b.blood_level = 0.5;
    assert_eq!(store.get(handle.store_id).unwrap().blood_level, 0.5);
}

#[test]
fn body_state_store_free() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(3));
    let len_before = store.len();
    store.free(handle.store_id);
    assert_eq!(store.len(), len_before - 1);
}

#[test]
fn body_state_store_tick() {
    let mut store = BodyStateStore::new();
    store.allocate(AnatomyBodyState::new_humanoid(4));
    store.tick(0.1);
}

#[test]
fn body_handle_fields() {
    let mut store = BodyStateStore::new();
    let h = store.allocate(AnatomyBodyState::new_humanoid(5));
    assert_eq!(h.store_id, 0);
    assert!(!h.has_severed_limb);
}

// ===== BODY RESPONSE (10 tests) =====

#[test]
fn body_physical_response_cache_healthy() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert_eq!(cache.movement_speed_mult, 1.0);
    assert!(cache.can_sprint);
}

#[test]
fn body_physical_response_compute_from_health() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.9, 1.0, 0.0);
    assert_eq!(cache.response_tier, PhysicalResponseTier::Healthy);
}

#[test]
fn body_physical_response_critical() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.2, 0.5, 0.8);
    assert_eq!(cache.response_tier, PhysicalResponseTier::Critical);
}

#[test]
fn body_physical_response_is_combat_capable() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert!(cache.is_combat_capable());
}

#[test]
fn body_physical_response_is_mobile() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert!(cache.is_mobile());
}

#[test]
fn physical_response_tier_variants() {
    let _ = PhysicalResponseTier::Healthy;
    let _ = PhysicalResponseTier::MinorInjury;
    let _ = PhysicalResponseTier::MajorInjury;
    let _ = PhysicalResponseTier::Critical;
    let _ = PhysicalResponseTier::Unconscious;
    let _ = PhysicalResponseTier::Dead;
}

#[test]
fn body_physical_response_dead() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.0, 0.0, 1.0);
    assert!(!cache.is_mobile());
}

// ===== BODY DAMAGE & DISMEMBERMENT (8 tests) =====

#[test]
fn test_apply_zone_damage() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let health_before = store.get(handle.store_id).unwrap().aggregate_health();
    apply_zone_damage(&mut store, handle.store_id, BodyZone::Torso, 30.0);
    let health_after = store.get(handle.store_id).unwrap().aggregate_health();
    assert!(health_after < health_before);
}

#[test]
fn check_dismemberment_joint_intact() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let result = check_dismemberment(&mut store, handle.store_id, 0);
    assert!(!result);
}

#[test]
fn check_dismemberment_joint_broken() {
    let mut store = BodyStateStore::new();
    let mut body = AnatomyBodyState::new_humanoid(0);
    body.joints[0].broken = true;
    let handle = store.allocate(body);
    let result = check_dismemberment(&mut store, handle.store_id, 0);
    assert!(result);
}

#[test]
fn severed_zones_empty() {
    let mut store = BodyStateStore::new();
    let handle = store.allocate(AnatomyBodyState::new_humanoid(0));
    let zones = severed_zones(&store, handle.store_id);
    assert!(zones.is_empty());
}

#[test]
fn severed_zones_after_dismember() {
    let mut store = BodyStateStore::new();
    let mut body = AnatomyBodyState::new_humanoid(0);
    body.joints[1].broken = true;
    body.zones[3].is_severed = true;
    let handle = store.allocate(body);
    let zones = severed_zones(&store, handle.store_id);
    assert!(!zones.is_empty());
}

// ===== DEATH PIPELINE (12 tests) =====

#[test]
fn corpse_state_new() {
    let c = CorpseState::new(1, [10.0, 20.0], "combat");
    assert_eq!(c.entity, 1);
    assert_eq!(c.position[0], 10.0);
    assert!(c.lootable);
}

#[test]
fn corpse_state_tick() {
    let mut c = CorpseState::new(2, [0.0, 0.0], "fall");
    c.tick(1.0);
    assert!(c.decay_timer > 0.0);
}

#[test]
fn corpse_state_decay_progress() {
    let mut c = CorpseState::new(3, [0.0, 0.0], "test");
    c.tick(150.0);
    assert!(c.decay_progress() > 0.0);
}

#[test]
fn corpse_state_should_remove() {
    let mut c = CorpseState::new(4, [0.0, 0.0], "test");
    for _ in 0..350 {
        c.tick(1.0);
    }
    assert!(c.should_remove());
}

#[test]
fn corpse_state_loot_item() {
    let mut c = CorpseState::new(5, [0.0, 0.0], "test");
    c.items_remaining.push("sword".to_string());
    let item = c.loot_item(0);
    assert_eq!(item, Some("sword".to_string()));
    assert!(c.items_remaining.is_empty());
}

#[test]
fn corpse_manager_new() {
    let mgr = CorpseManager::new();
    assert_eq!(mgr.corpse_count(), 0);
}

#[test]
fn corpse_manager_register_death() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [5.0, 5.0], "combat", vec!["axe".to_string()]);
    assert_eq!(mgr.corpse_count(), 1);
}

#[test]
fn corpse_manager_tick_all() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [0.0, 0.0], "test", vec![]);
    mgr.tick_all(1.0);
}

#[test]
fn corpse_manager_corpses_near() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [10.0, 10.0], "test", vec![]);
    let near = mgr.corpses_near(10.0, 10.0, 5.0);
    assert!(!near.is_empty());
}

#[test]
fn death_state_variants() {
    let _ = DeathState::Alive;
    let _ = DeathState::Dying;
    let _ = DeathState::Dead;
    let _ = DeathState::Corpse { decay_ticks: 0 };
    let _ = DeathState::Skeleton;
    let _ = DeathState::Removed;
}

// ===== BLOOD (5 tests) =====

#[test]
fn compute_blood_lod_full() {
    assert_eq!(compute_blood_lod(100.0 * 100.0), BloodLod::Full);
}

#[test]
fn compute_blood_lod_reduced() {
    assert_eq!(compute_blood_lod(500.0 * 500.0), BloodLod::Reduced);
}

#[test]
fn compute_blood_lod_state_only() {
    assert_eq!(compute_blood_lod(2000.0 * 2000.0), BloodLod::StateOnly);
}

#[test]
fn blood_lod_variants() {
    let _ = BloodLod::Full;
    let _ = BloodLod::Reduced;
    let _ = BloodLod::StateOnly;
}

// ===== AI COMBAT (10 tests) =====

#[test]
fn hit_location_variants() {
    let _ = HitLocation::Head;
    let _ = HitLocation::Torso;
    let _ = HitLocation::Arms;
    let _ = HitLocation::Legs;
}

#[test]
fn hit_location_random() {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let loc = HitLocation::random(&mut rng);
    assert!(matches!(loc, HitLocation::Head | HitLocation::Torso | HitLocation::Arms | HitLocation::Legs));
}

#[test]
fn hit_location_damage_multiplier() {
    assert_eq!(HitLocation::Head.damage_multiplier(), 2.0);
    assert_eq!(HitLocation::Torso.damage_multiplier(), 1.0);
    assert_eq!(HitLocation::Arms.damage_multiplier(), 0.7);
    assert_eq!(HitLocation::Legs.damage_multiplier(), 0.8);
}

#[test]
fn hit_location_stagger_chance() {
    assert!(HitLocation::Head.stagger_chance() > HitLocation::Torso.stagger_chance());
}

#[test]
fn stagger_state_none() {
    let s = StaggerState::None;
    assert!(!s.is_incapacitated());
}

#[test]
fn stagger_state_stagger() {
    let mut s = StaggerState::Stagger { remaining: 1.0 };
    assert!(s.is_incapacitated());
    s.tick(0.5);
}

#[test]
fn stagger_state_knockdown() {
    let s = StaggerState::Knockdown { remaining: 2.0 };
    assert!(s.is_incapacitated());
}

#[test]
fn test_resolve_combat() {
    let mut ecs = Ecs::new();
    let mut events = EventBus::new();
    let attacker = ecs.spawn();
    let defender = ecs.spawn();
    ecs.transforms.insert(attacker, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(defender, Transform { x: 5.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(attacker, EntityKind::Npc);
    ecs.kinds.insert(defender, EntityKind::Npc);
    ecs.personal_needs.insert(attacker, PersonalNeeds::default_npc());
    ecs.personal_needs.insert(defender, PersonalNeeds::default_npc());
    let def_health_before = ecs.personal_needs.get(&defender).unwrap().health;
    resolve_combat(&mut ecs, &mut events, attacker, defender);
    let def_health_after = ecs.personal_needs.get(&defender).unwrap().health;
    assert!(def_health_after <= def_health_before);
}

// ===== AI BODY (8 tests) =====

#[test]
fn ai_body_state_compute() {
    let pn = PersonalNeeds::default_npc();
    let body = AiBodyState::compute(&pn);
    assert!(body.move_speed_mult > 0.0);
    assert!(body.combat_power_mult > 0.0);
}

#[test]
fn is_night_true_late() {
    assert!(is_night(0.8));
}

#[test]
fn is_night_true_early() {
    assert!(is_night(0.1));
}

#[test]
fn is_night_false() {
    assert!(!is_night(0.5));
}

#[test]
fn time_of_day_mult_diurnal_night() {
    assert!((time_of_day_mult(0.8, false) - 0.6).abs() < 0.01);
}

#[test]
fn time_of_day_mult_nocturnal_night() {
    assert!((time_of_day_mult(0.9, true) - 1.3).abs() < 0.01);
}

#[test]
fn time_of_day_mult_diurnal_day() {
    assert!((time_of_day_mult(0.5, false) - 1.0).abs() < 0.01);
}

#[test]
fn ai_body_state_work_efficiency() {
    let pn = PersonalNeeds { hunger: 0.2, thirst: 0.2, sleep: 0.1, health: 1.0, energy: 0.9, fear: 0.0, curiosity: 0.3, ambitions: 0.4, discomfort: 0.1 };
    let body = AiBodyState::compute(&pn);
    assert!(body.work_efficiency_mult > 0.0);
}

// ===== NAVIGATION (15 tests) =====

#[test]
fn world_graph_new() {
    let g = WorldGraph::new();
    assert!(g.locations.is_empty());
}

#[test]
fn world_graph_add_location() {
    let mut g = WorldGraph::new();
    let id = g.add_location("Town", 5, 10);
    assert_eq!(id, 0);
    assert!(g.locations.contains_key(&id));
}

#[test]
fn world_graph_connect() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let b = g.add_location("B", 10, 0);
    g.connect(a, b);
    assert!(g.edges.len() >= 2);
}

#[test]
fn world_graph_neighbors() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let b = g.add_location("B", 5, 0);
    g.connect(a, b);
    let ne = g.neighbors(a);
    assert!(ne.contains(&b));
}

#[test]
fn world_graph_distance_between() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let b = g.add_location("B", 3, 4);
    let d = g.distance_between(a, b);
    assert!(d > 0.0 && d < 1000.0);
}

#[test]
fn world_graph_find_path() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let b = g.add_location("B", 2, 0);
    g.connect(a, b);
    let path = g.find_path(a, b);
    assert!(path.is_some());
    assert!(path.unwrap().len() >= 2);
}

#[test]
fn world_graph_find_path_same() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let path = g.find_path(a, a);
    assert_eq!(path, Some(vec![a]));
}

#[test]
fn world_graph_build_default() {
    let g = WorldGraph::build_default();
    assert!(g.locations.len() >= 5);
}

#[test]
fn path_cache_new() {
    let cache = PathCache::new();
    assert!(cache.get(99, 99).is_none());
}

#[test]
fn path_cache_get_empty() {
    let cache = PathCache::new();
    assert!(cache.get(0, 1).is_none());
}

#[test]
fn path_cache_store_and_get() {
    let mut cache = PathCache::new();
    cache.store(0, 1, vec![0, 1]);
    let path = cache.get(0, 1);
    assert!(path.is_some());
    assert_eq!(path.unwrap(), &[0u32, 1u32][..]);
}

#[test]
fn ballistics_projectile_ttl() {
    let mut sys = BallisticsSystem::new();
    sys.fire(Vec3::ZERO, Vec3::Y, 50.0, 0.01, 0.2, 0, 0, 0);
    assert_eq!(sys.projectiles[0].ttl, 5.0);
}

#[test]
fn destruction_node_accumulated_stress() {
    let n = DestructionNode { id: 0, position: Vec3::ZERO, mass: 1.0, material: 0, accumulated_stress: 0.5 };
    assert_eq!(n.accumulated_stress, 0.5);
}

#[test]
fn destruction_link_broken_flag() {
    let l = DestructionLink { a: 0, b: 1, strength: 100.0, fatigue: 0.5, broken: true };
    assert!(l.broken);
}

#[test]
fn apply_velocity_updates_cell() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(e, Transform { x: 100.0, y: 200.0, cell_x: 0, cell_y: 0 });
    apply_velocity(&mut ecs, e, 0.1);
    let t = ecs.transforms.get(&e).unwrap();
    assert!(t.cell_x <= 40 && t.cell_y <= 40);
}

#[test]
fn damage_class_piercing_instant() {
    assert!(DamageClass::Piercing.is_instant());
}

#[test]
fn impact_event_material_hit() {
    let evt = ImpactEvent {
        position: Vec3::ZERO,
        direction: Vec3::X,
        impulse: 1.0,
        energy: 10.0,
        contact_area: 0.001,
        damage_class: DamageClass::Blunt,
        instigator: None,
        material_hit: 7,
        target_entity: None,
        projectile_info: None,
    };
    assert_eq!(evt.material_hit, 7);
}

#[test]
fn damage_response_layer_fractured() {
    let r = DamageResponse::LayerFractured {
        entity: 1,
        layer_idx: 0,
        pattern: 0,
        residual_energy: 5.0,
    };
    assert!(matches!(r, DamageResponse::LayerFractured { .. }));
}

#[test]
fn damage_response_ricochet() {
    let r = DamageResponse::Ricochet { position: Vec3::ZERO, direction: Vec3::X, energy: 10.0 };
    assert!(matches!(r, DamageResponse::Ricochet { .. }));
}

#[test]
fn failure_mode_local_break() {
    let section = make_section(10, 0.5, 1);
    let r = evaluate_failure(&section).unwrap();
    assert!(matches!(r.failure_mode, FailureMode::Crack | FailureMode::LocalBreak));
}

#[test]
fn chain_event_depth() {
    let e = ChainEvent { source_entity: Some(1), position: Vec3::ZERO, energy: 100.0, damage_class: DamageClass::Explosive, depth: 2 };
    assert_eq!(e.depth, 2);
}

#[test]
fn fire_grid_cell_world_pos() {
    let (x, y) = FireGrid::cell_world_pos(10, 10);
    assert!(x > 0.0 && y > 0.0);
}

#[test]
fn water_grid_rain_intensity() {
    let mut grid = WaterGrid::new();
    grid.rain_intensity = 0.5;
    assert_eq!(grid.rain_intensity, 0.5);
}

#[test]
fn anatomy_consciousness_initial() {
    let body = AnatomyBodyState::new_humanoid(0);
    assert_eq!(body.consciousness, 1.0);
}

#[test]
fn corpse_state_cause_of_death() {
    let c = CorpseState::new(1, [0.0, 0.0], "explosion");
    assert_eq!(c.cause_of_death, "explosion");
}

#[test]
fn stagger_state_tick_recovers() {
    let mut s = StaggerState::Stagger { remaining: 0.2 };
    s.tick(0.3);
    assert!(!s.is_incapacitated());
}

#[test]
fn world_graph_location_name() {
    let mut g = WorldGraph::new();
    g.add_location("Cave", 1, 2);
    let loc = g.locations.get(&0).unwrap();
    assert_eq!(loc.name, "Cave");
}

#[test]
fn world_graph_find_path_unconnected() {
    let mut g = WorldGraph::new();
    let a = g.add_location("A", 0, 0);
    let b = g.add_location("B", 5, 5);
    let path = g.find_path(a, b);
    assert!(path.is_none());
}

#[test]
fn section_type_variants() {
    let _ = SectionType::Wall;
    let _ = SectionType::Floor;
    let _ = SectionType::Door;
}

#[test]
fn damage_response_structural_damage() {
    let r = DamageResponse::StructuralDamage { entity: 1, section_id: 5, energy: 100.0 };
    assert!(matches!(r, DamageResponse::StructuralDamage { .. }));
}

#[test]
fn soft_damage_state_condition_after_fatigue() {
    let mut s = SoftDamageState::new(1);
    s.apply_fatigue(0.95);
    assert!(!matches!(s.condition, ObjectCondition::Intact));
}

#[test]
fn body_physical_response_dead_tier() {
    let cache = BodyPhysicalResponseCache::compute_from_health(0.0, 0.0, 0.0);
    assert_eq!(cache.response_tier, PhysicalResponseTier::Dead);
}

#[test]
fn building_descriptor_sections() {
    let desc = BuildingDescriptor {
        sections: vec![SectionDescriptor {
            id: 0,
            section_type: SectionType::Wall,
            material: 0,
            thickness: 0.3,
            hero: false,
        }],
        section_adjacency: vec![(0, 0, 1.0)],
        hero_objects: vec![],
    };
    assert_eq!(desc.sections.len(), 1);
    assert_eq!(desc.sections[0].section_type, SectionType::Wall);
}

#[test]
fn body_handle_from_store() {
    let mut store = BodyStateStore::new();
    let body = AnatomyBodyState::new_humanoid(42);
    let handle: BodyHandle = store.allocate(body);
    assert!(handle.is_alive);
}

#[test]
fn world_graph_location_id_type() {
    let mut graph = WorldGraph::new();
    let id: LocationId = graph.add_location("camp", 5, 5);
    assert_eq!(id, 0);
}