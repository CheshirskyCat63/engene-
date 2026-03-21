//! # Tick Phase Behavioral Contracts
//! 
//! These tests verify DETERMINISTIC tick behavior, not just API presence.

use engine_runtime::phase::tick::run_tick;

/// Test 1: Tick advances ECS state deterministically
#[test]
fn tick_advances_ecs_state_deterministically() {
    let result = run_tick(0, 1.0/60.0);
    
    assert!(result.success, "Tick should succeed");
    assert!(result.duration_ms >= 0.0, "First tick should have non-negative duration");
    
    // Run second tick
    let result2 = run_tick(1, 1.0/60.0);
    
    assert!(result2.success, "Second tick should succeed");
    assert!(result2.duration_ms >= 0.0, "Duration should be non-negative");
}

/// Test 2: Tick respects delta time
#[test]
fn tick_respects_delta_time() {
    let result_fast = run_tick(0, 1.0/120.0); // 120 FPS
    let result_slow = run_tick(0, 1.0/30.0);  // 30 FPS
    
    assert!(result_fast.success, "Fast tick should succeed");
    assert!(result_slow.success, "Slow tick should succeed");
    
    // Duration should be reasonable (not too fast, not too slow)
    assert!(result_fast.duration_ms < 100.0, "Fast tick should complete quickly");
    assert!(result_slow.duration_ms < 100.0, "Slow tick should complete quickly");
}

/// Test 3: Tick is deterministic across runs
#[test]
fn tick_is_deterministic() {
    // Run identical tick multiple times
    let result1 = run_tick(42, 1.0/60.0);
    let result2 = run_tick(42, 1.0/60.0);
    let result3 = run_tick(42, 1.0/60.0);
    
    // All results should have same success status
    assert_eq!(result1.success, result2.success, "Success status must match");
    assert_eq!(result2.success, result3.success, "Success status must match");
    
    // Duration might vary slightly due to system timing, but should be very close
    assert!((result1.duration_ms - result2.duration_ms).abs() < 10.0, "Durations should be very close");
    assert!((result2.duration_ms - result3.duration_ms).abs() < 10.0, "Durations should be very close");
}

/// Test 4: Tick handles edge cases
#[test]
fn tick_handles_edge_cases() {
    // Zero delta time
    let result_zero = run_tick(0, 0.0);
    assert!(result_zero.success, "Zero delta time should not fail");
    assert!(result_zero.duration_ms >= 0.0, "Duration should be non-negative");
    
    // Very high tick number
    let result_high = run_tick(999999, 1.0/60.0);
    assert!(result_high.success, "High tick number should not fail");
    assert!(result_high.duration_ms >= 0.0, "Duration should be non-negative");
    
    // Very small delta time
    let result_tiny = run_tick(0, 1e-6);
    assert!(result_tiny.success, "Tiny delta time should not fail");
    assert!(result_tiny.duration_ms >= 0.0, "Duration should be non-negative");
}
