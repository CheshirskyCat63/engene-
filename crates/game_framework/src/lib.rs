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

use engine_startup::{BuildManifest, crash_telemetry, startup_tracing};
use engene::runtime::bootstrap::{EngineRuntimeAssembly, GameRuntimeAssembly};
use engene::world::heightmap::Heightmap;
use engene::world::world::WorldGrid;

pub fn run_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();
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
    let engine = GameRuntimeAssembly::vertical_slice(heightmap.clone(), &biomes);

    let mut app = game_runner::GameApp::new(engine, heightmap, biomes);
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    println!("[render] starting 3D view — click to capture mouse, ESC to release\n");
    let _ = event_loop.run_app(&mut app);
    println!("\n=== SESSION ENDED ===");
}

pub fn run_headless_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

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

    for _ in 0..ticks {
        engine.tick(sim_dt);
    }

    println!(
        "[headless] complete: tick={}, entities={}",
        engine.ecs.tick,
        engine.ecs.alive.len()
    );
}

mod game_runner {
    pub use engene::app::game_runner::GameApp;
}

