use super::*;

// Category 3: Resource registration (25 tests)
// =============================================================================

#[test]
fn tools_has_resource_grid() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

#[test]
fn headless_has_resource_grid() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_some());
}

#[test]
fn headless_has_heightmap() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<Arc<Heightmap>>().is_some());
}

#[test]
fn headless_has_world_streamer() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::world::streaming::WorldStreamer>()
        .is_some());
}

#[test]
fn headless_has_camp_states() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<Vec<CampState>>().is_some());
}

#[test]
fn headless_has_item_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::game::economy::item_registry::ItemRegistry>()
        .is_some());
}

#[test]
fn headless_has_milestone_tracker() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.resources.get::<WorldMilestoneTracker>().is_some());
}

#[test]
fn vertical_slice_has_quality_governor() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::core::quality_governor::QualityGovernor>()
        .is_some());
}

#[test]
fn vertical_slice_has_budget_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::core::budget_registry::BudgetRegistry>()
        .is_some());
}

#[test]
fn vertical_slice_has_sim_bus() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::core::events::sim_bus::SimBus>()
        .is_some());
}

#[test]
fn vertical_slice_has_terrain_truth() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::world::terrain_truth::TerrainTruth>()
        .is_some());
}

#[test]
fn vertical_slice_has_surface_state_store() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::world::surface_state::SurfaceStateStore>()
        .is_some());
}

#[test]
fn assemblies_have_canonical_material_truth_and_core_resources() {
    use engene::core::game_config::GameConfig;
    use engene::core::material_truth::MaterialTruthService;
    use engene::physics::ballistics::BallisticsSystem;
    use engene::world::fields::WorldFields;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));

    let vertical = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let headless = GameRuntimeAssembly::headless(&biomes);
    let tools = ToolsRuntimeAssembly::minimal();

    for engine in [&vertical, &headless] {
        assert!(
            engine.resources.get::<MaterialTruthService>().is_some(),
            "MaterialTruthService must be present in game assembly modes"
        );
        assert!(
            engine.resources.get::<GameConfig>().is_some(),
            "GameConfig must be explicitly present in game assembly modes"
        );
        assert!(
            engine.resources.get::<WorldFields>().is_some(),
            "WorldFields must be present in game assembly modes"
        );
        assert!(
            engine.resources.get::<BallisticsSystem>().is_some(),
            "BallisticsSystem must be present in game assembly modes"
        );
        assert!(
            engine
                .resources
                .get::<engene::world::surface_state::SurfaceStateStore>()
                .is_some(),
            "SurfaceStateStore must be present in game assembly modes"
        );
    }
    assert!(tools.resources.get::<MaterialTruthService>().is_none());
    assert!(tools.resources.get::<GameConfig>().is_none());
    assert!(tools.resources.get::<WorldFields>().is_none());
    assert!(tools.resources.get::<BallisticsSystem>().is_none());
    assert!(tools
        .resources
        .get::<engene::world::surface_state::SurfaceStateStore>()
        .is_none());

    let vertical_truth = vertical.resources.get::<MaterialTruthService>().unwrap();
    let headless_truth = headless.resources.get::<MaterialTruthService>().unwrap();
    let cfg = GameConfig::load_from_dir("game/data");
    let expected_audio0 = cfg
        .material_bridge
        .audio
        .iter()
        .find(|m| m.material_id == 0)
        .expect("material bridge must have id 0 mapping")
        .impact_sound_class
        .clone();
    assert_eq!(
        vertical_truth.query_audio(0).impact_sound_class,
        expected_audio0
    );
    assert_eq!(
        headless_truth.query_audio(0).impact_sound_class,
        expected_audio0
    );
}

