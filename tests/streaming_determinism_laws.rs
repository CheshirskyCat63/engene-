//! # Streaming Determinism Laws
//! 
//! These tests verify DETERMINISM GUARANTEES that must never change.
//! They test repeatability, order-independence where applicable, and temporal consistency.

use engine_runtime::phase::streaming::{run_streaming, StreamingInput};

/// Determinism Law 1: Pure input-output determinism
/// 
/// Same input must always produce identical output, regardless of
/// when the test is run, system state, or external factors.
#[test]
fn streaming_pure_input_output_determinism() {
    let test_cases = vec![
        // Case 1: Minimal scenario
        StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 1,
            pending_unload_count: 0,
            residency_budget: 5,
            known_loaded_chunks: Vec::new(),
        },
        // Case 2: Standard scenario
        StreamingInput {
            tick: 42,
            player_position: Some([3.5, -2.1, 0.0]),
            view_distance_chunks: 2,
            pending_unload_count: 2,
            residency_budget: 16,
            known_loaded_chunks: vec![[1, 1], [2, 2]],
        },
        // Case 3: High stress scenario
        StreamingInput {
            tick: 999,
            player_position: Some([-7.2, 3.1, 0.0]),
            view_distance_chunks: 4,
            pending_unload_count: 5,
            residency_budget: 100,
            known_loaded_chunks: vec![[-1, -1], [0, 1], [2, 3], [4, 4]],
        },
        // Case 4: Edge case - zero budget
        StreamingInput {
            tick: 123,
            player_position: Some([1.0, 1.0, 0.0]),
            view_distance_chunks: 3,
            pending_unload_count: 1,
            residency_budget: 0,
            known_loaded_chunks: vec![[0, 0]],
        },
        // Case 5: Edge case - no player position
        StreamingInput {
            tick: 456,
            player_position: None,
            view_distance_chunks: 2,
            pending_unload_count: 0,
            residency_budget: 10,
            known_loaded_chunks: Vec::new(),
        },
    ];
    
    for (case_idx, input) in test_cases.into_iter().enumerate() {
        // Run multiple times with identical input
        let outputs: Vec<_> = (0..5).map(|_| run_streaming(input.clone())).collect();
        
        // DETERMINISM: All outputs must be identical
        let first = &outputs[0];
        
        for (run_idx, output) in outputs.iter().enumerate() {
            assert_eq!(output.load_decisions, first.load_decisions, 
                "Case {}, Run {}: Load decisions differ", case_idx, run_idx);
            assert_eq!(output.unload_decisions, first.unload_decisions, 
                "Case {}, Run {}: Unload decisions differ", case_idx, run_idx);
            assert_eq!(output.chunks_to_load, first.chunks_to_load, 
                "Case {}, Run {}: Load lists differ", case_idx, run_idx);
            assert_eq!(output.chunks_to_unload, first.chunks_to_unload, 
                "Case {}, Run {}: Unload lists differ", case_idx, run_idx);
            assert_eq!(output.budget_saturation, first.budget_saturation, 
                "Case {}, Run {}: Budget saturation differs", case_idx, run_idx);
            
            // Duration may vary slightly due to system timing, but should be very close
            assert!((output.duration_ms - first.duration_ms).abs() < 1.0,
                "Case {}, Run {}: Duration differs too much: {} vs {}", 
                case_idx, run_idx, output.duration_ms, first.duration_ms);
        }
    }
}

/// Determinism Law 2: Temporal consistency
/// 
/// Tick number should not affect output when other inputs are identical.
/// This ensures streaming is stateless with respect to tick progression.
#[test]
fn streaming_temporal_consistency() {
    let base_input = StreamingInput {
        tick: 0, // Will be varied
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2,
        pending_unload_count: 1,
        residency_budget: 16,
        known_loaded_chunks: vec![[1, 1], [2, 2]],
    };
    
    let tick_numbers = [0, 1, 42, 999, 1000000];
    
    let mut first_output: Option<engine_runtime::phase::streaming::StreamingOutput> = None;
    
    for &tick_number in &tick_numbers {
        let mut input = base_input.clone();
        input.tick = tick_number;
        
        let output = run_streaming(input);
        
        if let Some(first) = &first_output {
            // DETERMINISM: Tick number should not affect streaming decisions
            assert_eq!(output.load_decisions, first.load_decisions,
                "Tick number {} affected load decisions", tick_number);
            assert_eq!(output.unload_decisions, first.unload_decisions,
                "Tick number {} affected unload decisions", tick_number);
            assert_eq!(output.chunks_to_load, first.chunks_to_load,
                "Tick number {} affected load list", tick_number);
            assert_eq!(output.chunks_to_unload, first.chunks_to_unload,
                "Tick number {} affected unload list", tick_number);
            assert_eq!(output.budget_saturation, first.budget_saturation,
                "Tick number {} affected budget saturation", tick_number);
        } else {
            first_output = Some(output);
        }
    }
}

