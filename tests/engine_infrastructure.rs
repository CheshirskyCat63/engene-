// ===== Physics: Chain Reactions =====

#[test]
fn chain_reaction_queue_depth_limit() {
    use engene::physics::chain_reactions::{ChainReactionQueue, ChainEvent};
    use engene::physics::damage_taxonomy::DamageClass;

    let mut queue = ChainReactionQueue::new();
    assert!(queue.is_empty());

    queue.submit(ChainEvent {
        source_entity: None,
        position: glam::Vec3::ZERO,
        energy: 500.0,
        damage_class: DamageClass::Explosive,
        depth: 0,
    });
    assert_eq!(queue.pending_count(), 1);

    queue.submit(ChainEvent {
        source_entity: None,
        position: glam::Vec3::ZERO,
        energy: 500.0,
        damage_class: DamageClass::Explosive,
        depth: 4, // at max depth => rejected
    });
    assert_eq!(queue.pending_count(), 1);

    queue.submit(ChainEvent {
        source_entity: None,
        position: glam::Vec3::ZERO,
        energy: 5.0, // below energy floor => rejected
        damage_class: DamageClass::Blunt,
        depth: 0,
    });
    assert_eq!(queue.pending_count(), 1);
}

#[test]
fn chain_reaction_drain_batch() {
    use engene::physics::chain_reactions::{ChainReactionQueue, ChainEvent};
    use engene::physics::damage_taxonomy::DamageClass;

    let mut queue = ChainReactionQueue::new();
    for i in 0..5 {
        queue.submit(ChainEvent {
            source_entity: Some(i),
            position: glam::Vec3::splat(i as f32),
            energy: 100.0,
            damage_class: DamageClass::Fragmentation,
            depth: 0,
        });
    }
    assert_eq!(queue.pending_count(), 5);

    let batch = queue.drain_batch();
    assert_eq!(batch.len(), 5);
    assert!(queue.is_empty());
}

// ===== Physics: Damage Taxonomy =====

#[test]
fn damage_class_instant_vs_cumulative() {
    use engene::physics::damage_taxonomy::DamageClass;

    assert!(DamageClass::Ballistic.is_instant());
    assert!(DamageClass::Explosive.is_instant());
    assert!(DamageClass::Fragmentation.is_instant());

    assert!(DamageClass::Thermal.is_cumulative());
    assert!(DamageClass::Erosion.is_cumulative());
    assert!(DamageClass::Fatigue.is_cumulative());
    assert!(DamageClass::Corrosion.is_cumulative());
}

#[test]
fn damage_capability_bitflags() {
    use engene::physics::damage_taxonomy::DamageCapability;

    let cap = DamageCapability::SURFACE | DamageCapability::THERMAL;
    assert!(cap.contains(DamageCapability::SURFACE));
    assert!(cap.contains(DamageCapability::THERMAL));
    assert!(!cap.contains(DamageCapability::MOISTURE));
}

// ===== Physics: Cloth Simulation =====

#[test]
fn cloth_sim_creation_and_step() {
    use engene::physics::cloth::ClothSim;

    let mut cloth = ClothSim::new(0, 4, 4, [0.0, 5.0, 0.0], 1.0);
    assert_eq!(cloth.particles.len(), 16);
    assert!(cloth.constraints.len() > 0);

    assert_eq!(cloth.particles[0].inv_mass, 0.0);
    assert!(cloth.particles[5].inv_mass > 0.0);

    let initial_y = cloth.particles[15].pos[1];
    cloth.step(1.0 / 60.0, 4);
    assert!(cloth.particles[15].pos[1] < initial_y, "gravity should pull particles down");

    let flat = cloth.positions_flat();
    assert_eq!(flat.len(), 48);
}

#[test]
fn cloth_world_manages_multiple() {
    use engene::physics::cloth::ClothWorld;

    let mut world = ClothWorld::new();
    let id1 = world.add_cloth(3, 3, [0.0, 5.0, 0.0], 1.0);
    let id2 = world.add_cloth(4, 4, [10.0, 5.0, 0.0], 1.0);
    assert_ne!(id1, id2);
    assert_eq!(world.cloths.len(), 2);
}

// ===== Physics: Fire Grid =====

