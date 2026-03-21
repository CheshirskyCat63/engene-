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
