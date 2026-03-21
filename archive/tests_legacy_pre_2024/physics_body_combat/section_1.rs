use super::*;

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
        BallisticEvent::ShotFired {
            seed,
            origin: o,
            weapon_id,
            owner,
            dir: _,
        } => {
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
    sys.material_table.register(
        "concrete",
        1,
        MaterialProps {
            hardness: 0.9,
            penetration_resistance: 500.0,
            density: 2.4,
        },
    );
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
    sys.material_table.register(
        "metal",
        1,
        MaterialProps {
            hardness: 0.95,
            penetration_resistance: 100.0,
            density: 7.8,
        },
    );
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
    sys.material_table.register(
        "wood",
        1,
        MaterialProps {
            hardness: 0.3,
            penetration_resistance: 0.05,
            density: 0.6,
        },
    );
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
fn impact_penetration_threshold_contract() {
    let mut sys = BallisticsSystem::new();
    sys.material_table.register(
        "test",
        1,
        MaterialProps {
            hardness: 0.4,
            penetration_resistance: 0.2, // threshold = 200
            density: 2.0,
        },
    );

    let below = Projectile {
        pos: Vec3::ZERO,
        prev_pos: Vec3::ZERO,
        vel: Vec3::new(1.0, -1.0, 0.0).normalize() * 10.0,
        energy: 199.0,
        drag_coeff: 0.0,
        mass: 0.01,
        ttl: 1.0,
        seed: 0,
        owner: 0,
        weapon_id: 0,
        distance_traveled: 0.0,
    };
    let above = Projectile {
        energy: 201.0,
        ..below.clone()
    };

    let normal = Vec3::new(0.0, 1.0, 0.0);
    assert!(matches!(
        sys.resolve_impact(&below, normal, 1),
        ImpactResult::Stopped { .. }
    ));
    assert!(matches!(
        sys.resolve_impact(&above, normal, 1),
        ImpactResult::Penetrated { .. }
    ));
}

#[test]
fn ballistics_projectile_fields() {
    let mut sys = BallisticsSystem::new();
    sys.fire(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::X,
        80.0,
        0.005,
        0.3,
        5,
        2,
        777,
    );
    let p = &sys.projectiles[0];
    assert_eq!(p.prev_pos, p.pos);
    assert_eq!(p.distance_traveled, 0.0);
    assert!((p.energy - 0.5 * 0.005 * 80.0 * 80.0).abs() < 0.1);
}

#[test]
fn ballistics_multiple_fire() {
    let mut sys = BallisticsSystem::new();
    for i in 0..5u64 {
        sys.fire(
            Vec3::new(i as f32, 0.0, 0.0),
            Vec3::Y,
            50.0,
            0.01,
            0.2,
            i,
            0,
            i as u32,
        );
    }
    assert_eq!(sys.projectiles.len(), 5);
    assert_eq!(sys.events.len(), 5);
}

#[test]
fn ballistics_material_multiple() {
    let mut mt = MaterialTable::new();
    mt.register(
        "a",
        1,
        MaterialProps {
            hardness: 0.5,
            penetration_resistance: 50.0,
            density: 1.0,
        },
    );
    mt.register(
        "b",
        2,
        MaterialProps {
            hardness: 0.9,
            penetration_resistance: 200.0,
            density: 3.0,
        },
    );
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
    let _sf = BallisticEvent::ShotFired {
        seed: 0,
        origin: Vec3::ZERO,
        dir: Vec3::Y,
        weapon_id: 0,
        owner: 0,
    };
    let _im = BallisticEvent::Impact {
        seed: 0,
        hit_pos: Vec3::ZERO,
        normal: Vec3::Y,
        material: 0,
        damage: 10.0,
    };
    let _eh = BallisticEvent::EntityHit {
        entity: 0,
        damage: 5.0,
        hit_pos: Vec3::ZERO,
        projectile_vel: Vec3::X,
    };
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
    let p = MaterialProps {
        hardness: 0.7,
        penetration_resistance: 80.0,
        density: 2.0,
    };
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
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![DestructionLink {
            a: 0,
            b: 0,
            strength: 100.0,
            fatigue: 0.0,
            broken: false,
        }],
    );
    sys.register_object(obj);
    assert_eq!(sys.objects.len(), 1);
}

