use std::sync::Arc;

use engene::core::runtime_config::{RuntimeConfig, RuntimeProfile};
use engene::runtime::bootstrap::{
    EngineRuntimeAssembly, GameRuntimeAssembly, ToolsRuntimeAssembly,
};
use engene::world::heightmap::Heightmap;
use engene::world::world::WorldGrid;

#[test]
fn game_bootstrap_has_critical_resources_and_systems() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_some());
    assert!(engine
        .resources
        .get::<engene::core::game_config::GameConfig>()
        .is_some());
    assert!(engine
        .resources
        .get::<engene::world::fields::WorldFields>()
        .is_some());
    assert!(engine
        .resources
        .get::<engene::core::material_truth::MaterialTruthService>()
        .is_some());

    let names: Vec<_> = engine.system_descriptors().iter().map(|d| d.name).collect();
    assert!(names.contains(&"Simulation"));
    assert!(names.contains(&"WorldTick"));
    assert!(names.contains(&"AI"));
    assert!(names.contains(&"Physics"));
}

#[test]
fn tools_bootstrap_stays_minimal_and_profiled_tools() {
    let engine = ToolsRuntimeAssembly::minimal();
    let cfg = engine.resources.get::<RuntimeConfig>().unwrap();
    assert_eq!(cfg.profile, RuntimeProfile::Tools);

    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
    assert!(engine
        .resources
        .get::<engene::core::game_config::GameConfig>()
        .is_none());
    assert!(engine.system_descriptors().is_empty());
}

#[test]
fn headless_kernel_bootstrap_stays_kernel_scoped() {
    let engine = EngineRuntimeAssembly::kernel_headless();
    let cfg = engine.resources.get::<RuntimeConfig>().unwrap();
    assert_eq!(cfg.profile, RuntimeProfile::Headless);

    assert!(engine
        .resources
        .get::<engene::core::game_config::GameConfig>()
        .is_none());
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

#[test]
fn app_game_runner_import_surface_stays_narrow() {
    let source = std::fs::read_to_string("src/app/game_runner/mod.rs")
        .expect("must read game runner startup module");
    assert!(source.contains("GameRuntimeAssembly::vertical_slice"));
    assert!(!source.contains("game_resources::"));
    assert!(!source.contains("game_systems::"));
    assert!(!source.contains("game_plugins::"));
}
