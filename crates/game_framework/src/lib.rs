//! Game role owner crate.
//! This crate provides game launch and headless behavior composition.

pub mod api {
    pub const CRATE: &str = "game_framework";
    pub const STATUS: &str = "game_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "game_framework";
    pub const TARGET_OWNER: &str = "game_framework";
}

use engine_startup::{install_panic_hook, BuildManifest};

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

    let mut engine = engine_runtime::EngineRuntimeAssembly::kernel_headless();
    let sim_dt = 1.0 / 20.0_f32;
    
    // Stateful resident set for streaming (owner between ticks)
    let mut resident_chunks: Vec<[i32; 2]> = Vec::new();

    for tick in 0..ticks {
        // ========================================================================
        // PHASE EXECUTION: Canonical order
        // engine_runtime now owns all phase execution
        // ========================================================================
        
        // Phase 1: Tick phase
        let _tick_result = engine_runtime::phase::tick::run_tick(tick, sim_dt);
        
        // Phase 2: Streaming phase - stateful loop with known_loaded_chunks
        let streaming_output = engine_runtime::phase::run_streaming(
            engine_runtime::phase::StreamingInput {
                tick,
                player_position: Some([0.0, 0.0, 0.0]),
                view_distance_chunks: 4,
                pending_unload_count: 2,
                residency_budget: 16,
                known_loaded_chunks: resident_chunks.clone(),
            }
        );
        
        // Update resident_chunks from streaming decisions
        // Remove unloaded chunks
        for chunk in &streaming_output.chunks_to_unload {
            resident_chunks.retain(|c| c != chunk);
        }
        // Add newly loaded chunks
        for chunk in &streaming_output.chunks_to_load {
            if !resident_chunks.contains(chunk) {
                resident_chunks.push(*chunk);
            }
        }
        
        // Phase 3: Persistence phase - report completed transitions
        let _persistence_result = engine_runtime::phase::run_persistence(
            engine_runtime::phase::PersistenceInput {
                tick,
                completed_loads: streaming_output.chunks_to_load.clone(),
                completed_unloads: streaming_output.chunks_to_unload.clone(),
                dirty_chunk_count: 0,
                save_enabled: true,
            }
        );
        
        // Phase 4-7: Additional phases will be added here in canonical order
        // Phase 4: Spatial phase
        // Phase 5: Audio phase
        // Phase 6: Editor phase  
        // Phase 7: Render phase
    }

    let ecs_tick = engine.ecs().tick;
    let alive_count = engine.ecs().alive.len();

    println!(
        "[headless] complete: tick={}, entities={}",
        ecs_tick,
        alive_count
    );
}
