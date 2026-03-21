//! Streaming contract invariants test

use super::{run_streaming, StreamingInput};

#[test]
fn test_streaming_budget_limits_loads() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 10, // Would be 100+ chunks without budget
        pending_unload_count: 0,
        residency_budget: 4,      // Very small budget
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Budget should limit load decisions
    assert!(output.load_decisions <= 4, "Load decisions {} exceed budget {}", output.load_decisions, 4);
}

#[test]
fn test_streaming_budget_saturation_triggers() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 10, // Would be 100+ chunks without budget
        pending_unload_count: 0,
        residency_budget: 4,      // Very small budget
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should trigger budget saturation when needed chunks exceed budget
    assert!(output.budget_saturation, "Budget saturation should trigger when view_distance requires more chunks than budget allows");
}

#[test]
fn test_streaming_no_player_gives_empty_path() {
    let input = StreamingInput {
        tick: 0,
        player_position: None, // No player - headless mode
        view_distance_chunks: 8,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should have predictable empty path when no player
    assert_eq!(output.load_decisions, 0, "No player should result in no load decisions");
    assert_eq!(output.unload_decisions, 0, "No player should result in no unload decisions");
    assert!(!output.budget_saturation, "No player should not trigger budget saturation");
}

#[test]
fn test_streaming_unloads_far_chunks() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 4,
        pending_unload_count: 10, // Allow unloading
        residency_budget: 16,
        known_loaded_chunks: vec![[10, 10], [15, 15]], // Far chunks
    };
    
    let output = run_streaming(input);
    
    // Should unload far chunks
    assert!(output.unload_decisions > 0, "Should unload far chunks");
    
    // Should not contain placeholder [0, 0]
    // Note: This test would need access to the actual chunks_to_unload list
    // For now, we just verify that unload decisions are made
}

#[test]
fn test_streaming_respects_known_loaded() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 4,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: vec![[0, 0], [1, 1]], // Already loaded nearby chunks
    };
    
    let output = run_streaming(input);
    
    // Should not reload already known chunks
    // This is a basic test - more sophisticated testing would require
    // access to the actual chunks_to_load list
    assert!(output.load_decisions > 0, "Load decisions should be positive");
}