#[test]
fn fire_grid_ignite_and_burn() {
    use engene::physics::fire::*;

    let mut grid = FireGrid::new();
    grid.set_fuel(5, 5, 10.0, 0.8);
    assert_eq!(grid.get(5, 5).state, FireState::Unlit);

    grid.ignite(5, 5);
    assert_eq!(grid.get(5, 5).state, FireState::Burning);
    assert_eq!(grid.active_fire_count(), 1);
    assert_eq!(grid.burning_cells().len(), 1);
}

#[test]
fn fire_grid_extinguish() {
    use engene::physics::fire::*;

    let mut grid = FireGrid::new();
    grid.set_fuel(3, 3, 10.0, 0.8);
    grid.ignite(3, 3);
    assert_eq!(grid.get(3, 3).state, FireState::Burning);

    grid.extinguish(3, 3);
    assert_eq!(grid.get(3, 3).state, FireState::Unlit);
    assert_eq!(grid.active_fire_count(), 0);
}

#[test]
fn fire_grid_no_ignite_without_fuel() {
    use engene::physics::fire::*;

    let mut grid = FireGrid::new();
    grid.ignite(2, 2);
    assert_eq!(grid.get(2, 2).state, FireState::Unlit);
}

// ===== Physics: Water Grid =====

#[test]
fn water_grid_add_and_query() {
    use engene::physics::water::WaterGrid;

    let mut grid = WaterGrid::new();
    grid.set_terrain_height(5, 5, 10.0);
    grid.add_water(5, 5, 3.0);

    let cell = grid.get(5, 5);
    assert!((cell.water_level - 3.0).abs() < 0.01);
    assert!((cell.height - 10.0).abs() < 0.01);
}

#[test]
fn water_grid_container_holds_water() {
    use engene::physics::water::WaterGrid;

    let mut grid = WaterGrid::new();
    grid.mark_container(3, 3);
    grid.add_water(3, 3, 5.0);

    let cell = grid.get(3, 3);
    assert!(cell.is_container);
    assert!(cell.water_level > 0.0);
}

#[test]
fn water_grid_buoyancy_force() {
    use engene::physics::water::WaterGrid;
    use engene::world::cell::CELL_SIZE;

    let mut grid = WaterGrid::new();
    grid.set_terrain_height(0, 0, 0.0);
    grid.add_water(0, 0, 5.0);

    let force = grid.buoyancy_force(CELL_SIZE * 0.5, CELL_SIZE * 0.5, 2.0);
    assert!(force > 0.0, "submerged object should get buoyancy");

    let no_force = grid.buoyancy_force(CELL_SIZE * 0.5, CELL_SIZE * 0.5, 100.0);
    assert_eq!(no_force, 0.0, "object above water gets no buoyancy");
}

// ===== Navigation: World Graph =====

#[test]
fn world_graph_pathfinding() {
    use engene::navigation::world_graph::WorldGraph;

    let graph = WorldGraph::build_default();
    assert_eq!(graph.locations.len(), 5);

    let village = 0;
    let forest_n = 1;
    let path = graph.find_path(village, forest_n);
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(*path.first().unwrap(), village);
    assert_eq!(*path.last().unwrap(), forest_n);
}

#[test]
fn world_graph_disconnected_no_path() {
    use engene::navigation::world_graph::WorldGraph;

    let mut graph = WorldGraph::new();
    let a = graph.add_location("A", 0, 0);
    let b = graph.add_location("B", 10, 10);

    let path = graph.find_path(a, b);
    assert!(path.is_none());
}

#[test]
fn world_graph_neighbors() {
    use engene::navigation::world_graph::WorldGraph;

    let graph = WorldGraph::build_default();
    let village_neighbors = graph.neighbors(0);
    assert!(village_neighbors.len() >= 3, "village should connect to 3 locations");
}

// ===== Navigation: RVO Avoidance =====

#[test]
fn rvo_system_single_agent() {
    use engene::navigation::avoidance::RvoSystem;

    let mut rvo = RvoSystem::new();
    rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
    rvo.compute();

    let vel = rvo.get_velocity(1).unwrap();
    assert!((vel[0] - 1.0).abs() < 0.01);
    assert!((vel[1] - 0.0).abs() < 0.01);
}

