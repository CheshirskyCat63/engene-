//! SDK role owner crate.
//! This crate is transitioning to own SDK runtime and editor startup.

pub mod api {
    pub const CRATE: &str = "sdk_app";
    pub const STATUS: &str = "transitional_role_owner";
    pub const CURRENT_OPERATOR_OWNER: &str = "sdk_app";
    pub const TARGET_OWNER: &str = "sdk_app";
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};

use engine_startup::{install_panic_hook, BuildManifest};

// Minimal stubs for compilation
pub struct SpatialDirtyJournal {
    pub force_rebuild: bool,
}
impl Default for SpatialDirtyJournal {
    fn default() -> Self {
        Self { force_rebuild: false }
    }
}

pub fn run_from_env_args() {
    if BuildManifest::handle_version_flag() {
        return;
    }

    install_panic_hook();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    puffin::set_scopes_on(true);

    let manifest = BuildManifest::current();
    println!("=== ENGENE SDK ===");
    manifest.print_full();
    println!();
    BuildManifest::ensure_data_dirs();
    manifest.write_manifest_json();

    println!("SDK app needs proper implementation");
    println!("\n=== SESSION ENDED ===");
}

pub fn run_headless_from_env_args() {
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

    println!("SDK headless needs proper implementation");
    println!("\n=== SESSION ENDED ===");
}
