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
        
        // Phase 2: Streaming phase (stateful) - ENHANCED
        let streaming_config = engine_world::StreamingConfig::default();
        let mut streaming_owner = engine_world::StreamingOwner::new(streaming_config.clone());
        
        // Initialize with some chunks
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 0, z: 0 });
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 1, z: 0 });
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 2, z: 0 });
        
        let mut resident_chunks: Vec<[i32; 2]> = streaming_owner.get_loaded_chunks()
            .into_iter()
            .map(|coord| [coord.x, coord.z])
            .collect();
        
        let persistence_input = engine_runtime::phase::persistence_enhanced::PersistenceEnhancedInput::from_context(
            tick,
            streaming_owner,
            "test_saves".to_string(),
            1,
        );
        
        let _persistence_result = engine_runtime::phase::run_persistence_enhanced(persistence_input);
        
        let streaming_output = engine_runtime::phase::run_streaming_enhanced(
            engine_runtime::phase::streaming_enhanced::StreamingEnhancedInput {
                tick,
                player_position: Some([0.0, 0.0, 0.0]), // Headless with anchor at origin
                view_distance_chunks: 4, // Smaller view for headless
                pending_unload_count: 2,
                residency_budget: 8,
                known_loaded_chunks: resident_chunks.clone(), // Pass current resident set
                streaming_config: streaming_config,
            }
        );
        
        // Update resident set based on streaming decisions
        // Remove unloaded chunks first
        for chunk_to_unload in &streaming_output.chunks_to_unload {
            resident_chunks.retain(|&chunk| chunk != *chunk_to_unload);
        }
        
        // Then add newly loaded chunks (avoid duplicates)
        for chunk_to_load in &streaming_output.chunks_to_load {
            if !resident_chunks.contains(chunk_to_load) {
                resident_chunks.push(*chunk_to_load);
            }
        }
        
        // Additional phases will be added here in canonical order
        // Phase 3: Persistence phase
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
