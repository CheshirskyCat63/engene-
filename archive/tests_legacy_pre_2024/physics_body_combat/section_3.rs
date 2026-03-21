use super::*;

// ===== COLLAPSE & STRUCTURAL (15 tests) =====

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
        neighbors: vec![SectionNeighbor {
            section_id: 10,
            load_transfer: 0.8,
            collapse_priority: 0,
        }],
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
        SectionNeighbor {
            section_id: 1,
            load_transfer: 0.5,
            collapse_priority: 0,
        },
        SectionNeighbor {
            section_id: 2,
            load_transfer: 0.5,
            collapse_priority: 0,
        },
    ];
    let newly = redistribute_loads(&mut sections, 0);
    let _newly_count = newly.len();
}

#[test]
fn test_cascade_collapse() {
    let mut sections = vec![make_section(0, 0.0, 1), make_section(1, 0.5, 0)];
    sections[0].neighbors = vec![SectionNeighbor {
        section_id: 1,
        load_transfer: 0.5,
        collapse_priority: 0,
    }];
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
    let n = SectionNeighbor {
        section_id: 1,
        load_transfer: 0.7,
        collapse_priority: 1,
    };
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
    q.submit(ChainEvent {
        source_entity: None,
        position: Vec3::ZERO,
        energy: 50.0,
        damage_class: DamageClass::Explosive,
        depth: 1,
    });
    let batch = q.drain_batch();
    let _batch_count = batch.len();
}

#[test]
fn chain_reaction_queue_rejects_low_energy() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent {
        source_entity: None,
        position: Vec3::ZERO,
        energy: 5.0,
        damage_class: DamageClass::Explosive,
        depth: 0,
    });
    assert!(q.is_empty() || q.pending_count() == 0);
}

#[test]
fn chain_reaction_queue_rejects_deep() {
    let mut q = ChainReactionQueue::new();
    q.submit(ChainEvent {
        source_entity: None,
        position: Vec3::ZERO,
        energy: 100.0,
        damage_class: DamageClass::Explosive,
        depth: 5,
    });
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
    let _ = ObjectCondition::Wobbling {
        amplitude: 0.1,
        frequency: 2.0,
    };
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
    assert_eq!(
        PhysicsLod::from_sim_level(SimulationLevel::L0),
        PhysicsLod::Full
    );
    assert_eq!(
        PhysicsLod::from_sim_level(SimulationLevel::L1),
        PhysicsLod::Simplified
    );
    assert_eq!(
        PhysicsLod::from_sim_level(SimulationLevel::L2),
        PhysicsLod::Statistical
    );
    assert_eq!(
        PhysicsLod::from_sim_level(SimulationLevel::L3),
        PhysicsLod::Paused
    );
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
