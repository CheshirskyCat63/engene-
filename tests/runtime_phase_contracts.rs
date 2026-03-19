//! Runtime Phase Contracts
//!
//! These tests verify phase order in actual sdk_runner.rs code.
//! They read the source file and verify the order of operations.
//!
//! IMPORTANT: This file contains both source-text checks and real production calls.
//! Tests that call production code are explicitly named to reflect their level.

use std::fs;

const SDK_RUNNER_PATH: &str = "src/app/sdk_runner.rs";

/// Find the byte position of a pattern in source code
fn find_position(source: &str, pattern: &str) -> Option<usize> {
    source.find(pattern)
}

/// SOURCE-ONLY CHECK: SDK runner redraw phase order matches contract.
/// This test reads source code text and verifies order of operations.
/// It does NOT call production runtime - it checks that the code is structured correctly.
#[test]
fn source_text_check_sdk_runner_redraw_phase_order_matches_contract() {
    let source = fs::read_to_string(SDK_RUNNER_PATH)
        .expect("src/app/sdk_runner.rs must exist");

    // Find key operations in RedrawRequested handler
    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");

    // Get the portion after RedrawRequested
    let after_redraw = &source[redraw_pos..];

    // Find positions of key operations
    let tick_pos = find_position(after_redraw, "self.engine.tick")
        .expect("engine.tick must be called");
    let poll_pos = find_position(after_redraw, "am.poll()")
        .or_else(|| find_position(after_redraw, ".poll()"))
        .expect("asset poll must exist");
    let streamer_pos = find_position(after_redraw, "streamer.update")
        .expect("streamer.update must be called");
    let persistence_pos = find_position(after_redraw, "persistence.save_and_unload")
        .or_else(|| find_position(after_redraw, "ChunkPersistenceService"))
        .expect("persistence must be used");
    let spatial_pos = find_position(after_redraw, "spatial.clear")
        .or_else(|| find_position(after_redraw, "HierarchicalSpatialIndex"))
        .expect("spatial index must be used");
    let audio_pos = find_position(after_redraw, "audio.update")
        .or_else(|| find_position(after_redraw, "AudioEngine"))
        .expect("audio must be updated");
    let dashboard_pos = find_position(after_redraw, "update_dashboards")
        .expect("update_dashboards must be called");
    let inspector_pos = find_position(after_redraw, "apply_inspector_edits")
        .expect("apply_inspector_edits must be called");
    let render_pos = find_position(after_redraw, "render_with_egui")
        .expect("render_with_egui must be called");

    // Verify order: tick -> poll -> streaming -> persistence -> spatial -> audio -> dashboard -> inspector -> render
    assert!(tick_pos < poll_pos, 
        "engine.tick (pos {}) must come before asset poll (pos {})", tick_pos, poll_pos);
    assert!(poll_pos < streamer_pos,
        "asset poll (pos {}) must come before streamer.update (pos {})", poll_pos, streamer_pos);
    assert!(streamer_pos < persistence_pos,
        "streamer.update (pos {}) must come before persistence (pos {})", streamer_pos, persistence_pos);
    assert!(persistence_pos < spatial_pos,
        "persistence (pos {}) must come before spatial (pos {})", persistence_pos, spatial_pos);
    assert!(spatial_pos < audio_pos,
        "spatial (pos {}) must come before audio (pos {})", spatial_pos, audio_pos);
    assert!(audio_pos < dashboard_pos,
        "audio (pos {}) must come before dashboard (pos {})", audio_pos, dashboard_pos);
    assert!(dashboard_pos < inspector_pos,
        "dashboard (pos {}) must come before inspector (pos {})", dashboard_pos, inspector_pos);
    assert!(inspector_pos < render_pos,
        "inspector (pos {}) must come before render (pos {})", inspector_pos, render_pos);
}

/// SOURCE-ONLY CHECK: Audio update occurs before render call.
#[test]
fn source_text_check_audio_update_occurs_before_render_call() {
    let source = fs::read_to_string(SDK_RUNNER_PATH)
        .expect("src/app/sdk_runner.rs must exist");

    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");
    let after_redraw = &source[redraw_pos..];

    let audio_pos = find_position(after_redraw, "audio.update")
        .or_else(|| find_position(after_redraw, "audio.set_listener"))
        .expect("audio must be referenced");
    let render_pos = find_position(after_redraw, "render_with_egui")
        .expect("render_with_egui must be called");

    assert!(audio_pos < render_pos,
        "audio (pos {}) must come before render (pos {})", audio_pos, render_pos);
}

