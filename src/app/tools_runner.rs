//! Tools doctor runner for the ENGENE tools-only runtime.

use crate::core::build_manifest::BuildManifest;
use crate::core::crash_telemetry;
use crate::runtime::bootstrap::ToolsRuntimeAssembly;
use crate::tools::doctor;

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