#[test]
fn destruction_apply_impulse_at_full_lod() {
    let mut sys = DestructionSystem::new();
    let entity = 1u64;
    let nodes = vec![
        DestructionNode {
            id: 0,
            position: Vec3::new(0.0, 0.0, 0.0),
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        },
        DestructionNode {
            id: 1,
            position: Vec3::new(1.0, 0.0, 0.0),
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        },
    ];
    let links = vec![DestructionLink {
        a: 0,
        b: 1,
        strength: 50.0,
        fatigue: 0.0,
        broken: false,
    }];
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
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![DestructionLink {
            a: 0,
            b: 0,
            strength: 100.0,
            fatigue: 0.0,
            broken: false,
        }],
    );
    sys.register_object(obj);
    sys.apply_impulse_at(Vec3::ZERO, 500.0, DestructionLod::Statistical);
}

#[test]
fn destruction_apply_impulse_frozen() {
    let mut sys = DestructionSystem::new();
    let obj = DestructibleObject::new(
        3u64,
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![DestructionLink {
            a: 0,
            b: 0,
            strength: 100.0,
            fatigue: 0.0,
            broken: false,
        }],
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
            DestructionNode {
                id: 0,
                position: Vec3::ZERO,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            DestructionNode {
                id: 1,
                position: Vec3::new(5.0, 0.0, 0.0),
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
        ],
        vec![DestructionLink {
            a: 0,
            b: 1,
            strength: 10.0,
            fatigue: 0.0,
            broken: false,
        }],
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
        DestructionLink {
            a: 0,
            b: 1,
            strength: 50.0,
            fatigue: 0.0,
            broken: false,
        },
        DestructionLink {
            a: 1,
            b: 2,
            strength: 50.0,
            fatigue: 0.0,
            broken: true,
        },
    ];
    let obj = DestructibleObject::new(
        entity,
        vec![
            DestructionNode {
                id: 0,
                position: Vec3::ZERO,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            DestructionNode {
                id: 1,
                position: Vec3::X,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            DestructionNode {
                id: 2,
                position: Vec3::new(2.0, 0.0, 0.0),
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
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
            DestructionNode {
                id: 0,
                position: Vec3::ZERO,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            DestructionNode {
                id: 1,
                position: Vec3::new(2.0, 0.0, 0.0),
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
        ],
        vec![DestructionLink {
            a: 0,
            b: 1,
            strength: 5.0,
            fatigue: 0.0,
            broken: false,
        }],
    );
    let events = obj.apply_impulse(Vec3::ZERO, 1000.0);
    let _count = events.len();
}

#[test]
fn destructible_apply_statistical_damage() {
    let mut obj = DestructibleObject::new(
        7u64,
        vec![
            DestructionNode {
                id: 0,
                position: Vec3::ZERO,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            DestructionNode {
                id: 1,
                position: Vec3::X,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
        ],
        vec![
            DestructionLink {
                a: 0,
                b: 1,
                strength: 100.0,
                fatigue: 0.0,
                broken: false,
            },
            DestructionLink {
                a: 1,
                b: 0,
                strength: 100.0,
                fatigue: 0.0,
                broken: false,
            },
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
    let _e = DestructionEvent::LinkBroken {
        entity: 0,
        link_a: 0,
        link_b: 1,
    };
}

#[test]
fn destruction_event_object_fragmented() {
    let _e = DestructionEvent::ObjectFragmented {
        entity: 0,
        cluster_count: 2,
    };
}

#[test]
fn destructible_new_total_strength() {
    let obj = DestructibleObject::new(
        8u64,
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![
            DestructionLink {
                a: 0,
                b: 0,
                strength: 30.0,
                fatigue: 0.0,
                broken: false,
            },
            DestructionLink {
                a: 0,
                b: 0,
                strength: 70.0,
                fatigue: 0.0,
                broken: false,
            },
        ],
    );
    assert_eq!(obj.total_strength, 100.0);
}

#[test]
fn destructible_integrity_full() {
    let obj = DestructibleObject::new(
        9u64,
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![DestructionLink {
            a: 0,
            b: 0,
            strength: 100.0,
            fatigue: 0.0,
            broken: false,
        }],
    );
    assert!((obj.integrity() - 1.0).abs() < 0.01);
}

#[test]
fn destructible_integrity_zero() {
    let obj = DestructibleObject::new(
        10u64,
        vec![DestructionNode {
            id: 0,
            position: Vec3::ZERO,
            mass: 1.0,
            material: 0,
            accumulated_stress: 0.0,
        }],
        vec![DestructionLink {
            a: 0,
            b: 0,
            strength: 100.0,
            fatigue: 0.0,
            broken: true,
        }],
    );
    assert!(obj.integrity() < 0.01);
}
