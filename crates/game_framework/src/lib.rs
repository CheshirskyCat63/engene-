//! Game role owner crate.
//! This crate is transitioning to own game launch and headless behavior.

pub mod api {
    pub const CRATE: &str = "game_framework";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "game_framework";
    pub const TARGET_OWNER: &str = "game_framework";
}

use std::sync::Arc;

use winit::event_loop::EventLoop;

// Temporary stubs for missing engine modules
pub struct EngineRuntimeAssembly;
impl EngineRuntimeAssembly {
    pub fn kernel_headless() -> Self { Self }
    pub fn tick(&mut self, _dt: f32) {}
}
pub struct EngineEcs {
    pub tick: u32,
    pub alive: Vec<u32>,
}
impl EngineRuntimeAssembly {
    pub fn ecs(&mut self) -> &mut EngineEcs {
        static mut ECS: EngineEcs = EngineEcs { tick: 0, alive: vec![] };
        unsafe { &mut ECS }
    }
}
pub struct GameRuntimeAssembly;
pub struct WorldGrid {
    pub cells: Vec<WorldCell>,
}
impl WorldGrid {
    pub fn generate() -> Self { Self { cells: vec![] } }
}
pub struct WorldCell {
    pub biome: Biome,
}
pub struct Biome;
impl Clone for Biome {
    fn clone(&self) -> Self { *self }
}
impl Copy for Biome {}
pub struct Heightmap;
impl Heightmap {
    pub fn generate(_biomes: &[Biome]) -> Self { Self }
}

use engine_startup::{install_panic_hook, BuildManifest};

pub fn run_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    install_panic_hook();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    let manifest = BuildManifest::current();
    println!("=== ENGENE GAME ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly; // TODO: implement proper assembly

    // TODO: Fix game_runner import
    // let mut app = game_runner::GameApp::new(engine, heightmap, biomes);
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    println!("[render] starting 3D view — click to capture mouse, ESC to release\n");
    // TODO: Fix run_app method
    // let _ = event_loop.run_app(&mut app);
    println!("Game runner integration needed");
    println!("\n=== SESSION ENDED ===");
}

pub fn run_headless_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let manifest = BuildManifest::current();
    println!("=== ENGENE HEADLESS (KERNEL) ===");
    manifest.print_full();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let ticks: u64 = std::env::args()
        .skip_while(|a| a != "--ticks")
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1200);

    println!("[headless] kernel tick soak: {} ticks", ticks);

    let mut engine = EngineRuntimeAssembly::kernel_headless();
    let sim_dt = 1.0 / 20.0_f32;

    for tick in 0..ticks {
        // ========================================================================
        // TEMPORARY DOUBLE-RUN: Transitional debt
        // NEW: engine_runtime::phase::tick::run_tick() - new phase owner
        // OLD: engine.tick() - old orchestration path
        // 
        // This duplication is TEMPORARY. After tick extraction is complete,
        // old path will be removed and only run_tick() will remain.
        // See: docs/canonical/CURRENT_RUNTIME_TRUTH.md
        // ========================================================================
        
        // NEW: Call new phase entrypoint (engine_runtime owns tick now)
        let _phase_result = engine_runtime::phase::tick::run_tick(tick, sim_dt);
        
        // OLD: Legacy path - will be deprecated after full extraction
        engine.tick(sim_dt);
    }

    let ecs_tick = engine.ecs().tick;
    let alive_count = engine.ecs().alive.len();

    println!(
        "[headless] complete: tick={}, entities={}",
        ecs_tick,
        alive_count
    );
}

// TODO: Implement proper game runner
mod game_runner {
    pub struct GameApp;
}