#[test]
fn ballistics_tick_uses_runtime_world_fields_not_local_defaults() {
    use engene::core::events::canonical::ImpactEvent;
    use engene::physics::ballistics::BallisticsSystem;
    use engene::world::fields::WorldFields;
    use glam::Vec3;

    fn run_with_wind(dir: Vec3) -> f32 {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let mut engine = GameRuntimeAssembly::headless(&biomes);
        {
            let fields = engine
                .resources
                .get_mut::<WorldFields>()
                .expect("headless must have WorldFields");
            fields.wind.base_dir = dir.normalize();
            fields.wind.strength = 800.0;
            fields.wind.turbulence = 0.0;
            fields.wind.curl_turbulence = 0.0;
            fields.wind.storm_wind = Vec3::ZERO;
        }
        {
            let bal = engine
                .resources
                .get_mut::<BallisticsSystem>()
                .expect("headless must have BallisticsSystem");
            bal.fire(
                Vec3::new(0.0, 400.0, 0.0),
                Vec3::X,
                120.0,
                0.01,
                0.2,
                10,
                0,
                0,
            );
        }

        engine.tick(1.0 / 20.0);
        let bal = engine.resources.get::<BallisticsSystem>().unwrap();
        if let Some(p) = bal.projectiles.first() {
            p.pos.x
        } else {
            let impacts = engine.events.read::<ImpactEvent>();
            impacts
                .first()
                .map(|ev| ev.position.x)
                .expect("projectile should produce either active state or impact evidence")
        }
    }

    let with_pos_wind = run_with_wind(Vec3::X);
    let with_neg_wind = run_with_wind(-Vec3::X);
    assert!(
        (with_pos_wind - with_neg_wind).abs() > 0.1,
        "ballistics tick must consume runtime WorldFields; opposite wind should change trajectory"
    );
}

#[test]
fn canonical_weather_drives_wetness_through_owner_path() {
    use engene::world::fields::WorldFields;
    use engene::world::surface_state::SurfaceStateStore;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    {
        let fields = engine
            .resources
            .get_mut::<WorldFields>()
            .expect("headless must have WorldFields");
        fields.rain.set_rain(5.0, 1.0);
        fields.wind.strength = 10.0;
        fields.wind.turbulence = 0.0;
        fields.wind.curl_turbulence = 0.0;
    }
    engine.tick(1.0 / 20.0);
    let wet_after_rain = engine
        .resources
        .get::<SurfaceStateStore>()
        .and_then(|s| s.get(0, 0).map(|p| p.wetness_mask))
        .expect("wetness patch should be created by canonical rain route");
    assert!(wet_after_rain > 0.0);

    {
        let fields = engine.resources.get_mut::<WorldFields>().unwrap();
        fields.rain.set_rain(0.0, 0.0);
    }
    engine.tick(1.0 / 20.0);
    let wet_after_first_water_tick = engine
        .resources
        .get::<SurfaceStateStore>()
        .and_then(|s| s.get(0, 0).map(|p| p.wetness_mask))
        .expect("wetness patch should remain owner-managed");
    engine.tick(1.0 / 20.0);
    let wet_after_second_water_tick = engine
        .resources
        .get::<SurfaceStateStore>()
        .and_then(|s| s.get(0, 0).map(|p| p.wetness_mask))
        .expect("wetness patch should remain owner-managed");
    assert!(
        wet_after_first_water_tick > wet_after_rain,
        "water->wetness owner path should increase wetness after rain source is disabled"
    );
    assert!(
        wet_after_second_water_tick > wet_after_first_water_tick,
        "water->wetness owner path should continue producing wetness across subsequent ticks"
    );
}

#[test]
fn doctor_reports_canonical_wiring_invariants() {
    use engene::tools::doctor::{run_doctor, DoctorMode};

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    let report = run_doctor(&engine, DoctorMode::Advisory);
    let wiring_msgs: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.category == "canonical_wiring")
        .map(|d| d.message.clone())
        .collect();
    assert!(
        wiring_msgs.iter().any(|m| m.contains("WorldFields wired")),
        "doctor should assert canonical world-field dependency wiring"
    );
    assert!(
        wiring_msgs
            .iter()
            .any(|m| m.contains("MaterialTruthService canonical path active")),
        "doctor should assert material truth is not fallback-primary"
    );
}

#[test]
fn doctor_reports_missing_canonical_world_fields_dependency() {
    use engene::tools::doctor::{run_doctor, DoctorMode};
    use engene::world::fields::WorldFields;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    let _removed = engine.resources.take::<WorldFields>();
    let report = run_doctor(&engine, DoctorMode::Advisory);
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.category == "canonical_wiring" && d.message.contains("WorldFields is missing")));
}

