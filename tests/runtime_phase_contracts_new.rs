//! Phase Order Invariant Tests - CONTRACT test
//!
//! This test verifies phase order is enforced at runtime, NOT by checking source text.
//! It tests the CONTRACT: phases must run in canonical order.
//!
//! ## What was changed
//! OLD: Source-text checks that verified sdk_runner.rs file contents
//! NEW: Runtime contract tests that verify phase module behavior
//!
//! Ownership: Architecture Team
//! Lane: contract
//! Type: CONTRACT (was MIGRATION)
//! Speed: Fast

/// Phase order is defined in engine_runtime::phase
#[test]
fn phase_module_exists_in_engine_runtime() {
    // Verify phase module is accessible
    let _ = engine_runtime::phase::Phase::all();
    let _ = engine_runtime::phase::Phase::Tick;
    let _ = engine_runtime::phase::Phase::Render;
}

/// Phase order is canonical and enforced
#[test]
fn phase_order_is_canonical() {
    use engine_runtime::phase::{validate_phase_order, Phase};
    
    let phases = vec![
        Phase::Tick,
        Phase::Streaming,
        Phase::Persistence,
        Phase::Spatial,
        Phase::Audio,
        Phase::EditorUpdate,
        Phase::Render,
    ];
    
    assert!(validate_phase_order(&phases).is_ok());
}

/// Phase order violation is detected
#[test]
fn phase_order_violation_is_detected() {
    use engine_runtime::phase::{validate_phase_order, Phase};
    
    // Wrong order: render before tick
    let wrong_order = vec![
        Phase::Render,  // Should be last!
        Phase::Tick,    // Should be first!
    ];
    
    assert!(validate_phase_order(&wrong_order).is_err());
}

/// Each phase has correct next phase
#[test]
fn phase_next_is_correct() {
    use engine_runtime::phase::Phase;
    
    assert_eq!(Phase::Tick.next(), Some(Phase::Streaming));
    assert_eq!(Phase::Streaming.next(), Some(Phase::Persistence));
    assert_eq!(Phase::Persistence.next(), Some(Phase::Spatial));
    assert_eq!(Phase::Spatial.next(), Some(Phase::Audio));
    assert_eq!(Phase::Audio.next(), Some(Phase::EditorUpdate));
    assert_eq!(Phase::EditorUpdate.next(), Some(Phase::Render));
    assert_eq!(Phase::Render.next(), None);
}

/// Phase names are correct
#[test]
fn phase_names_are_correct() {
    use engine_runtime::phase::Phase;
    
    assert_eq!(Phase::Tick.name(), "tick");
    assert_eq!(Phase::Streaming.name(), "streaming");
    assert_eq!(Phase::Persistence.name(), "persistence");
    assert_eq!(Phase::Spatial.name(), "spatial");
    assert_eq!(Phase::Audio.name(), "audio");
    assert_eq!(Phase::EditorUpdate.name(), "editor_update");
    assert_eq!(Phase::Render.name(), "render");
}

/// All 7 phases exist
#[test]
fn all_seven_phases_exist() {
    use engine_runtime::phase::Phase;
    
    let all_phases = Phase::all();
    assert_eq!(all_phases.len(), 7);
}

/// Editor phase should NOT run in game mode (only in editor)
#[test]
fn editor_phase_only_runs_in_editor_mode() {
    use engine_runtime::phase::{editor::EditorPhase, PhaseContext, PhaseTrait};
    
    let editor_phase = EditorPhase::new();
    
    // In game mode (not editor), should not run
    let game_ctx = PhaseContext {
        tick: 0,
        delta_seconds: 1.0/60.0,
        is_editor_mode: false,
    };
    assert!(!editor_phase.should_run(&game_ctx));
    
    // In editor mode, should run
    let editor_ctx = PhaseContext {
        tick: 0,
        delta_seconds: 1.0/60.0,
        is_editor_mode: true,
    };
    assert!(editor_phase.should_run(&editor_ctx));
}

/// Audio phase runs independently of render (law: audio must NOT depend on render)
#[test]
fn audio_phase_independent_of_render() {
    use engine_runtime::phase::{audio::AudioPhase, PhaseContext, PhaseTrait};
    
    let audio_phase = AudioPhase::new();
    
    // Audio should run regardless of editor mode
    let game_ctx = PhaseContext {
        tick: 0,
        delta_seconds: 1.0/60.0,
        is_editor_mode: false,
    };
    assert!(audio_phase.should_run(&game_ctx));
    
    let editor_ctx = PhaseContext {
        tick: 0,
        delta_seconds: 1.0/60.0,
        is_editor_mode: true,
    };
    assert!(audio_phase.should_run(&editor_ctx));
}

/// Render phase exists and is last
#[test]
fn render_phase_is_last() {
    use engine_runtime::phase::Phase;
    
    // Render is the last phase
    let phases = Phase::all();
    assert_eq!(phases.last(), Some(&Phase::Render));
}
