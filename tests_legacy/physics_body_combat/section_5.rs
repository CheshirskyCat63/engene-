use super::*;

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
    assert!(matches!(
        loc,
        HitLocation::Head | HitLocation::Torso | HitLocation::Arms | HitLocation::Legs
    ));
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
    ecs.transforms.insert(
        attacker,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        defender,
        Transform {
            x: 5.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(attacker, EntityKind::Npc);
    ecs.kinds.insert(defender, EntityKind::Npc);
    ecs.personal_needs
        .insert(attacker, PersonalNeeds::default_npc());
    ecs.personal_needs
        .insert(defender, PersonalNeeds::default_npc());
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
    let pn = PersonalNeeds {
        hunger: 0.2,
        thirst: 0.2,
        sleep: 0.1,
        health: 1.0,
        energy: 0.9,
        fear: 0.0,
        curiosity: 0.3,
        ambitions: 0.4,
        discomfort: 0.1,
    };
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
    let n = DestructionNode {
        id: 0,
        position: Vec3::ZERO,
        mass: 1.0,
        material: 0,
        accumulated_stress: 0.5,
    };
    assert_eq!(n.accumulated_stress, 0.5);
}

#[test]
fn destruction_link_broken_flag() {
    let l = DestructionLink {
        a: 0,
        b: 1,
        strength: 100.0,
        fatigue: 0.5,
        broken: true,
    };
    assert!(l.broken);
}

#[test]
fn apply_velocity_updates_cell() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 200.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    let r = DamageResponse::Ricochet {
        position: Vec3::ZERO,
        direction: Vec3::X,
        energy: 10.0,
    };
    assert!(matches!(r, DamageResponse::Ricochet { .. }));
}

#[test]
fn failure_mode_local_break() {
    let section = make_section(10, 0.5, 1);
    let r = evaluate_failure(&section).unwrap();
    assert!(matches!(
        r.failure_mode,
        FailureMode::Crack | FailureMode::LocalBreak
    ));
}

#[test]
fn chain_event_depth() {
    let e = ChainEvent {
        source_entity: Some(1),
        position: Vec3::ZERO,
        energy: 100.0,
        damage_class: DamageClass::Explosive,
        depth: 2,
    };
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
fn physics_system_world_fields_drive_fire_wind_and_water_rain() {
    let mut sys = PhysicsSystem::new(Arc::new(Heightmap::flat(50.0)));
    let mut fields = WorldFields::new();
    fields.wind.base_dir = Vec3::new(1.0, 0.0, 0.0);
    fields.wind.strength = 20.0;
    fields.wind.turbulence = 0.0;
    fields.wind.curl_turbulence = 0.0;
    fields.rain.set_rain(0.75, 1.0);

    let rain = sys.apply_world_fields(&fields, 0.0);
    assert!(rain > 0.7);
    assert!(
        sys.fire.wind[0] > 0.0,
        "canonical wind must reach fire solver input"
    );
    assert!(
        sys.water.rain_intensity > 0.7,
        "canonical rain must reach water solver input"
    );
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
    let r = DamageResponse::StructuralDamage {
        entity: 1,
        section_id: 5,
        energy: 100.0,
    };
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
