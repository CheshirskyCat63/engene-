//! Game runtime runner entrypoint and startup orchestration.

mod app_loop;
mod diagnostics;

use std::sync::Arc;

use app_loop::GameApp;
use winit::event_loop::EventLoop;

use crate::core::build_manifest::BuildManifest;
use crate::core::crash_telemetry;
use crate::runtime::bootstrap::GameRuntimeAssembly;
use crate::world::heightmap::Heightmap;
use crate::world::world::WorldGrid;

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

    let mut app = GameApp::new(engine, heightmap, biomes);
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    println!("[render] starting 3D view — click to capture mouse, ESC to release\n");
    let _ = event_loop.run_app(&mut app);
    println!("\n=== SESSION ENDED ===");
}
