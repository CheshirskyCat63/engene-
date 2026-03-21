//! SDK role owner crate.
//! This crate is transitioning to own SDK runtime and editor startup.
//!
//! ## Ownership Structure
//! - Editor logic: sdk_app::editor (extracted from sdk_runner)  
//! - Runtime phases: engine_runtime::phase (in progress)
//! - Active runtime: transitional, depends on phase extraction
//!
//! ## Current Status
//! - run_from_env_args() is thin adapter - delegates to game_framework for headless, editor pending
//! - editor::update_editor extracted but not fully wired

pub mod editor;

pub mod api {
    pub const CRATE: &str = "sdk_app";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "sdk_app";
    pub const TARGET_OWNER: &str = "sdk_app";
}

/// SDK entrypoint - thin adapter
/// OWNER: sdk_app
/// 
/// Routes to:
/// - Editor mode: pending phase extraction (see docs)
/// - Headless: delegates to game_framework
pub fn run_from_env_args() {
    // For now, SDK editor mode requires phase extraction to complete
    // Thin adapter to existing working paths
    use engine_startup::{install_panic_hook, BuildManifest};
    
    if BuildManifest::handle_version_flag() {
        return;
    }

    install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let manifest = BuildManifest::current();
    println!("=== ENGENE SDK ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();
    
    // Current state: editor mode pending phase extraction
    // Thin adapter routes to game_framework for headless
    println!("[sdk] NOTE: Editor mode pending phase extraction");
    println!("[sdk] For headless: use cargo run -p app_engene_headless");
    println!("[sdk] For game: use cargo run -p app_engene_game");
    println!("\n=== SESSION ENDED ===");
}

/// SDK headless entrypoint - independent implementation
/// OWNER: sdk_app
pub fn run_headless_from_env_args() {
    use engine_startup::{install_panic_hook, BuildManifest};

    if BuildManifest::handle_version_flag() {
        return;
    }

    install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let manifest = BuildManifest::current();
    println!("=== ENGENE SDK HEADLESS ===");
    manifest.print_full();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let ticks: u64 = std::env::args()
        .skip_while(|a| a != "--ticks")
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1200);

    println!("[sdk_headless] kernel tick soak: {} ticks", ticks);

    let mut engine = engine_runtime::assembly::EngineRuntimeAssembly::kernel_headless();
    let sim_dt = 1.0 / 20.0_f32;

    // Stateful resident set for streaming (owner between ticks)
    let mut resident_chunks: Vec<[i32; 2]> = Vec::new();

    for tick in 0..ticks {
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
        for chunk in &streaming_output.chunks_to_unload {
            resident_chunks.retain(|c| c != chunk);
        }
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
    }

    let ecs_tick = engine.ecs().tick;
    let alive_count = engine.ecs().alive.len();

    println!(
        "[sdk_headless] complete: tick={}, entities={}",
        ecs_tick,
        alive_count
    );
}
