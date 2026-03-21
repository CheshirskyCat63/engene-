//! SDK role owner crate.
//! This crate is transitioning to own SDK runtime and editor startup.
//!
//! ## Ownership Structure
//! - Editor logic: sdk_app::editor (extracted from sdk_runner)  
//! - Runtime phases: engine_runtime::phase (in progress)
//! - Active runtime: transitional, depends on phase extraction
//!
//! ## Current Status
//! - run_from_env_args() is STUB pending phase extraction
//! - editor::update_editor extracted but not wired
//! - This is transitional debt

pub mod editor;

pub mod api {
    pub const CRATE: &str = "sdk_app";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "sdk_app";
    pub const TARGET_OWNER: &str = "sdk_app";
}

/// SDK entrypoint - STUB, needs implementation
/// OWNER: sdk_app
/// 
/// TODO: Connect to real runtime after phase extraction completes
pub fn run_from_env_args() {
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
    
    // STUB: Real runtime path not yet connected
    println!("[sdk] ERROR: run_from_env_args() is stub");
    println!("[sdk] Runtime implementation pending phase extraction");
    println!("[sdk] See: docs/canonical/SDK_RUNNER_OWNERSHIP_AUDIT.md");
    println!("\n=== SESSION ENDED ===");
}

/// SDK headless entrypoint - STUB
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
    println!();
    
    // STUB
    println!("[sdk] ERROR: run_headless_from_env_args() is stub");
    println!("\n=== SESSION ENDED ===");
}
