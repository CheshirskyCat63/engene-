use super::*;

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
    // check_overlap() uses radius as per-entity radius, so overlap threshold is 2*radius.
    // Boundary-near overlap for radius=1.0 should be true just under distance 2.0.
    assert!(check_overlap(0.0, 0.0, 1.9, 0.0, 1.0));
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
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    ecs.transforms.insert(
        a,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        b,
        Transform {
            x: 1.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    let pi = ProjectileInfo {
        caliber: 0.01,
        velocity: Vec3::Y,
        mass: 0.01,
        fragmentation: false,
    };
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
    let evt = StressEvent {
        target_entity: 1,
        damage_class: DamageClass::Thermal,
        intensity: 1.0,
        duration: 5.0,
        position: None,
        source_direction: None,
    };
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
    store.register(DamageableObject {
        entity: 4,
        capability: DamageCapability::empty(),
        layers: vec![],
        structural_section: None,
    });
    let rem = store.remove(4);
    assert!(rem.is_some());
    assert!(store.get(4).is_none());
}

#[test]
fn damageable_store_capability_of() {
    let mut store = DamageableStore::new();
    store.register(DamageableObject {
        entity: 5,
        capability: DamageCapability::THERMAL,
        layers: vec![],
        structural_section: None,
    });
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
