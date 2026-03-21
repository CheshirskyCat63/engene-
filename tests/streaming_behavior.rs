//! # Streaming Behavioral Contracts
//! 
//! These tests verify DETERMINISTIC streaming behavior, not just API presence.
//! Each test uses specific inputs and expects EXACT outputs.

use engine_runtime::phase::streaming::{run_streaming, StreamingInput};

/// Test 1: Basic load decisions around player
#[test]
fn streaming_loads_around_player_deterministically() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2, // 5x5 grid = 25 chunks max
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should respect budget - loads min(25, 16) = 16 chunks
    assert_eq!(output.load_decisions, 16, "Should load exactly 16 chunks (budget limit)");
    assert_eq!(output.unload_decisions, 0, "Should unload nothing initially");
    assert!(output.budget_saturation, "Budget should be saturated (16 >= 16)");
    
    // Verify specific chunks are loaded (row-major order from top-left)
    let loaded_chunks: std::collections::HashSet<_> = output.chunks_to_load.iter().cloned().collect();
    
    // Center chunk should always be loaded
    assert!(loaded_chunks.contains(&[0, 0]), "Center chunk [0,0] must be loaded");
    
    // Should load chunks in deterministic order (row-major from top-left)
    let expected_chunks = vec![
        [-2, -2], [-2, -1], [-2, 0], [-2, 1], [-2, 2], // Row x=-2
        [-1, -2], [-1, -1], [-1, 0], [-1, 1], [-1, 2], // Row x=-1
        [0, -2],  [0, -1],  [0, 0],  [0, 1],  [0, 2],  // Row x=0 (first 5)
        [1, -2],                                          // Row x=1 (1st to reach 16)
    ];
    
    for chunk in &expected_chunks {
        assert!(loaded_chunks.contains(chunk), "Expected chunk {:?} must be loaded", chunk);
    }
    
    // Should not load beyond budget limit
    assert_eq!(output.chunks_to_load.len(), 16, "Should load exactly budget amount");
}

/// Test 2: Known chunks are excluded from new loads
#[test]
fn streaming_excludes_known_loaded_chunks() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: vec![[0, 0], [1, 0], [0, 1]], // Already loaded
    };
    
    let output = run_streaming(input);
    
    // Should load fewer chunks since some are already known
    // Total candidates: 25 - 3 known = 22, but budget is 16, so loads 16
    assert_eq!(output.load_decisions, 16, "Should load 16 chunks (budget limit)");
    assert_eq!(output.unload_decisions, 0, "Should unload nothing");
    
    // Known chunks should NOT be in load list
    let loaded_chunks: std::collections::HashSet<_> = output.chunks_to_load.iter().cloned().collect();
    
    assert!(!loaded_chunks.contains(&[0, 0]), "Known chunk [0,0] should not be in load list");
    assert!(!loaded_chunks.contains(&[1, 0]), "Known chunk [1,0] should not be in load list");
    assert!(!loaded_chunks.contains(&[0, 1]), "Known chunk [0,1] should not be in load list");
    
    // But other chunks should be loaded
    assert!(loaded_chunks.contains(&[-1, 0]), "Unknown chunk [-1,0] should be loaded");
    assert!(loaded_chunks.contains(&[0, -1]), "Unknown chunk [0,-1] should be loaded");
    assert!(loaded_chunks.contains(&[-2, -2]), "Unknown chunk [-2,-2] should be loaded");
}