#[test]
fn rvo_system_collision_avoidance() {
    use engene::navigation::avoidance::RvoSystem;

    let mut rvo = RvoSystem::new();
    rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
    rvo.add_agent(2, [0.5, 0.0], [-1.0, 0.0], [-1.0, 0.0]);
    rvo.compute();

    let v1 = rvo.get_velocity(1).unwrap();
    let v2 = rvo.get_velocity(2).unwrap();
    assert!(v1[1].abs() > 0.01 || v2[1].abs() > 0.01 ||
            v1[0] < 1.0 || v2[0] > -1.0,
            "agents should adjust to avoid collision");
}

// ===== Navigation: NavDirtyTracker =====

#[test]
fn nav_dirty_tracker_marking_and_draining() {
    use engene::navigation::dynamic_nav_update::NavDirtyTracker;

    let mut tracker = NavDirtyTracker::new();
    assert!(tracker.is_empty());

    tracker.mark_dirty(5, 5);
    tracker.mark_dirty(6, 6);
    assert_eq!(tracker.pending_count(), 2);

    let batch = tracker.drain_dirty_batch();
    assert_eq!(batch.len(), 2);
    assert!(tracker.is_empty());
}

#[test]
fn nav_dirty_tracker_area_marking() {
    use engene::navigation::dynamic_nav_update::NavDirtyTracker;

    let mut tracker = NavDirtyTracker::new();
    tracker.mark_area_dirty(glam::Vec3::new(100.0, 0.0, 100.0), 20.0, 10.0);
    assert!(tracker.pending_count() >= 9, "area should dirty multiple cells");
}

#[test]
fn nav_dirty_tracker_cluster_rebuilds() {
    use engene::navigation::dynamic_nav_update::NavDirtyTracker;

    let mut tracker = NavDirtyTracker::new();
    tracker.mark_cluster_rebuild(1);
    tracker.mark_cluster_rebuild(2);
    tracker.mark_cluster_rebuild(1); // duplicate

    let rebuilds = tracker.drain_cluster_rebuilds();
    assert_eq!(rebuilds.len(), 2);
}

// ===== Animation: Locomotion =====

#[test]
fn locomotion_state_machine() {
    use engene::animation::locomotion::*;

    let mut loco = LocomotionMachine::new();
    assert_eq!(loco.state, LocomotionState::Idle);

    loco.set_clip(LocomotionState::Idle, 0);
    loco.set_clip(LocomotionState::Walk, 1);
    loco.set_clip(LocomotionState::Run, 2);
    loco.set_clip(LocomotionState::Death, 3);

    assert_eq!(loco.current_clip(), Some(0));

    loco.update(2.0, false, 0.016);
    assert_eq!(loco.state, LocomotionState::Walk);
    assert_eq!(loco.current_clip(), Some(1));

    loco.update(5.0, false, 0.016);
    assert_eq!(loco.state, LocomotionState::Run);

    loco.update(0.0, true, 0.016);
    assert_eq!(loco.state, LocomotionState::Death);
    assert_eq!(loco.current_clip(), Some(3));
}

// ===== Input State =====

#[test]
fn input_state_key_tracking() {
    use engene::input::input::InputState;
    use winit::keyboard::KeyCode;

    let mut input = InputState::new();
    assert!(!input.is_key_down(KeyCode::KeyW));

    input.key_pressed(KeyCode::KeyW);
    assert!(input.is_key_down(KeyCode::KeyW));

    input.key_released(KeyCode::KeyW);
    assert!(!input.is_key_down(KeyCode::KeyW));
}

#[test]
fn input_state_mouse_capture() {
    use engene::input::input::InputState;

    let mut input = InputState::new();
    input.accumulate_mouse(10.0, 20.0);
    assert_eq!(input.mouse_dx, 0.0, "uncaptured mouse should not accumulate");

    input.mouse_captured = true;
    input.accumulate_mouse(10.0, 20.0);
    assert!((input.mouse_dx - 10.0).abs() < 0.01);
    assert!((input.mouse_dy - 20.0).abs() < 0.01);

    input.end_frame();
    assert_eq!(input.mouse_dx, 0.0);
    assert_eq!(input.mouse_dy, 0.0);
}

