use std::sync::Arc;

use engene::core::runtime_config::{RuntimeConfig, RuntimeProfile};
use engene::runtime::bootstrap::{
    EngineRuntimeAssembly, GameRuntimeAssembly, ToolsRuntimeAssembly,
};

#[test]
fn runtime_profiles_are_explicit_and_non_overlapping() {
    let kernel = EngineRuntimeAssembly::kernel_headless();
    let tools = ToolsRuntimeAssembly::minimal();

    let grid = engene::world::world::WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(engene::world::heightmap::Heightmap::generate(&biomes));
    let game = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

    let kernel_cfg = kernel
        .resources
        .get::<RuntimeConfig>()
        .expect("kernel runtime must have RuntimeConfig");
    let tools_cfg = tools
        .resources
        .get::<RuntimeConfig>()
        .expect("tools runtime must have RuntimeConfig");
    let game_cfg = game
        .resources
        .get::<RuntimeConfig>()
        .expect("game runtime must have RuntimeConfig");

    assert_eq!(kernel_cfg.profile, RuntimeProfile::Headless);
    assert_eq!(tools_cfg.profile, RuntimeProfile::Tools);
    assert_eq!(game_cfg.profile, RuntimeProfile::Game);
}

#[test]
fn role_bootstrap_boundaries_are_enforced() {
    let kernel = EngineRuntimeAssembly::kernel_headless();
    let tools = ToolsRuntimeAssembly::minimal();

    assert!(
        kernel
            .resources
            .get::<engene::core::game_config::GameConfig>()
            .is_none(),
        "kernel runtime must not bootstrap game config"
    );
    assert!(
        tools
            .resources
            .get::<engene::core::game_config::GameConfig>()
            .is_none(),
        "tools runtime must not bootstrap game config"
    );
    assert!(
        tools
            .resources
            .get::<engene::world::resources::ResourceGrid>()
            .is_none(),
        "tools runtime must not bootstrap world resources"
    );
}