/// Determinism Law 3: Input permutation invariance
/// 
// Order of known_loaded_chunks should not affect output.
/// This tests that streaming treats known chunks as a set, not an ordered list.
#[test]
fn streaming_input_permutation_invariance() {
    let base_input = StreamingInput {
        tick: 42,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 3,
        pending_unload_count: 2,
        residency_budget: 20,
        known_loaded_chunks: vec![[0, 0], [1, 1], [2, 2], [3, 3]], // Will be permuted
    };
    
    let chunk_permutations = vec![
        vec![[0, 0], [1, 1], [2, 2], [3, 3]],
        vec![[3, 3], [2, 2], [1, 1], [0, 0]],
        vec![[1, 1], [0, 0], [3, 3], [2, 2]],
        vec![[2, 2], [3, 3], [0, 0], [1, 1]],
        vec![[1, 1], [3, 3], [2, 2], [0, 0]],
    ];
    
    let mut first_output: Option<engine_runtime::phase::streaming::StreamingOutput> = None;
    
    for (perm_idx, known_chunks) in chunk_permutations.into_iter().enumerate() {
        let mut input = base_input.clone();
        input.known_loaded_chunks = known_chunks;
        
        let output = run_streaming(input);
        
        if let Some(first) = &first_output {
            // DETERMINISM: Known chunks order should not affect output
            assert_eq!(output.load_decisions, first.load_decisions,
                "Permutation {} affected load decisions", perm_idx);
            assert_eq!(output.unload_decisions, first.unload_decisions,
                "Permutation {} affected unload decisions", perm_idx);
            assert_eq!(output.chunks_to_load, first.chunks_to_load,
                "Permutation {} affected load list", perm_idx);
            assert_eq!(output.chunks_to_unload, first.chunks_to_unload,
                "Permutation {} affected unload list", perm_idx);
            assert_eq!(output.budget_saturation, first.budget_saturation,
                "Permutation {} affected budget saturation", perm_idx);
        } else {
            first_output = Some(output);
        }
    }
}

/// Determinism Law 4: Floating-point stability
/// 
// Small floating-point variations in position should not cause
// dramatically different streaming decisions (within reasonable bounds).
#[test]
fn streaming_floating_point_stability() {
    let base_position = [0.0, 0.0, 0.0];
    let variations = [
        [0.0, 0.0, 0.0],           // Exact
        [0.0001, 0.0001, 0.0],     // Small positive
        [-0.0001, -0.0001, 0.0],   // Small negative
        [0.000001, 0.000001, 0.0], // Very small
        [1e-10, 1e-10, 0.0],       // Extremely small
    ];
    
    let base_input = StreamingInput {
        tick: 0,
        player_position: Some(base_position), // Will be varied
        view_distance_chunks: 2,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: Vec::new(),
    };
    
    let mut reference_output: Option<engine_runtime::phase::streaming::StreamingOutput> = None;
    
    for (var_idx, &position) in variations.iter().enumerate() {
        let mut input = base_input.clone();
        input.player_position = Some(position);
        
        let output = run_streaming(input);
        
        if let Some(reference) = &reference_output {
            // DETERMINISM: Small position variations should not cause major changes
            // Allow for some boundary changes due to chunk grid alignment
            let load_diff = (output.load_decisions as i32 - reference.load_decisions as i32).abs();
            let unload_diff = (output.unload_decisions as i32 - reference.unload_decisions as i32).abs();
            
            assert!(load_diff <= 2, 
                "Position variation {} caused too many load decision changes: {} vs {}", 
                var_idx, output.load_decisions, reference.load_decisions);
            assert!(unload_diff <= 1,
                "Position variation {} caused too many unload decision changes: {} vs {}",
                var_idx, output.unload_decisions, reference.unload_decisions);
        } else {
            reference_output = Some(output);
        }
    }
}

/// Determinism Law 5: Repeatability under stress
/// 
// Determinism must hold even under high-stress conditions with
// large view distances, many known chunks, and complex scenarios.
#[test]
fn streaming_repeatability_under_stress() {
    let stress_scenarios = vec![
        // Stress 1: Large view distance
        StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 10, // 21x21 = 441 chunks
            pending_unload_count: 0,
            residency_budget: 100,
            known_loaded_chunks: (0..50).map(|i| [i % 10 - 5, i / 10 - 5]).collect(),
        },
        // Stress 2: Many known chunks
        StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 5,
            pending_unload_count: 10,
            residency_budget: 25,
            known_loaded_chunks: (0..100).map(|i| [i % 20 - 10, i / 20 - 10]).collect(),
        },
        // Stress 3: High pending unload
        StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 3,
            pending_unload_count: 50,
            residency_budget: 16,
            known_loaded_chunks: (0..75).map(|i| [i % 15 - 7, i / 15 - 7]).collect(),
        },
    ];
    
    for (stress_idx, input) in stress_scenarios.into_iter().enumerate() {
        // Run multiple times under stress
        let output1 = run_streaming(input.clone());
        let output2 = run_streaming(input.clone());
        let output3 = run_streaming(input.clone());
        
        // DETERMINISM: Even under stress, results must be identical
        assert_eq!(output1.load_decisions, output2.load_decisions, 
            "Stress {}: Load decisions not repeatable", stress_idx);
        assert_eq!(output2.load_decisions, output3.load_decisions, 
            "Stress {}: Load decisions not repeatable", stress_idx);
        
        assert_eq!(output1.unload_decisions, output2.unload_decisions, 
            "Stress {}: Unload decisions not repeatable", stress_idx);
        assert_eq!(output2.unload_decisions, output3.unload_decisions, 
            "Stress {}: Unload decisions not repeatable", stress_idx);
        
        assert_eq!(output1.chunks_to_load, output2.chunks_to_load, 
            "Stress {}: Load lists not repeatable", stress_idx);
        assert_eq!(output2.chunks_to_load, output3.chunks_to_load, 
            "Stress {}: Load lists not repeatable", stress_idx);
        
        assert_eq!(output1.chunks_to_unload, output2.chunks_to_unload, 
            "Stress {}: Unload lists not repeatable", stress_idx);
        assert_eq!(output2.chunks_to_unload, output3.chunks_to_unload, 
            "Stress {}: Unload lists not repeatable", stress_idx);
        
        assert_eq!(output1.budget_saturation, output2.budget_saturation, 
            "Stress {}: Budget saturation not repeatable", stress_idx);
        assert_eq!(output2.budget_saturation, output3.budget_saturation, 
            "Stress {}: Budget saturation not repeatable", stress_idx);
    }
}