// ===== Network: Interpolation Buffer =====

#[cfg(feature = "networking")]
#[test]
fn interpolation_buffer_snapshots() {
    use engene::network::interpolation::InterpolationBuffer;
    use engene::memory::component_delta::ComponentDelta;

    let mut buf = InterpolationBuffer::new();
    assert_eq!(buf.snapshot_count(), 0);
    assert_eq!(buf.latest_tick(), 0);

    buf.push_snapshot(1, ComponentDelta { component_type_id: 0, entity_ids: vec![1], data: vec![0] });
    buf.advance(0.016);
    buf.push_snapshot(2, ComponentDelta { component_type_id: 0, entity_ids: vec![1], data: vec![0] });

    assert_eq!(buf.snapshot_count(), 2);
    assert_eq!(buf.latest_tick(), 2);
}

#[cfg(feature = "networking")]
#[test]
fn interpolation_factor_range() {
    use engene::network::interpolation::InterpolationBuffer;
    use engene::memory::component_delta::ComponentDelta;

    let mut buf = InterpolationBuffer::new();
    let factor = buf.interpolation_factor();
    assert!((factor - 1.0).abs() < 0.01, "no snapshots => 1.0");

    for tick in 0..4 {
        buf.push_snapshot(tick, ComponentDelta { component_type_id: 0, entity_ids: vec![], data: vec![] });
        buf.advance(0.016);
    }

    let f = buf.interpolation_factor();
    assert!(f >= 0.0 && f <= 1.0);
}

// ===== Memory: Dirty Flags =====

#[test]
fn dirty_flags_mark_and_query() {
    use engene::memory::component_delta::DirtyFlags;

    let mut flags = DirtyFlags::new(100);
    assert!(!flags.is_dirty(5, 0));
    assert!(!flags.any_dirty(5));

    flags.mark(5, 0);
    flags.mark(5, 3);
    assert!(flags.is_dirty(5, 0));
    assert!(flags.is_dirty(5, 3));
    assert!(!flags.is_dirty(5, 1));
    assert!(flags.any_dirty(5));

    flags.clear_all();
    assert!(!flags.any_dirty(5));
}

// ===== World: Spatial Index =====

#[test]
fn spatial_index_insert_and_query() {
    use engene::world::spatial_index::SpatialIndex;

    let mut idx = SpatialIndex::new();
    idx.insert(1, 100.0, 100.0);
    idx.insert(2, 105.0, 105.0);
    idx.insert(3, 5000.0, 5000.0);

    let near = idx.candidates_in_radius(100.0, 100.0, 500.0);
    assert!(near.contains(&1));
    assert!(near.contains(&2));
    assert!(!near.contains(&3));
}

#[test]
fn spatial_index_clear() {
    use engene::world::spatial_index::SpatialIndex;

    let mut idx = SpatialIndex::new();
    idx.insert(1, 100.0, 100.0);
    idx.clear();

    let near = idx.candidates_in_radius(100.0, 100.0, 500.0);
    assert!(near.is_empty());
}

// ===== World: Terrain Masks =====

#[test]
fn terrain_mask_store_apply_and_decay() {
    use engene::world::terrain_masks::TerrainMaskStore;

    let mut store = TerrainMaskStore::new();
    store.apply_scorch(5, 5, 0.8);
    store.apply_wetness(5, 5, 0.6);

    let patch = store.get(5, 5).unwrap();
    assert!((patch.scorched - 0.8).abs() < 0.01);
    assert!((patch.wetness - 0.6).abs() < 0.01);

    store.update(100.0);
    let patch_after = store.get(5, 5).unwrap();
    assert!(patch_after.scorched <= 0.8);
    assert!(patch_after.wetness < 0.6);
}

// ===== World: Surface State =====

#[test]
fn surface_state_store_basic_operations() {
    use engene::world::surface_state::SurfaceStateStore;
    use engene::physics::damage_pipeline::response_aggregator::SurfaceMaskType;

    let mut store = SurfaceStateStore::new();
    assert_eq!(store.active_count(), 0);

    store.apply_mask_delta(10, 10, SurfaceMaskType::Scorch, 0.5);
    assert!(store.active_count() > 0);

    let patch = store.get(10, 10);
    assert!(patch.is_some());
}

