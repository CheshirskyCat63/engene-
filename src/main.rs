//! ENGENE — Thin dispatcher.
//!
//! The main entry point delegates to the game runtime.
//! For the full editor, use: cargo run --bin engene_sdk
//! For headless testing, use: cargo run --bin engene_headless -- --months 12

use std::sync::Arc;
use winit::event_loop::EventLoop;

type ArcHeightmap = Arc<engene::world::heightmap::Heightmap>;

fn main() {
    engene::core::crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    let manifest = engene::core::build_manifest::BuildManifest::current();
    println!("=== ENGENE ===");
    manifest.print_full();
    println!("  Tip: use 'cargo run --bin engene_sdk' for the editor");
    println!("  Tip: use 'cargo run --bin engene_headless -- --months 12' for headless sim\n");
    engene::core::build_manifest::BuildManifest::ensure_data_dirs();

    let grid = engene::world::world::WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap: ArcHeightmap = Arc::new(engene::world::heightmap::Heightmap::generate(&biomes));

    let engine = engene::app::runtime_assembly::RuntimeAssembly::vertical_slice(
        heightmap.clone(),
        &biomes,
    );

    let doctor_report = engene::tools::doctor::run_doctor(
        &engine,
        engene::tools::doctor::DoctorMode::Advisory,
    );
    println!(
        "[doctor] startup: {} errors, {} warnings",
        doctor_report.error_count(),
        doctor_report.warning_count()
    );
    doctor_report.print();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = engene::app::game_app::GameApp {
        window: None,
        renderer: None,
        engine,
        camera: engene::graphics::camera::FlyCamera::new(),
        input: engene::input::input::InputState::new(),
        last_frame: None,
        frame: 0,
        sim_accum: 0.0,
        last_report_month: 0,
        heightmap,
        biomes,
    };

    println!("[render] starting 3D view — click to capture mouse, ESC to release\n");
    let _ = event_loop.run_app(&mut app);
    println!("\n=== SESSION ENDED ===");
}

