//! Kernel/headless runtime runner shared by headless entrypoints.

use crate::core::build_manifest::BuildManifest;
use crate::core::crash_telemetry;
use crate::runtime::bootstrap::EngineRuntimeAssembly;

pub fn run_from_env_args() {
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