// ===== World: Terrain Deformation =====

#[test]
fn terrain_deformation_submit_and_process() {
    use engene::world::terrain_deformation::TerrainDeformationSystem;
    use engene::world::terrain_damage::CraterStamp;

    let mut sys = TerrainDeformationSystem::new();
    assert_eq!(sys.patch_count(), 0);

    sys.submit_crater(CraterStamp::from_explosion(
        glam::Vec3::new(100.0, 0.0, 100.0),
        500.0,
    ));
    sys.process_frame();

    assert!(sys.patch_count() > 0);
    let dirty = sys.drain_dirty_patches();
    assert!(!dirty.is_empty());
}

// ===== World: Origin Shift =====

#[test]
fn origin_shift_threshold() {
    use engene::world::origin_shift::OriginShift;

    assert!(!OriginShift::needs_shift(0.0, 0.0));
    assert!(OriginShift::needs_shift(10000.0, 10000.0));

    let mut shift = OriginShift::new();
    let (sx, sz) = OriginShift::compute_shift(5000.0, 3000.0);
    shift.apply_shift(sx, sz);
    assert_eq!(shift.shift_count, 1);

    let (ax, az) = shift.world_to_absolute(0.0, 0.0);
    assert!(ax.abs() > 0.0 || az.abs() > 0.0);
}

// ===== Block 1: Identity Invariant Tests =====

#[test]
fn identity_every_persistent_entity_has_pid() {
    use engene::core::ecs::Ecs;
    let mut ecs = Ecs::new();
    let (e1, pid1) = ecs.spawn_new();
    let (e2, pid2) = ecs.spawn_new();
    assert!(ecs.identity.persistent_id_of(e1).is_some());
    assert!(ecs.identity.persistent_id_of(e2).is_some());
    assert_ne!(pid1, pid2);
}

#[test]
fn identity_no_duplicate_pid_after_restore() {
    use engene::core::ecs::Ecs;
    let mut ecs = Ecs::new();
    let (_e1, pid1) = ecs.spawn_new();
    let result = ecs.spawn_restored(pid1);
    assert!(result.is_err(), "duplicate PID should be rejected");
}

#[test]
fn identity_tombstone_transitions() {
    use engene::core::persistent_id::{EntityPresence, IdentityRegistry};
    let mut reg = IdentityRegistry::new();
    let pid = reg.register_new(1);
    assert!(matches!(reg.presence(pid), EntityPresence::Live(1)));
    reg.mark_unloaded(pid);
    assert!(matches!(reg.presence(pid), EntityPresence::Unloaded));
    reg.mark_dead(pid, 100);
    assert!(matches!(reg.presence(pid), EntityPresence::Dead));
    assert_eq!(reg.tombstone_count(), 1);
    reg.gc_tombstones(300, 100);
    assert_eq!(reg.tombstone_count(), 0);
}

#[test]
fn identity_unloaded_entity_not_dead() {
    use engene::core::persistent_id::{EntityPresence, IdentityRegistry};
    let mut reg = IdentityRegistry::new();
    let pid = reg.register_new(1);
    reg.mark_unloaded(pid);
    assert!(!matches!(reg.presence(pid), EntityPresence::Dead));
    assert!(matches!(reg.presence(pid), EntityPresence::Unloaded));
}

#[test]
fn identity_entity_ref_resolution() {
    use engene::core::persistent_id::{EntityRef, IdentityRegistry};
    let mut reg = IdentityRegistry::new();
    let pid = reg.register_new(42);
    let eref = EntityRef::new(pid);
    assert_eq!(eref.resolve(&reg), Some(42));
    reg.mark_unloaded(pid);
    assert!(eref.resolve(&reg).is_none());
    assert!(eref.is_unloaded(&reg));
    assert!(!eref.is_dead(&reg));
}

#[test]
fn identity_relink_context_resolves() {
    use engene::core::persistent_id::{IdentityRegistry, RelinkContext};
    let mut reg = IdentityRegistry::new();
    let pid = reg.register_new(10);
    let ctx = RelinkContext { registry: &reg };
    assert_eq!(ctx.resolve_persistent(pid), Some(10));
}
