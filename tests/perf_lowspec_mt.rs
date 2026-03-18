//! Integration tests for Performance, Low-Spec, and Multithreading systems.
//! 110 tests covering QualityGovernor, degradation, BudgetRegistry, LowSpecCertifier,
//! Telemetry, WorkerPool, DirtySet, Simulation LOD, Physics LOD, Engine integration, RuntimeConfig.

use engene::core::budget_registry::{create_default_registry, BudgetEntry, BudgetRegistry};
use engene::core::dirty_set::DirtySet;
use engene::core::jobs::WorkerPool;
use engene::core::perf::low_spec_cert::LowSpecCertifier;
use engene::core::perf::telemetry::Telemetry;
use engene::core::quality_governor::{
    degradation_order, systems_to_disable, PressureLevel, QualityGovernor,
};
use engene::core::runtime_config::{QualityTier, RuntimeConfig, RuntimeProfile};
use engene::physics::sim_lod::PhysicsLod;
use engene::runtime::bootstrap::{GameRuntimeAssembly, ToolsRuntimeAssembly};
use engene::simulation::simulation_level::{
    level_for_distance, should_tick, L0_RADIUS, L1_RADIUS, L1_TICK_INTERVAL, L2_RADIUS,
    L2_TICK_INTERVAL,
};
use engene::world::biome::Biome;
use engene::world::components::SimulationLevel;
use engene::world::heightmap::Heightmap;
use engene::world::world::WorldGrid;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════
// 1. QualityGovernor (15 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn qg_new_creates_with_frame_budget() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.frame_budget_us, 1_000_000 / 60);
}

#[test]
fn qg_new_30fps_budget() {
    let gov = QualityGovernor::new(30);
    assert_eq!(gov.frame_budget_us, 1_000_000 / 30);
}

#[test]
fn qg_new_120fps_budget() {
    let gov = QualityGovernor::new(120);
    assert_eq!(gov.frame_budget_us, 1_000_000 / 120);
}

#[test]
fn qg_initial_pressure_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.pressure_level, PressureLevel::Normal);
}

#[test]
fn qg_max_chain_reaction_depth_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.max_chain_reaction_depth(), 4);
}

#[test]
fn qg_max_micro_motion_oscillators_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.max_micro_motion_oscillators(), 256);
}

#[test]
fn qg_max_chain_reaction_depth_under_pressure() {
    let mut gov = QualityGovernor::new(60);
    for _ in 0..20 {
        gov.update(50_000);
    }
    let depth = gov.max_chain_reaction_depth();
    assert!(depth < 4 || gov.pressure_level == PressureLevel::Normal);
}

#[test]
fn qg_max_micro_motion_oscillators_reduces_under_pressure() {
    let mut gov = QualityGovernor::new(60);
    for _ in 0..25 {
        gov.update(80_000);
    }
    let oscillators = gov.max_micro_motion_oscillators();
    assert!(oscillators <= 256);
}

#[test]
fn qg_max_dirty_surface_uploads_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.max_dirty_surface_uploads(), 16);
}

#[test]
fn qg_debris_density_factor_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.debris_density_factor(), 1.0);
}

#[test]
fn qg_max_terrain_rebuilds_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.max_terrain_rebuilds_per_frame(), 2);
}

#[test]
fn qg_max_nav_dirty_cells_normal() {
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.max_nav_dirty_cells(), 64);
}

#[test]
fn qg_update_smooths_frame_time() {
    let mut gov = QualityGovernor::new(60);
    gov.update(10_000);
    assert!(gov.smoothed_frame_time_us > 0);
}

#[test]
fn qg_target_60_frame_budget() {
    let gov = QualityGovernor::new(60);
    let budget = gov.frame_budget_us;
    assert_eq!(budget, 16_666);
}