/// Test 3: Unloading distant chunks
#[test]
fn streaming_unloads_distant_chunks() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2,
        pending_unload_count: 3, // Need to make room
        residency_budget: 10,
        known_loaded_chunks: vec![
            [0, 0], [1, 0], [0, 1],    // Close chunks
            [5, 5], [6, 5], [5, 6],    // Distant chunks (should be unloaded)
            [2, 0], [0, 2],            // Medium distance
        ],
    };
    
    let output = run_streaming(input);
    
    // Should unload the distant chunks
    assert_eq!(output.unload_decisions, 3, "Should unload exactly 3 distant chunks");
    
    let unloaded_chunks: std::collections::HashSet<_> = output.chunks_to_unload.iter().cloned().collect();
    
    // Distant chunks should be unloaded
    assert!(unloaded_chunks.contains(&[5, 5]), "Distant chunk [5,5] should be unloaded");
    assert!(unloaded_chunks.contains(&[6, 5]), "Distant chunk [6,5] should be unloaded");
    assert!(unloaded_chunks.contains(&[5, 6]), "Distant chunk [5,6] should be unloaded");
    
    // Close chunks should NOT be unloaded
    assert!(!unloaded_chunks.contains(&[0, 0]), "Close chunk [0,0] should not be unloaded");
    assert!(!unloaded_chunks.contains(&[1, 0]), "Close chunk [1,0] should not be unloaded");
    assert!(!unloaded_chunks.contains(&[0, 1]), "Close chunk [0,1] should not be unloaded");
}

/// Test 4: Budget saturation behavior
#[test]
fn streaming_budget_saturation_is_deterministic() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 4, // Would be 81 chunks (9x9 grid)
        pending_unload_count: 0,
        residency_budget: 10, // Much smaller than potential loads
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // Should respect budget exactly
    assert_eq!(output.load_decisions, 10, "Should load exactly budget amount");
    assert!(output.budget_saturation, "Budget should be saturated");
    
    // Should load the 10 closest chunks to player (row-major order from top-left)
    let loaded_chunks: std::collections::HashSet<_> = output.chunks_to_load.iter().cloned().collect();
    
    // Verify row-major order from top-left
    let expected_close_chunks = vec![
        [-4, -4], [-4, -3], [-4, -2], [-4, -1], [-4, 0], // Row x=-4 (first 5)
        [-4, 1], [-4, 2], [-4, 3], [-4, 4], [-3, -4],      // Row x=-4 (remaining 5) + x=-3 (1st)
    ];
    
    for chunk in &expected_close_chunks {
        assert!(loaded_chunks.contains(chunk), "Close chunk {:?} should be loaded", chunk);
    }
    
    // Should NOT load distant chunks when budget is limited
    assert!(!loaded_chunks.contains(&[4, 4]), "Distant chunk [4,4] should not be loaded");
    assert!(!loaded_chunks.contains(&[0, 4]), "Distant chunk [0,4] should not be loaded");
}

/// Test 5: Deterministic replay - same input = same output
#[test]
fn streaming_is_deterministic() {
    let input = StreamingInput {
        tick: 42, // Specific tick number
        player_position: Some([3.5, -2.1, 0.0]),
        view_distance_chunks: 3,
        pending_unload_count: 2,
        residency_budget: 12,
        known_loaded_chunks: vec![[1, 1], [2, 2], [4, 0]],
    };
    
    // Run twice with identical input
    let output1 = run_streaming(input.clone());
    let output2 = run_streaming(input.clone());
    
    // Should be identical
    assert_eq!(output1.load_decisions, output2.load_decisions, "Load decisions must be deterministic");
    assert_eq!(output1.unload_decisions, output2.unload_decisions, "Unload decisions must be deterministic");
    assert_eq!(output1.chunks_to_load, output2.chunks_to_load, "Load lists must be identical");
    assert_eq!(output1.chunks_to_unload, output2.chunks_to_unload, "Unload lists must be identical");
    assert_eq!(output1.budget_saturation, output2.budget_saturation, "Budget saturation must be deterministic");
    
    // Verify specific expected values for this input
    assert_eq!(output1.load_decisions, 12, "Expected 12 load decisions for this specific input");
    assert_eq!(output1.unload_decisions, 0, "Expected 0 unload decisions for this specific input (no unloading logic triggered)");
    assert!(output1.budget_saturation, "Budget should be saturated for this input");
}
