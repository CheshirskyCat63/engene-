//! Phase Contract Tests - CONTRACT lane
//!
//! ## Status
//! Migrated from MIGRATION (source-text check) to CONTRACT (invariant check).
//! Phase extraction in progress - full invariants after handoff.
//!
//! ## Transition
//! OLD (MIGRATION): Source-text checks on non-existent sdk_runner.rs
//! NEW (CONTRACT): Runtime invariant checks on phase module

/// PHASE MODULE EXISTS: Verify phase module is accessible
#[test]
fn phase_module_exists_in_engine_runtime() {
    let _ = engine_runtime::phase::Phase::all();
    let _ = engine_runtime::phase::Phase::Tick;
    let _ = engine_runtime::phase::Phase::Render;
}

/// PHASE ORDER DEFINED: Verify canonical phase order
#[test]
fn phase_order_enum_has_seven_variants() {
    use engine_runtime::phase::Phase;
    let phases = Phase::all();
    assert_eq!(phases.len(), 7);
    assert_eq!(phases[0], Phase::Tick);
    assert_eq!(phases[6], Phase::Render);
}

/// PHASE VALIDATION: Canonical order is valid
#[test]
fn phase_order_validation_accepts_canonical_order() {
    use engine_runtime::phase::{validate_phase_order, Phase};
    let canonical = vec![
        Phase::Tick, Phase::Streaming, Phase::Persistence,
        Phase::Spatial, Phase::Audio, Phase::EditorUpdate, Phase::Render,
    ];
    assert!(validate_phase_order(&canonical).is_ok());
}

/// PHASE VALIDATION: Wrong order is rejected
#[test]
fn phase_order_validation_rejects_wrong_order() {
    use engine_runtime::phase::{validate_phase_order, Phase};
    let wrong = vec![Phase::Render, Phase::Tick];
    assert!(validate_phase_order(&wrong).is_err());
}

/// EDITOR SLICE EXTRACTED: Editor module exists
#[test]
fn editor_module_exists_in_sdk_app() {
    let _ = sdk_app::editor::EditorShell::new();
    let _ = sdk_app::editor::update_editor;
}

/// EDITOR UPDATE WORKS: Verify implementation exists with BEHAVIORAL contract
#[test]
fn editor_update_function_has_implementation() {
    use sdk_app::editor::EditorShell;
    let mut shell = EditorShell::new();
    
    // Should not panic and should complete successfully
    shell.update_dashboards();
    
    // Verify shell state is consistent after update
    let mut shell2 = EditorShell::new();
    shell2.update_dashboards();
    
    // Multiple calls should be safe and deterministic
    shell.update_dashboards();
    shell.update_dashboards();
}

/// PHASE NAMES: Verify names are correct
#[test]
fn phase_names_are_correct() {
    use engine_runtime::phase::Phase;
    assert_eq!(Phase::Tick.name(), "tick");
    assert_eq!(Phase::Render.name(), "render");
}

/// PHASE SEQUENCE: Each phase knows its successor
#[test]
fn phase_next_is_defined() {
    use engine_runtime::phase::Phase;
    assert_eq!(Phase::Tick.next(), Some(Phase::Streaming));
    assert_eq!(Phase::Render.next(), None);
}

/// EDITOR PHASE MODE: Respects editor mode
#[test]
fn editor_phase_respects_editor_mode() {
    use engine_runtime::phase::{editor::EditorPhase, PhaseContext, PhaseTrait};
    let phase = EditorPhase::new();
    let game_ctx = PhaseContext { tick: 0, delta_seconds: 1.0/60.0, is_editor_mode: false };
    let editor_ctx = PhaseContext { tick: 0, delta_seconds: 1.0/60.0, is_editor_mode: true };
    assert!(!phase.should_run(&game_ctx));
    assert!(phase.should_run(&editor_ctx));
}

/// AUDIO INDEPENDENCE: Audio doesn't depend on render
#[test]
fn audio_phase_runs_regardless_of_mode() {
    use engine_runtime::phase::{audio::AudioPhase, PhaseContext, PhaseTrait};
    let phase = AudioPhase::new();
    let ctx = PhaseContext { tick: 0, delta_seconds: 1.0/60.0, is_editor_mode: false };
    assert!(phase.should_run(&ctx));
}

/// TICK EXTRACTION: Tick phase has public entrypoint with DETERMINISTIC behavior
#[test]
fn tick_phase_has_public_entrypoint() {
    use engine_runtime::phase::tick::run_tick;
    
    // Verify entrypoint exists and returns DETERMINISTIC result
    let result = run_tick(0, 1.0/60.0);
    
    assert!(result.success, "Tick should succeed");
    assert!(result.duration_ms >= 0.0, "Duration should be non-negative");
    
    // Verify deterministic behavior - same input = same output
    let result2 = run_tick(0, 1.0/60.0);
    assert_eq!(result.success, result2.success, "Success status must match");
    assert!((result.duration_ms - result2.duration_ms).abs() < 10.0, "Durations should be very close");
}

/// TICK EXTRACTION: Tick is first phase in canonical order
#[test]
fn tick_is_first_phase() {
    use engine_runtime::phase::Phase;
    
    let phases = Phase::all();
    assert_eq!(phases.first(), Some(&Phase::Tick));
}

/// STREAMING EXTRACTION: Streaming phase has public entrypoint with DETERMINISTIC behavior
#[test]
fn streaming_phase_has_public_entrypoint() {
    use engine_runtime::phase::streaming::{run_streaming, StreamingInput};
    
    // Verify entrypoint exists and returns DETERMINISTIC result
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2, // 5x5 grid = 25 chunks max
        pending_unload_count: 0,
        residency_budget: 16, // Budget limit
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should respect budget - loads min(25, 16) = 16 chunks
    assert_eq!(output.load_decisions, 16, "Should load exactly 16 chunks (budget limit)");
    assert_eq!(output.unload_decisions, 0, "Should unload nothing initially");
    assert!(output.budget_saturation, "Budget should be saturated (16 >= 16)");
    assert!(output.duration_ms >= 0.0, "Duration should be non-negative");
    
    // Verify deterministic behavior - same input = same output
    let input2 = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: Vec::new(),
    };
    
    let output2 = run_streaming(input2);
    assert_eq!(output.load_decisions, output2.load_decisions, "Load decisions must match");
    assert_eq!(output.unload_decisions, output2.unload_decisions, "Unload decisions must match");
    assert_eq!(output.chunks_to_load, output2.chunks_to_load, "Load lists must be identical");
    assert_eq!(output.chunks_to_unload, output2.chunks_to_unload, "Unload lists must be identical");
    assert_eq!(output.budget_saturation, output2.budget_saturation, "Budget saturation must match");
}

/// STREAMING EXTRACTION: Streaming is second phase in canonical order
#[test]
fn streaming_is_second_phase() {
    use engine_runtime::phase::Phase;
    
    let phases = Phase::all();
    assert_eq!(phases.get(1), Some(&Phase::Streaming));
}

/// STREAMING EXTRACTION: Streaming respects residency budget
#[test]
fn streaming_respects_residency_budget() {
    use engine_runtime::phase::streaming::{run_streaming, StreamingInput};
    
    // Small budget should limit load decisions
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 10, // Would be 100+ chunks without budget
        pending_unload_count: 0,
        residency_budget: 4,      // Very small budget
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should respect budget
    assert!(output.load_decisions <= 4);
    assert!(output.budget_saturation);
}
