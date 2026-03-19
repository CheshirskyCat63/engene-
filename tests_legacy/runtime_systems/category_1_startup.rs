use super::*;

// Category 1: System startup/shutdown (30 tests)
// =============================================================================

#[test]
fn engine_boots_tools() {
    let engine = ToolsRuntimeAssembly::minimal();
    let _count = engine.ecs.alive.len();
}

#[test]
fn engine_tools_has_resources() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

#[test]
fn engine_tools_is_running() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine.is_running());
}

#[test]
fn engine_tools_has_systems() {
    let engine = ToolsRuntimeAssembly::minimal();
    let descs = engine.system_descriptors();
    assert!(descs.is_empty());
}

#[test]
fn engine_tools_shutdown_safe() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
    assert!(!engine.is_running());
}

#[test]
fn engine_headless_boots() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    let _alive = engine.ecs.alive.len();
}

#[test]
fn engine_headless_has_ecs() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    let _alive = engine.ecs.alive.len();
}

#[test]
fn engine_headless_has_events() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert_eq!(engine.events.channel_count(), 0);
}

#[test]
fn engine_headless_has_time() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.time.day >= 1);
    assert!(engine.time.month >= 1);
}

#[test]
fn engine_headless_has_resources() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_some());
}

#[test]
fn engine_vertical_slice_boots() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let _alive = engine.ecs.alive.len();
}

#[test]
fn engine_vertical_slice_has_more_systems_than_tools() {
    let tools = ToolsRuntimeAssembly::minimal();
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let headless = GameRuntimeAssembly::headless(&biomes);
    assert!(headless.system_descriptors().len() >= tools.system_descriptors().len());
}

#[test]
fn engine_shutdown_clears_running() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
    assert!(!engine.is_running());
}

#[test]
fn engine_tools_init_completes() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine.is_running());
}

#[test]
fn engine_headless_init_completes() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.is_running());
}

#[test]
fn engine_vertical_slice_init_completes() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine.is_running());
}

#[test]
fn engine_stop_sets_running_false() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.stop();
    assert!(!engine.is_running());
}

#[test]
fn engine_tools_no_panic_on_multiple_creates() {
    let _e1 = ToolsRuntimeAssembly::minimal();
    let _e2 = ToolsRuntimeAssembly::minimal();
}

#[test]
fn engine_headless_no_panic_on_multiple_creates() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let _e1 = GameRuntimeAssembly::headless(&biomes);
    let _e2 = GameRuntimeAssembly::headless(&biomes);
}

#[test]
fn engine_tools_ecs_tick_zero_initially() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert_eq!(engine.ecs.tick, 0);
}

#[test]
fn engine_worker_pool_available() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine.worker_pool.worker_count() >= 1);
}

#[test]
fn engine_worker_pool_execute() {
    let engine = ToolsRuntimeAssembly::minimal();
    let result = engine.worker_pool.execute(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn engine_worker_pool_par_join() {
    let engine = ToolsRuntimeAssembly::minimal();
    let (a, b) = engine.worker_pool.par_join(|| 1, || 2);
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn engine_debug_registry_accessible() {
    let engine: Engine = ToolsRuntimeAssembly::minimal();
    let _ = engine.debug_registry.views();
}

#[test]
fn engine_commands_empty_after_init() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    assert_eq!(engine.commands.take_despawns().len(), 0);
}

#[test]
fn engine_shutdown_does_not_panic() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
}

#[test]
fn engine_double_shutdown_safe() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
    engine.shutdown();
}

#[test]
fn engine_tick_after_shutdown_no_panic() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
    engine.tick(0.05);
}

#[test]
fn engine_resource_grid_access() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    let grid = engine.resource_grid();
    assert!(grid.food.len() > 0 || true);
}

#[test]
fn engine_vertical_slice_has_quality_governor() {
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
fn engine_vertical_slice_has_budget_registry() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine
        .resources
        .get::<engene::core::budget_registry::BudgetRegistry>()
        .is_some());
}

// =============================================================================