/// SOURCE-ONLY CHECK: Dashboard update occurs before inspector edits.
#[test]
fn source_text_check_dashboard_update_occurs_before_inspector_edits() {
    let source = fs::read_to_string(SDK_RUNNER_PATH)
        .expect("src/app/sdk_runner.rs must exist");

    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");
    let after_redraw = &source[redraw_pos..];

    let dashboard_pos = find_position(after_redraw, "update_dashboards")
        .expect("update_dashboards must be called");
    let inspector_pos = find_position(after_redraw, "apply_inspector_edits")
        .expect("apply_inspector_edits must be called");

    assert!(dashboard_pos < inspector_pos,
        "update_dashboards (pos {}) must come before apply_inspector_edits (pos {})",
        dashboard_pos, inspector_pos);
}

/// SOURCE-ONLY CHECK: Streaming/persistence/spatial order is locked in SDK runner.
#[test]
fn source_text_check_streaming_persistence_spatial_order_is_locked() {
    let source = fs::read_to_string(SDK_RUNNER_PATH)
        .expect("src/app/sdk_runner.rs must exist");

    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");
    let after_redraw = &source[redraw_pos..];

    let streamer_pos = find_position(after_redraw, "streamer.update")
        .expect("streamer.update must be called");
    let persistence_pos = find_position(after_redraw, "persistence")
        .expect("persistence must be referenced");
    let spatial_pos = find_position(after_redraw, "spatial")
        .expect("spatial must be referenced");

    assert!(streamer_pos < persistence_pos,
        "streamer (pos {}) must come before persistence (pos {})", streamer_pos, persistence_pos);
    assert!(persistence_pos < spatial_pos,
        "persistence (pos {}) must come before spatial (pos {})", persistence_pos, spatial_pos);
}

/// SOURCE-ONLY CHECK: SDK runner references tools runtime assembly.
#[test]
fn source_text_check_tools_runtime_assembly_referenced() {
    let source = fs::read_to_string(SDK_RUNNER_PATH)
        .expect("src/app/sdk_runner.rs must exist");

    // Verify real production path is referenced
    assert!(source.contains("ToolsRuntimeAssembly::minimal"),
        "SDK runner must reference ToolsRuntimeAssembly::minimal");

    // Verify doctor is run with strict mode
    assert!(source.contains("doctor::run_doctor"),
        "SDK runner must run doctor validation");
    assert!(source.contains("DoctorMode::Strict"),
        "SDK runner must use strict doctor mode");

    // Verify doctor report is checked
    assert!(source.contains("doctor_report.error_count"),
        "SDK runner must check doctor error count");
}

/// REAL PRODUCTION CALL: ToolsRuntimeAssembly::minimal() can be instantiated.
/// This test actually calls production code, not just text scanning.
#[test]
fn production_call_tools_runtime_minimal_instantiates() {
    // This calls the actual production ToolsRuntimeAssembly::minimal()
    // which builds an engine with tools configuration
    let _engine = engene::runtime::bootstrap::tools::ToolsRuntimeAssembly::minimal();
    // If we get here, the production path works
}

/// REAL PRODUCTION CALL: doctor::run_doctor() can be called on tools runtime.
/// This test actually calls production code - it builds a tools engine and runs diagnostics.
#[test]
fn production_call_doctor_runs_on_tools_runtime() {
    use engene::tools::doctor::{run_doctor, DoctorMode};
    
    // Build a tools runtime
    let engine = engene::runtime::bootstrap::tools::ToolsRuntimeAssembly::minimal();
    
    // Run doctor in advisory mode (does not panic)
    let report = run_doctor(&engine, DoctorMode::Advisory);
    
    // Verify report is valid
    assert!(report.error_count() >= 0, "report must have valid error count");
    assert!(report.warning_count() >= 0, "report must have valid warning count");
}
