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

/// EDITOR UPDATE WORKS: Verify implementation exists
#[test]
fn editor_update_function_has_implementation() {
    use sdk_app::editor::EditorShell;
    let mut shell = EditorShell::new();
    shell.update_dashboards(); // Should not panic
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

/// TICK EXTRACTION: Tick phase has public entrypoint
#[test]
fn tick_phase_has_public_entrypoint() {
    use engine_runtime::phase::tick::run_tick;
    
    // Verify entrypoint exists and returns valid result
    let result = run_tick(0, 1.0/60.0);
    assert!(result.is_ok());
}

/// TICK EXTRACTION: Tick is first phase in canonical order
#[test]
fn tick_is_first_phase() {
    use engine_runtime::phase::Phase;
    
    let phases = Phase::all();
    assert_eq!(phases.first(), Some(&Phase::Tick));
}

/// STREAMING EXTRACTION: Streaming phase has public entrypoint
#[test]
fn streaming_phase_has_public_entrypoint() {
    use engine_runtime::phase::streaming::{run_streaming, StreamingInput};
    
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 4,
        pending_unload_count: 0,
        residency_budget: 16,
    };
    
    let output = run_streaming(input);
    
    // Verify output has expected structure
    assert!(output.load_decisions > 0 || output.unload_decisions == 0);
    assert!(!output.duration_ms.is_nan());
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
    };
    
    let output = run_streaming(input);
    
    // Should respect budget
    assert!(output.load_decisions <= 4);
    assert!(output.budget_saturation);
}
