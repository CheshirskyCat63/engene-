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
    let source = fs::read_to_string(SDK_RUNNER_PATH).expect("src/app/sdk_runner.rs must exist");

    // Find key operations in RedrawRequested handler
    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");

    // Get the portion after RedrawRequested
    let after_redraw = &source[redraw_pos..];

    // Find positions of key operations
    let tick_pos = find_position(after_redraw, "sdk_runner_phases::tick::run")
        .expect("tick phase must be called");
    let poll_pos = find_position(after_redraw, "am.poll()")
        .or_else(|| find_position(after_redraw, ".poll()"))
        .expect("asset poll must exist");
    let streamer_pos = find_position(after_redraw, "sdk_runner_phases::streaming::run")
        .expect("streaming phase must be called");
    let persistence_pos = find_position(after_redraw, "sdk_runner_phases::persistence::run")
        .expect("persistence phase must be called");
    let audio_pos = find_position(after_redraw, "sdk_runner_phases::audio::run")
        .expect("audio phase must be called");
    let editor_pos = find_position(after_redraw, "sdk_runner_phases::editor::run")
        .expect("editor phase must be called");
    let spatial_pos = find_position(after_redraw, "sdk_runner_phases::spatial::run")
        .expect("spatial phase must be called");
    let render_pos = find_position(after_redraw, "sdk_runner_phases::render::run")
        .expect("render phase must be called");

    // Verify order: tick -> poll -> streaming -> persistence -> audio -> editor -> spatial -> render
    assert!(
        tick_pos < poll_pos,
        "engine.tick (pos {}) must come before asset poll (pos {})",
        tick_pos,
        poll_pos
    );
    assert!(
        poll_pos < streamer_pos,
        "asset poll (pos {}) must come before streaming (pos {})",
        poll_pos,
        streamer_pos
    );
    assert!(
        streamer_pos < persistence_pos,
        "streaming (pos {}) must come before persistence (pos {})",
        streamer_pos,
        persistence_pos
    );
    assert!(
        persistence_pos < audio_pos,
        "persistence (pos {}) must come before audio (pos {})",
        persistence_pos,
        audio_pos
    );
    assert!(
        audio_pos < editor_pos,
        "audio (pos {}) must come before editor (pos {})",
        audio_pos,
        editor_pos
    );
    assert!(
        editor_pos < spatial_pos,
        "editor (pos {}) must come before spatial (pos {})",
        editor_pos,
        spatial_pos
    );
    assert!(
        spatial_pos < render_pos,
        "spatial (pos {}) must come before render (pos {})",
        spatial_pos,
        render_pos
    );
}

/// SOURCE-ONLY CHECK: Audio update occurs before render call.
#[test]
fn source_text_check_audio_update_occurs_before_render_call() {
    let source = fs::read_to_string(SDK_RUNNER_PATH).expect("src/app/sdk_runner.rs must exist");

    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");
    let after_redraw = &source[redraw_pos..];

    let audio_pos = find_position(after_redraw, "sdk_runner_phases::audio::run")
        .expect("audio phase must be called");
    let render_pos = find_position(after_redraw, "sdk_runner_phases::render::run")
        .expect("render phase must be called");

    assert!(
        audio_pos < render_pos,
        "audio (pos {}) must come before render (pos {})",
        audio_pos,
        render_pos
    );
}

/// SOURCE-ONLY CHECK: Dashboard update occurs before inspector edits.
#[test]
fn source_text_check_dashboard_update_occurs_before_inspector_edits() {
    let source = fs::read_to_string("src/app/sdk_runner/sdk_runner_phases/editor.rs")
        .expect("src/app/sdk_runner/sdk_runner_phases/editor.rs must exist");

    let dashboard_pos =
        find_position(&source, "update_dashboards").expect("update_dashboards must be called");
    let inspector_pos = find_position(&source, "apply_inspector_edits")
        .expect("apply_inspector_edits must be called");

    assert!(
        dashboard_pos < inspector_pos,
        "update_dashboards (pos {}) must come before apply_inspector_edits (pos {})",
        dashboard_pos,
        inspector_pos
    );
}

/// SOURCE-ONLY CHECK: Streaming/editor/spatial order is locked in SDK runner.
#[test]
fn source_text_check_streaming_editor_spatial_order_is_locked() {
    let source = fs::read_to_string(SDK_RUNNER_PATH).expect("src/app/sdk_runner.rs must exist");

    let redraw_pos = find_position(&source, "WindowEvent::RedrawRequested")
        .expect("RedrawRequested handler must exist");
    let after_redraw = &source[redraw_pos..];

    let streamer_pos = find_position(after_redraw, "sdk_runner_phases::streaming::run")
        .expect("streaming phase must be called");
    let editor_pos = find_position(after_redraw, "sdk_runner_phases::editor::run")
        .expect("editor phase must be called");
    let spatial_pos = find_position(after_redraw, "sdk_runner_phases::spatial::run")
        .expect("spatial phase must be called");

    assert!(
        streamer_pos < editor_pos,
        "streamer (pos {}) must come before editor (pos {})",
        streamer_pos,
        editor_pos
    );
    assert!(
        editor_pos < spatial_pos,
        "editor (pos {}) must come before spatial (pos {})",
        editor_pos,
        spatial_pos
    );
}

/// SOURCE-ONLY CHECK: SDK runner references tools runtime assembly.
#[test]
fn source_text_check_tools_runtime_assembly_referenced() {
    let source = fs::read_to_string(SDK_RUNNER_PATH).expect("src/app/sdk_runner.rs must exist");

    // Verify real production path is referenced
    assert!(
        source.contains("ToolsRuntimeAssembly::minimal"),
        "SDK runner must reference ToolsRuntimeAssembly::minimal"
    );

    // Verify doctor is run with strict mode
    assert!(
        source.contains("doctor::run_doctor"),
        "SDK runner must run doctor validation"
    );
    assert!(
        source.contains("DoctorMode::Strict"),
        "SDK runner must use strict doctor mode"
    );

    // Verify doctor report is checked
    assert!(
        source.contains("doctor_report.error_count"),
        "SDK runner must check doctor error count"
    );
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
    let _ = report.error_count();
    let _ = report.warning_count();
}
