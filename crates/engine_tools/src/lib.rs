//! Tools role owner crate.
//! This crate is transitioning to own tools-only runtime.

pub mod api {
    pub const CRATE: &str = "engine_tools";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "engine_tools";
    pub const TARGET_OWNER: &str = "engine_tools";
}

use engene::runtime::bootstrap::ToolsRuntimeAssembly;
use engene::tools::doctor;
use engine_startup::{crash_telemetry, BuildManifest};

pub fn run_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    crash_telemetry::install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let manifest = BuildManifest::current();
    println!("=== ENGENE TOOLS ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    let engine = ToolsRuntimeAssembly::minimal();
    let report = doctor::run_doctor(&engine, doctor::DoctorMode::Strict);

    println!(
        "[doctor] tools runtime: {} errors, {} warnings",
        report.error_count(),
        report.warning_count()
    );
    report.print();
}
