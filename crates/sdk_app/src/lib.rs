//! SDK role owner crate.
//! This crate is transitioning to own SDK runtime and editor startup.
//!
//! ## Ownership Structure
//! - Editor logic: sdk_app::editor (extracted from sdk_runner)  
//! - Runtime phases: engine_runtime::phase (in progress)
//! - Active runtime: transitional, depends on phase extraction
//!
//! ## Current Status
//! - run_from_env_args() is thin adapter - editor pending, headless redirects to engene_run
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
/// - Headless: redirects to engene_run
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
    
    println!("[sdk] NOTE: Editor mode pending phase extraction");
    println!("[sdk] For headless: use cargo run -p engene_run");
    println!("\n=== SESSION ENDED ===");
}
