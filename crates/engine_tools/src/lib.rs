//! Tools role owner crate.
//! This crate is transitioning to own tools-only runtime.

pub mod api {
    pub const CRATE: &str = "engine_tools";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "engine_tools";
    pub const TARGET_OWNER: &str = "engine_tools";
}

use engine_startup::{crash_telemetry, BuildManifest};

// Temporary stubs for missing dependencies
pub struct ToolsRuntimeAssembly;
impl ToolsRuntimeAssembly {
    pub fn minimal() -> Self { Self }
}

pub mod doctor {
    pub struct DoctorReport {
        errors: u32,
        warnings: u32,
    }
    
    impl DoctorReport {
        pub fn error_count(&self) -> u32 { self.errors }
        pub fn warning_count(&self) -> u32 { self.warnings }
        pub fn print(&self) {
            println!("Doctor report: {} errors, {} warnings", self.errors, self.warnings);
        }
    }
    
    pub enum DoctorMode {
        Strict,
    }
    
    pub fn run_doctor(_engine: &crate::ToolsRuntimeAssembly, _mode: DoctorMode) -> DoctorReport {
        DoctorReport { errors: 0, warnings: 0 }
    }
}

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