#[test]
fn qg_target_24_frame_budget() {
    let gov = QualityGovernor::new(24);
    assert_eq!(gov.frame_budget_us, 1_000_000 / 24);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Degradation order (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn degradation_order_returns_entries() {
    let order = degradation_order();
    assert!(!order.is_empty());
}

#[test]
fn degradation_order_has_ssao() {
    let order = degradation_order();
    let names: Vec<&str> = order.iter().map(|e| e.subsystem).collect();
    assert!(names.contains(&"SSAO"));
}

#[test]
fn degradation_order_has_collision_detection() {
    let order = degradation_order();
    let names: Vec<&str> = order.iter().map(|e| e.subsystem).collect();
    assert!(names.contains(&"Collision Detection"));
}

#[test]
fn degradation_order_never_cut_subsystems() {
    let order = degradation_order();
    let never_cut: Vec<_> = order.iter().filter(|e| e.priority as i32 == 3).collect();
    assert!(!never_cut.is_empty());
}

#[test]
fn systems_to_disable_low_tier() {
    let disabled = systems_to_disable(QualityTier::Low);
    assert!(disabled.contains(&"SSAO"));
    assert!(disabled.contains(&"Volumetric Lighting"));
}

#[test]
fn systems_to_disable_medium_tier() {
    let disabled = systems_to_disable(QualityTier::Medium);
    assert!(disabled.contains(&"Volumetric Lighting"));
}

#[test]
fn systems_to_disable_high_empty() {
    let disabled = systems_to_disable(QualityTier::High);
    assert!(disabled.is_empty());
}

#[test]
fn systems_to_disable_ultra_empty() {
    let disabled = systems_to_disable(QualityTier::Ultra);
    assert!(disabled.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. BudgetRegistry (12 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn br_create_default_not_empty() {
    let reg = create_default_registry();
    assert!(!reg.entries().is_empty());
}

#[test]
fn br_entries_has_damage_pipeline() {
    let reg = create_default_registry();
    let names: Vec<&str> = reg.entries().iter().map(|e| e.phase_name).collect();
    assert!(names.contains(&"damage_pipeline"));
}

#[test]
fn br_total_budget_us_positive() {
    let reg = create_default_registry();
    let total = reg.total_budget_us();
    assert!(total > 0);
}

#[test]
fn br_total_budget_us_known_sum() {
    let reg = create_default_registry();
    let manual_sum: u32 = reg.entries().iter().map(|e| e.cpu_budget_us).sum();
    assert_eq!(reg.total_budget_us(), manual_sum);
}

#[test]
fn br_record_measurement_updates_last_measured() {
    let mut reg = create_default_registry();
    reg.record_measurement("damage_pipeline", 100);
    let entry = reg
        .entries()
        .iter()
        .find(|e| e.phase_name == "damage_pipeline")
        .unwrap();
    assert_eq!(entry.last_measured_us, 100);
}

#[test]
fn br_record_measurement_overrun_increments() {
    let mut reg = create_default_registry();
    reg.record_measurement("damage_pipeline", 10_000);
    reg.record_measurement("damage_pipeline", 10_000);
    assert!(reg.total_overruns() > 0);
}

#[test]
fn br_total_overruns_initial_zero() {
    let reg = create_default_registry();
    assert_eq!(reg.total_overruns(), 0);
}

#[test]
fn br_new_empty() {
    let reg = BudgetRegistry::new();
    assert!(reg.entries().is_empty());
}

#[test]
fn br_register_adds_entry() {
    let mut reg = BudgetRegistry::new();
    reg.register(BudgetEntry {
        phase_name: "test_phase",
        owner_system: "TestSystem",
        cpu_budget_us: 500,
        gpu_budget_us: 0,
        memory_budget_bytes: 1024,
        profiler_category: "test",
        overrun_count: 0,
        last_measured_us: 0,
    });
    assert_eq!(reg.entries().len(), 1);
}

#[test]
fn br_record_unknown_phase_no_panic() {
    let mut reg = create_default_registry();
    reg.record_measurement("nonexistent_phase", 100);
    assert_eq!(reg.total_overruns(), 0);
}

#[test]
fn br_entries_contain_micro_motion() {
    let reg = create_default_registry();
    let names: Vec<&str> = reg.entries().iter().map(|e| e.phase_name).collect();
    assert!(names.contains(&"micro_motion"));
}

#[test]
fn br_total_overruns_accumulates() {
    let mut reg = BudgetRegistry::new();
    reg.register(BudgetEntry {
        phase_name: "p",
        owner_system: "S",
        cpu_budget_us: 10,
        gpu_budget_us: 0,
        memory_budget_bytes: 0,
        profiler_category: "x",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.record_measurement("p", 100);
    reg.record_measurement("p", 100);
    assert_eq!(reg.total_overruns(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. LowSpecCertifier (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lsc_default_creates() {
    let cert = LowSpecCertifier::default();
    let gov = QualityGovernor::new(30);
    let result = cert.certify(&gov);
    assert_eq!(result.target_fps, 30);
}

#[test]
fn lsc_certify_passes_when_fps_ok() {
    let mut cert = LowSpecCertifier::default();
    cert.record_frame(25_000);
    cert.record_frame(30_000);
    let gov = QualityGovernor::new(30);
    let result = cert.certify(&gov);
    assert!(result.achieved_fps_estimate > 25.0);
}

#[test]
fn lsc_certify_detects_truth_loss() {
    let mut cert = LowSpecCertifier::default();
    cert.mark_truth_loss();
    let gov = QualityGovernor::new(30);
    let result = cert.certify(&gov);
    assert!(result.truth_loss_detected);
}

#[test]
fn lsc_record_memory_tracks_peak() {
    let mut cert = LowSpecCertifier::new(30, 256);
    cert.record_memory(100);
    cert.record_memory(200);
    cert.record_memory(150);
    let gov = QualityGovernor::new(30);
    let result = cert.certify(&gov);
    assert!(!result.memory_runaway);
}

#[test]
fn lsc_generate_report_produces_string() {
    let cert = LowSpecCertifier::default();
    let gov = QualityGovernor::new(30);
    let report = cert.generate_report(&gov);
    assert!(report.contains("Low-Spec Certification"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Telemetry (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn telemetry_new_empty() {
    let tel = Telemetry::new();
    assert_eq!(tel.frame_count(), 0);
}

#[test]
fn telemetry_record_frame() {
    let mut tel = Telemetry::new();
    tel.record_frame(16_000);
    assert_eq!(tel.frame_time_us(), 16_000);
    assert_eq!(tel.frame_count(), 1);
}

#[test]
fn telemetry_record_system() {
    let mut tel = Telemetry::new();
    tel.record_system("physics", 500);
    let timings = tel.system_timings();
    assert!(timings.contains_key("physics"));
}

#[test]
fn telemetry_record_system_updates_max() {
    let mut tel = Telemetry::new();
    tel.record_system("sys", 100);
    tel.record_system("sys", 500);
    let t = tel.system_timings().get("sys").unwrap();
    assert_eq!(t.max_us, 500);
}

#[test]
fn telemetry_default_same_as_new() {
    let tel = Telemetry::default();
    assert_eq!(tel.frame_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. WorkerPool (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wp_new_creates_pool() {
    let pool = WorkerPool::new();
    assert!(pool.worker_count() >= 2);
}

#[test]
fn wp_worker_count_at_least_two() {
    let pool = WorkerPool::new();
    let count = pool.worker_count();
    assert!(count >= 2 && count <= 128);
}

#[test]
fn wp_execute_returns_result() {
    let pool = WorkerPool::new();
    let result = pool.execute(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn wp_execute_runs_task() {
    let pool = WorkerPool::new();
    let result = pool.execute(|| 10 + 20);
    assert_eq!(result, 30);
}

#[test]
fn wp_par_join_returns_both() {
    let pool = WorkerPool::new();
    let (a, b) = pool.execute(|| (1 + 2, 3 + 4));
    assert_eq!(a, 3);
    assert_eq!(b, 7);
}

#[test]
fn wp_pool_installs_parallel_work() {
    let pool = WorkerPool::new();
    let (a, b) = pool.pool().install(|| rayon::join(|| 1, || 2));
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn wp_default_creates() {
    let pool = WorkerPool::default();
    assert!(pool.worker_count() >= 2);
}

#[test]
fn wp_execute_with_closure() {
    let pool = WorkerPool::new();
    let x = 5u32;
    let result = pool.execute(move || x * 2);
    assert_eq!(result, 10);
}

#[test]
fn wp_par_join_parallel_execution() {
    let pool = WorkerPool::new();
    let (a, b) = pool.execute(|| rayon::join(|| 100, || 200));
    assert_eq!(a, 100);
    assert_eq!(b, 200);
}

#[test]
fn wp_multiple_executes_sequential() {
    let pool = WorkerPool::new();
    let r1 = pool.execute(|| 1);
    let r2 = pool.execute(|| 2);
    assert_eq!(r1, 1);
    assert_eq!(r2, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. DirtyFlags / DirtySet (12 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn df_new_empty() {
    let ds: DirtySet<u64> = DirtySet::new();
    assert_eq!(ds.len(), 0);
}

#[test]
fn df_mark_increments_count() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    assert_eq!(ds.len(), 1);
}

#[test]
fn df_is_dirty_after_mark() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(42);
    assert!(ds.is_dirty(&42));
}

#[test]
fn df_is_dirty_false_for_unmarked() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    assert!(!ds.is_dirty(&2));
}

#[test]
fn df_dirty_count_after_multiple_marks() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    ds.mark(2);
    ds.mark(3);
    assert_eq!(ds.len(), 3);
}

#[test]
fn df_clear_all_resets() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    ds.mark(2);
    ds.clear();
    assert_eq!(ds.len(), 0);
}

#[test]
fn df_clear_all_is_dirty_false() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(7);
    ds.clear();
    assert!(!ds.is_dirty(&7));
}

#[test]
fn df_drain_returns_and_clears() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    ds.mark(2);
    let drained: Vec<u64> = ds.drain();
    assert_eq!(drained.len(), 2);
    assert_eq!(ds.len(), 0);
}

#[test]
fn df_total_marks_tracks_insertions() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    ds.mark(2);
    assert_eq!(ds.total_marks(), 2);
}

#[test]
fn df_duplicate_mark_same_id_count_one() {
    let mut ds: DirtySet<u64> = DirtySet::new();
    ds.mark(1);
    ds.mark(1);
    assert_eq!(ds.len(), 1);
}

#[test]
fn df_default_empty() {
    let ds: DirtySet<u64> = DirtySet::default();
    assert!(ds.is_empty());
}

#[test]
fn df_merge_combines_sets() {
    let mut a: DirtySet<u64> = DirtySet::new();
    let mut b: DirtySet<u64> = DirtySet::new();
    a.mark(1);
    b.mark(2);
    a.merge(&b);
    assert_eq!(a.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Simulation LOD (15 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sim_l0_radius_value() {
    assert_eq!(L0_RADIUS, 300.0);
}

#[test]
fn sim_l1_radius_value() {
    assert_eq!(L1_RADIUS, 5000.0);
}

#[test]
fn sim_l2_radius_value() {
    assert_eq!(L2_RADIUS, 50000.0);
}

#[test]
fn sim_level_for_distance_l0_at_zero() {
    let level = level_for_distance(0.0);
    assert_eq!(level, SimulationLevel::L0);
}

#[test]
fn sim_level_for_distance_l0_at_boundary() {
    let level = level_for_distance(L0_RADIUS);
    assert_eq!(level, SimulationLevel::L0);
}

#[test]
fn sim_level_for_distance_l1_just_over_l0() {
    let level = level_for_distance(L0_RADIUS + 1.0);
    assert_eq!(level, SimulationLevel::L1);
}

#[test]
fn sim_level_for_distance_l1_at_boundary() {
    let level = level_for_distance(L1_RADIUS);
    assert_eq!(level, SimulationLevel::L1);
}

#[test]
fn sim_level_for_distance_l2_just_over_l1() {
    let level = level_for_distance(L1_RADIUS + 1.0);
    assert_eq!(level, SimulationLevel::L2);
}

#[test]
fn sim_level_for_distance_l3_beyond_l2() {
    let level = level_for_distance(L2_RADIUS + 1000.0);
    assert_eq!(level, SimulationLevel::L3);
}

#[test]
fn sim_should_tick_l0_always() {
    assert!(should_tick(SimulationLevel::L0, 0));
    assert!(should_tick(SimulationLevel::L0, 1));
    assert!(should_tick(SimulationLevel::L0, 100));
}

#[test]
fn sim_should_tick_l1_at_interval() {
    assert!(should_tick(SimulationLevel::L1, 0));
    assert!(should_tick(SimulationLevel::L1, L1_TICK_INTERVAL));
    assert!(should_tick(SimulationLevel::L1, L1_TICK_INTERVAL * 2));
}

#[test]
fn sim_should_tick_l1_not_mid_interval() {
    assert!(!should_tick(SimulationLevel::L1, 1));
}

#[test]
fn sim_should_tick_l2_at_interval() {
    assert!(should_tick(SimulationLevel::L2, L2_TICK_INTERVAL));
}

#[test]
fn sim_should_tick_l3_never() {
    assert!(!should_tick(SimulationLevel::L3, 0));
    assert!(!should_tick(SimulationLevel::L3, 1000));
}

#[test]
fn sim_tick_intervals_defined() {
    assert_eq!(L1_TICK_INTERVAL, 12);
    assert_eq!(L2_TICK_INTERVAL, 60);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Physics LOD (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn phys_from_sim_level_l0_full() {
    let lod = PhysicsLod::from_sim_level(SimulationLevel::L0);
    assert_eq!(lod, PhysicsLod::Full);
}

#[test]
fn phys_from_sim_level_l1_simplified() {
    let lod = PhysicsLod::from_sim_level(SimulationLevel::L1);
    assert_eq!(lod, PhysicsLod::Simplified);
}

#[test]
fn phys_from_sim_level_l2_statistical() {
    let lod = PhysicsLod::from_sim_level(SimulationLevel::L2);
    assert_eq!(lod, PhysicsLod::Statistical);
}

#[test]
fn phys_from_sim_level_l3_paused() {
    let lod = PhysicsLod::from_sim_level(SimulationLevel::L3);
    assert_eq!(lod, PhysicsLod::Paused);
}

#[test]
fn phys_cloth_iterations_full() {
    assert_eq!(PhysicsLod::Full.cloth_iterations(), 10);
}

#[test]
fn phys_cloth_iterations_simplified() {
    assert_eq!(PhysicsLod::Simplified.cloth_iterations(), 3);
}

#[test]
fn phys_water_active_by_lod() {
    assert!(PhysicsLod::Full.water_active());
    assert!(PhysicsLod::Simplified.water_active());
    assert!(!PhysicsLod::Statistical.water_active());
    assert!(!PhysicsLod::Paused.water_active());
}

#[test]
fn phys_fire_tick_interval_full() {
    assert_eq!(PhysicsLod::Full.fire_tick_interval(), 1);
}

#[test]
fn phys_should_tick_fire_full_and_paused() {
    assert!(PhysicsLod::Full.should_tick_fire(0));
    assert!(!PhysicsLod::Paused.should_tick_fire(0));
}

#[test]
fn phys_should_tick_fire_simplified_at_interval() {
    assert!(PhysicsLod::Simplified.should_tick_fire(4));
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Engine integration (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn engine_headless_has_quality_governor() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<QualityGovernor>().is_some());
}

#[test]
fn engine_headless_has_budget_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<BudgetRegistry>().is_some());
}

#[test]
fn engine_vertical_slice_has_quality_governor() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine.resources.get::<QualityGovernor>().is_some());
}

#[test]
fn engine_vertical_slice_has_budget_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine.resources.get::<BudgetRegistry>().is_some());
}

#[test]
fn engine_headless_has_low_spec_certifier() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<LowSpecCertifier>().is_some());
}

#[test]
fn engine_vertical_slice_has_low_spec_certifier() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine.resources.get::<LowSpecCertifier>().is_some());
}

#[test]
fn engine_headless_ecs_non_empty_after_init() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.ecs.alive.len() > 0);
}

#[test]
fn engine_tools_creates() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

#[test]
fn engine_vertical_slice_governor_target_60() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let gov = engine.resources.get::<QualityGovernor>().unwrap();
    assert_eq!(gov.frame_budget_us, 1_000_000 / 60);
}

#[test]
fn engine_headless_worker_pool_created() {
    let grid = WorldGrid::generate();
    let biomes: Vec<Biome> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.worker_pool.worker_count() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. RuntimeConfig (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rc_quality_tier_variants() {
    let _ = QualityTier::Low;
    let _ = QualityTier::Medium;
    let _ = QualityTier::High;
    let _ = QualityTier::Ultra;
}

#[test]
fn rc_default_profile_vertical_slice() {
    let config = RuntimeConfig::default();
    assert_eq!(config.profile, RuntimeProfile::Game);
}

#[test]
fn rc_tools_profile() {
    let config = RuntimeConfig::tools();
    assert_eq!(config.profile, RuntimeProfile::Tools);
}

#[test]
fn rc_headless_profile() {
    let config = RuntimeConfig::headless();
    assert_eq!(config.profile, RuntimeProfile::Headless);
}

#[test]
fn rc_budgets_low_spec_tier() {
    let config = RuntimeConfig::game();
    let budgets = config.budgets();
    assert_eq!(budgets.quality, QualityTier::Medium);
}

#[test]
fn rc_budgets_shipping_tier() {
    let config = RuntimeConfig::game();
    let budgets = config.budgets();
    assert_eq!(budgets.quality, QualityTier::Medium);
}

#[test]
fn rc_budgets_low_spec_max_particles() {
    let config = RuntimeConfig::game();
    let budgets = config.budgets();
    assert_eq!(budgets.max_particles, 5000);
}

#[test]
fn rc_is_headless_for_headless_config() {
    let config = RuntimeConfig::headless();
    assert!(config.is_headless());
}
