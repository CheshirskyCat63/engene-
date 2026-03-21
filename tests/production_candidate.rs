//! Phase 10: AAA Closeout / Production Candidate
//! Final acceptance test suite that validates everything works together.

#[test]
fn production_build_manifest() {
    use engene::core::build_manifest::BuildManifest;
    let manifest = BuildManifest::current();
    assert!(!manifest.engine_version.is_empty());
    assert!(!manifest.profile.is_empty());
}

#[test]
fn production_schema_compatibility() {
    use engene::core::build_manifest::{SaveCompatibility, SCHEMA_VERSION_SAVE};
    let compat = SaveCompatibility::current();
    assert!(compat.is_compatible(SCHEMA_VERSION_SAVE));
    assert!(!compat.is_compatible(SCHEMA_VERSION_SAVE + 100));
}

#[test]
fn production_doctor_no_errors() {
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::tools::doctor;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

    let report = doctor::run_doctor(&engine, doctor::DoctorMode::Advisory);
    assert_eq!(
        report.error_count(),
        0,
        "production candidate: 0 doctor errors required"
    );
}

#[test]
fn production_runtime_truth_json() {
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::tools::doctor;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

    let json = doctor::generate_runtime_truth_json(&engine);
    assert!(json.contains("engine_version"));
    assert!(json.contains("entity_count"));
    assert!(json.contains("doctor"));
}

#[test]
fn production_world_layout_skeleton() {
    use engene::world::authored_sets::WorldLayout;
    let layout = WorldLayout::vertical_slice_skeleton();
    assert!(!layout.camps.is_empty());
    assert!(!layout.habitats.is_empty());
    assert!(!layout.roads.is_empty());
}

#[test]
fn production_item_registry_coverage() {
    use engene::game::economy::item_registry::{ItemCategory, ItemRegistry};
    let registry = ItemRegistry::new();
    assert!(registry.by_category(ItemCategory::Medkit).len() >= 2);
    assert!(registry.by_category(ItemCategory::Food).len() >= 2);
    assert!(registry.by_category(ItemCategory::Ammo).len() >= 2);
    assert!(registry.by_category(ItemCategory::Artifact).len() >= 2);
}

#[test]
fn production_animation_clip_map() {
    use engene::animation::clip_map::{AnimationState, ClipMap};
    let map = ClipMap::new();
    assert!(map.clip_for_state(&AnimationState::Idle).is_some());
    assert!(map.clip_for_state(&AnimationState::Walk).is_some());
    assert!(map.clip_for_state(&AnimationState::Run).is_some());
    assert!(map.clip_for_state(&AnimationState::DeathFall).is_some());
    assert!(map.clip_count() >= 15);
}

#[test]
fn production_sound_bank_events() {
    use engene::audio::sound_bank::{SoundBank, SoundEvent};
    let bank = SoundBank::new();
    assert!(bank.get(&SoundEvent::FootstepDirt).is_some());
    assert!(bank.get(&SoundEvent::AmbientWind).is_some());
    assert!(bank.get(&SoundEvent::WeaponFirePistol).is_some());
    assert!(bank.event_count() >= 20);
}

#[test]
fn production_camp_simulation() {
    use engene::simulation::camp_simulation::CampState;
    let mut camp = CampState::new("Test Camp", "Loners", 5);
    assert!(camp.is_safe());

    camp.tick_daily();
    assert!(camp.mood > 0.0);

    camp.report_danger(0.8);
    assert!(!camp.is_safe());
    assert!(camp.pressure_level() > 0.0);
}

#[test]
fn production_body_response_cache() {
    use engene::body::body_response::{BodyPhysicalResponseCache, PhysicalResponseTier};

    let healthy = BodyPhysicalResponseCache::healthy();
    assert_eq!(healthy.response_tier, PhysicalResponseTier::Healthy);
    assert!(healthy.is_combat_capable());
    assert!(healthy.is_mobile());

    let injured = BodyPhysicalResponseCache::compute_from_health(0.3, 0.5, 0.7);
    assert_eq!(injured.response_tier, PhysicalResponseTier::MajorInjury);
    assert!(injured.movement_speed_mult < 1.0);

    let dead = BodyPhysicalResponseCache::compute_from_health(0.0, 1.0, 1.0);
    assert_eq!(dead.response_tier, PhysicalResponseTier::Dead);
    assert!(!dead.is_combat_capable());
    assert!(!dead.is_mobile());
}

#[test]
fn production_render_validation() {
    use engene::graphics::render_validation::RenderValidationResult;
    let result = RenderValidationResult::validate_pipeline();
    assert!(result.pass_count() > 0);
    assert!(
        result.completion_pct() > 50.0,
        "render pipeline should be >50% complete"
    );
}

#[test]
fn production_milestone_tracker() {
    use engene::simulation::world_milestones::WorldMilestoneTracker;
    let mut tracker = WorldMilestoneTracker::new();
    for _ in 0..5 {
        tracker.record_bankruptcy();
    }
    tracker.check_milestones(3);
    assert!(!tracker.milestones_achieved.is_empty());
}

#[test]
fn production_perf_budget() {
    use engene::core::perf::perf_budget::PerfBudgetManager;
    let mut mgr = PerfBudgetManager::new(60);
    mgr.record("AI", 4000);
    mgr.record("Physics", 2000);
    let ai = mgr.get("AI").unwrap();
    assert!(ai.last_measured_us == 4000);
    assert!(mgr.total_utilization() > 0.0);
}