#[test]
fn engine_kernel_assembly_does_not_boot_game_content_by_default() {
    use engene::core::game_config::GameConfig;
    use engene::core::material_truth::MaterialTruthService;
    use engene::physics::ballistics::BallisticsSystem;
    use engene::runtime::bootstrap::EngineRuntimeAssembly;
    use engene::world::world::WorldGrid;

    let engine = EngineRuntimeAssembly::kernel_headless();
    assert!(engine.resources.get::<GameConfig>().is_none());
    assert!(engine.resources.get::<MaterialTruthService>().is_none());
    assert!(engine.resources.get::<BallisticsSystem>().is_none());
    assert!(engine.resources.get::<WorldGrid>().is_none());
}

#[test]
fn tools_runtime_does_not_autoload_legacy_demo_chunk_content() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert_eq!(
        engine.ecs.alive.len(),
        0,
        "tools runtime must not auto-bootstrap legacy authored demo chunk content"
    );
}

#[test]
fn strict_doctor_passes_for_minimal_tools_role() {
    use engene::tools::doctor::{run_doctor, DoctorMode};

    let engine = ToolsRuntimeAssembly::minimal();
    let report = run_doctor(&engine, DoctorMode::Strict);
    assert_eq!(report.warning_count(), 0);
    assert_eq!(report.error_count(), 0);
}

#[test]
fn strict_doctor_fails_when_tools_canonical_dependency_is_removed() {
    use engene::tools::doctor::{run_doctor, DoctorMode};
    use engene::world::fields::WorldFields;

    let result = std::panic::catch_unwind(|| {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let mut engine = GameRuntimeAssembly::headless(&biomes);
        let _removed = engine.resources.take::<WorldFields>();
        let _ = run_doctor(&engine, DoctorMode::Strict);
    });
    assert!(
        result.is_err(),
        "strict doctor must fail when canonical world-field dependency is missing"
    );
}

#[test]
fn doctor_plugin_alignment_is_role_aware_between_tools_and_game_runtime() {
    use engene::tools::doctor::{run_doctor, DoctorMode};

    let tools = ToolsRuntimeAssembly::minimal();
    let tools_report = run_doctor(&tools, DoctorMode::Advisory);
    assert!(tools_report
        .diagnostics
        .iter()
        .any(|d| d.category == "plugin_alignment" && d.message.contains("0 expected")));

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let vertical = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let vertical_report = run_doctor(&vertical, DoctorMode::Advisory);
    assert!(vertical_report
        .diagnostics
        .iter()
        .any(|d| d.category == "plugin_alignment" && d.message.contains("14 expected")));
}

#[test]
fn vertical_slice_has_prefab_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::content::prefabs::prefab_registry::PrefabRegistry>()
        .is_some());
}

#[test]
fn vertical_slice_has_faction_relations() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::game::gameplay::factions::FactionRelations>()
        .is_some());
}

#[test]
fn resource_grid_from_biomes() {
    let biomes = vec![Biome::Forest, Biome::Plains];
    let grid = engene::world::resources::ResourceGrid::new(&biomes);
    assert!(grid.food.len() > 0);
}

#[test]
fn item_registry_defaults() {
    let reg = engene::game::economy::item_registry::ItemRegistry::new();
    let medkit = reg.get("medkit");
    assert!(medkit.is_some());
}

#[test]
fn item_registry_by_category() {
    use engene::game::economy::item_registry::ItemCategory;
    let reg = engene::game::economy::item_registry::ItemRegistry::new();
    let medkits = reg.by_category(ItemCategory::Medkit);
    assert!(!medkits.is_empty());
}

#[test]
fn no_duplicate_resource_grid() {
    let engine = ToolsRuntimeAssembly::minimal();
    let r1 = engine
        .resources
        .get::<engene::world::resources::ResourceGrid>();
    assert!(r1.is_none());
}

#[test]
fn headless_has_destruction_system() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::physics::destruction::DestructionSystem>()
        .is_some());
}

#[test]
fn headless_has_terrain_deformation() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::world::terrain_deformation::TerrainDeformationSystem>()
        .is_some());
}

#[test]
fn vertical_slice_has_clip_map() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::animation::clip_map::ClipMap>()
        .is_some());
}

#[test]
fn vertical_slice_has_schema_migration_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::core::build_manifest::SchemaMigrationRegistry>()
        .is_some());
}

#[test]
fn tools_minimal_resources() {
    let engine = ToolsRuntimeAssembly::minimal();
    let count = engine.resources.type_ids().len();
    assert!(count >= 1);
}

// =============================================================================
