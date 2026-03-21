use engine_startup::{install_panic_hook, BuildManifest};
use engine_runtime::assembly::EngineRuntimeAssembly;
use engine_runtime::phase::{self, StreamingInput, PersistenceInput};

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
    
    let mut resident_chunks: Vec<[i32; 2]> = Vec::new();

    for tick in 0..ticks {
        let _tick_result = phase::tick::run_tick(tick, sim_dt);
        
        let streaming_output = phase::run_streaming(
            StreamingInput {
                tick,
                player_position: Some([0.0, 0.0, 0.0]),
                view_distance_chunks: 4,
                pending_unload_count: 2,
                residency_budget: 16,
                known_loaded_chunks: resident_chunks.clone(),
            }
        );
        
        for chunk in &streaming_output.chunks_to_unload {
            resident_chunks.retain(|c| c != chunk);
        }
        for chunk in &streaming_output.chunks_to_load {
            if !resident_chunks.contains(chunk) {
                resident_chunks.push(*chunk);
            }
        }
        
        let _persistence_result = phase::run_persistence(
            PersistenceInput {
                tick,
                completed_loads: streaming_output.chunks_to_load.clone(),
                completed_unloads: streaming_output.chunks_to_unload.clone(),
                dirty_chunk_count: 0,
                save_enabled: true,
            }
        );
    }

    let ecs_tick = engine.ecs().tick;
    let alive_count = engine.ecs().alive.len();

    println!(
        "[headless] complete: tick={}, entities={}",
        ecs_tick,
        alive_count
    );
}