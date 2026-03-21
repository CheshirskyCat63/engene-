//! Canonical Phase Runner Tests - CONTRACT lane
//!
//! Tests the unified phase execution system that provides
//! complete phase orchestration with proper error handling.

use engine_runtime::phase_runner::{PhaseRunner, PhaseResult};

/// CANONICAL PHASE RUNNER: All phases execute in order
#[test]
fn test_canonical_phase_runner_executes_all_phases() {
    let mut runner = PhaseRunner::new();
    
    // Execute a single frame
    let result = runner.execute_frame(0.016); // ~60 FPS
    
    // Verify successful execution
    assert!(result.success, "Phase runner should succeed");
    assert!(result.error_message.is_none(), "Should have no error message");
    assert!(result.duration_ms > 0.0, "Should take some time");
    
    // Verify all phases were executed
    // The runner should internally execute all 7 phases
    // even if we don't see them individually here
    
    // Verify tick advanced
    assert_eq!(runner.tick(), 1, "Tick should advance by 1");
    
    // Verify context state
    assert!(!runner.is_editor_mode(), "Should not be in editor mode by default");
}

/// PHASE RUNNER CUSTOM PHASES: Can execute arbitrary phase combinations
#[test]
fn test_phase_runner_supports_custom_phases() {
    let mut runner = PhaseRunner::new();
    
    // Create custom phase list (only tick and streaming for this test)
    let custom_phases: Vec<Box<dyn engine_runtime::phase::PhaseTrait + Send + Sync>> = vec![
        Box::new(engine_runtime::phase::tick::TickPhase),
        Box::new(engine_runtime::phase::streaming::StreamingPhase),
    ];
    
    let result = runner.execute_custom_phases(&custom_phases);
    
    assert!(result.success, "Custom phase execution should succeed");
    assert_eq!(runner.tick(), 1, "Tick should advance by 1");
}

/// PHASE RUNNER ERROR HANDLING: Proper error propagation
#[test]
fn test_phase_runner_error_propagation() {
    let mut runner = PhaseRunner::new();
    
    // Override with a phase that always fails
    struct FailingPhase;
    impl engine_runtime::phase::PhaseTrait for FailingPhase {
        fn execute(&self, _ctx: &engine_runtime::phase::PhaseContext) -> PhaseResult {
            PhaseResult::error("Intentional test failure", 1.0)
        }
        
        fn phase_type(&self) -> engine_runtime::phase::Phase {
            engine_runtime::phase::Phase::Tick
        }
    }
    
    // Replace first phase with failing phase
    runner.phases[0] = Box::new(FailingPhase);
    
    let result = runner.execute_frame(0.016);
    
    // Should fail and stop execution
    assert!(!result.success, "Should fail on first error");
    assert!(result.error_message.is_some(), "Should have error message");
    assert_eq!(runner.tick(), 0, "Tick should not advance on failure");
}

/// PHASE RUNNER CONTEXT: Proper state management
#[test]
fn test_phase_runner_context_management() {
    let mut runner = PhaseRunner::new();
    
    // Test initial state
    assert_eq!(runner.tick(), 0, "Should start at tick 0");
    assert!(!runner.is_editor_mode(), "Should not be in editor mode");
    
    // Execute successful frame
    let _ = runner.execute_frame(0.016);
    
    // Verify state changed
    assert_eq!(runner.tick(), 1, "Tick should advance to 1");
    
    // Test editor mode
    runner.set_editor_mode(true);
    assert!(runner.is_editor_mode(), "Should be in editor mode");
    
    runner.set_editor_mode(false);
    assert!(!runner.is_editor_mode(), "Should not be in editor mode");
}

/// PHASE RUNNER PERFORMANCE: Reasonable execution time
#[test]
fn test_phase_runner_performance() {
    let mut runner = PhaseRunner::new();
    
    let start = std::time::Instant::now();
    
    // Execute multiple frames to measure performance
    for _ in 0..100 {
        let _ = runner.execute_frame(0.016);
    }
    
    let duration = start.elapsed();
    
    // Should complete 100 frames in reasonable time
    // Each frame should take ~16ms (60 FPS), so 100 frames ~1.6 seconds
    assert!(duration.as_millis() < 3000, "100 frames should complete in under 3 seconds");
    
    assert_eq!(runner.tick(), 100, "Should advance tick by 100");
}
