//! # Streaming Contract Law Tests
//! 
//! These tests verify ARCHITECTURAL LAWS that must never change.
//! They test fundamental invariants, not implementation details.
//! 
//! If these tests fail, the architectural contract is broken.

use engine_runtime::phase::streaming::{run_streaming, StreamingInput};

/// Contract Law 1: Budget is never exceeded
#[test]
fn streaming_never_exceeds_budget() {
    let budgets = [1, 5, 10, 16, 100];
    
    for &budget in &budgets {
        let input = StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 10, // Huge view distance (21x21 = 441 chunks)
            pending_unload_count: 0,
            residency_budget: budget,
            known_loaded_chunks: Vec::new(),
        };
        
        let output = run_streaming(input);
        
        // LAW: Load decisions must never exceed budget
        assert!(output.load_decisions <= budget, 
            "Budget law violated: {} > {} for budget {}", 
            output.load_decisions, budget, budget);
        
        // LAW: Budget saturation flag must be accurate
        assert_eq!(output.budget_saturation, output.load_decisions >= budget,
            "Budget saturation flag inaccurate for budget {}", budget);
    }
}

/// Contract Law 2: Known chunks are never reloaded
#[test]
fn streaming_never_reloads_known_chunks() {
    let known_chunks = vec![
        [0, 0], [1, 0], [0, 1], [-1, 0], [0, -1],
        [2, 2], [-2, -2], [3, 1], [-3, -1]
    ];
    
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 5,
        pending_unload_count: 0,
        residency_budget: 50,
        known_loaded_chunks: known_chunks.clone(),
    };
    
    let output = run_streaming(input);
    let loaded_chunks: std::collections::HashSet<_> = output.chunks_to_load.iter().cloned().collect();
    
    // LAW: No known chunk should appear in load list
    for known_chunk in &known_chunks {
        assert!(!loaded_chunks.contains(known_chunk),
            "Contract violated: Known chunk {:?} was scheduled for reload", known_chunk);
    }
}

/// Contract Law 3: Unload count is capped by pending_unload_count
#[test]
fn streaming_unload_count_is_capped() {
    let pending_counts = [0, 1, 5, 10, 20];
    
    for &pending_count in &pending_counts {
        let input = StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 2,
            pending_unload_count: pending_count,
            residency_budget: 16,
            known_loaded_chunks: vec![
                [0, 0], [1, 0], [0, 1], [-1, 0], [0, -1],
                [5, 5], [6, 5], [5, 6], [7, 7], [8, 8], // Distant chunks
                [2, 0], [0, 2], [-2, 0], [0, -2], [3, 3], // Medium distance
            ],
        };
        
        let output = run_streaming(input);
        
        // LAW: Unload decisions must never exceed pending count
        assert!(output.unload_decisions <= pending_count,
            "Unload cap violated: {} > {} for pending {}", 
            output.unload_decisions, pending_count, pending_count);
        
        // LAW: Unload list length must match unload decisions
        assert_eq!(output.chunks_to_unload.len(), output.unload_decisions as usize,
            "Unload list length mismatch for pending {}", pending_count);
    }
}

/// Contract Law 4: Deterministic output for identical input
#[test]
fn streaming_is_deterministic_by_contract() {
    let test_cases = vec![
        // Case 1: Basic scenario
        StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 2,
            pending_unload_count: 0,
            residency_budget: 16,
            known_loaded_chunks: Vec::new(),
        },
        // Case 2: With known chunks
        StreamingInput {
            tick: 42,
            player_position: Some([3.5, -2.1, 0.0]),
            view_distance_chunks: 3,
            pending_unload_count: 2,
            residency_budget: 12,
            known_loaded_chunks: vec![[1, 1], [2, 2], [4, 0]],
        },
        // Case 3: High budget scenario
        StreamingInput {
            tick: 999,
            player_position: Some([-7.2, 3.1, 0.0]),
            view_distance_chunks: 4,
            pending_unload_count: 5,
            residency_budget: 100,
            known_loaded_chunks: vec![[-1, -1], [0, 1]],
        },
    ];
    
    for (i, input) in test_cases.into_iter().enumerate() {
        // Run multiple times with identical input
        let output1 = run_streaming(input.clone());
        let output2 = run_streaming(input.clone());
        let output3 = run_streaming(input.clone());
        
        // LAW: All outputs must be identical
        assert_eq!(output1.load_decisions, output2.load_decisions, 
            "Determinism violated in case {} - load decisions", i);
        assert_eq!(output2.load_decisions, output3.load_decisions, 
            "Determinism violated in case {} - load decisions", i);
        
        assert_eq!(output1.unload_decisions, output2.unload_decisions, 
            "Determinism violated in case {} - unload decisions", i);
        assert_eq!(output2.unload_decisions, output3.unload_decisions, 
            "Determinism violated in case {} - unload decisions", i);
        
        assert_eq!(output1.chunks_to_load, output2.chunks_to_load, 
            "Determinism violated in case {} - load lists", i);
        assert_eq!(output2.chunks_to_load, output3.chunks_to_load, 
            "Determinism violated in case {} - load lists", i);
        
        assert_eq!(output1.chunks_to_unload, output2.chunks_to_unload, 
            "Determinism violated in case {} - unload lists", i);
        assert_eq!(output2.chunks_to_unload, output3.chunks_to_unload, 
            "Determinism violated in case {} - unload lists", i);
        
        assert_eq!(output1.budget_saturation, output2.budget_saturation, 
            "Determinism violated in case {} - budget saturation", i);
        assert_eq!(output2.budget_saturation, output3.budget_saturation, 
            "Determinism violated in case {} - budget saturation", i);
    }
}

/// Contract Law 5: Center chunk is loaded when budget covers it in traversal order
#[test]
fn streaming_loads_center_chunk_when_budget_covers_traversal_position() {
    let view_distances = [1, 2, 3, 4, 5];
    let budgets = [5, 9, 16, 25, 50]; // Budgets that reach center chunk in row-major traversal
    
    for &view_distance in &view_distances {
        for &budget in &budgets {
            let input = StreamingInput {
                tick: 0,
                player_position: Some([0.0, 0.0, 0.0]),
                view_distance_chunks: view_distance,
                pending_unload_count: 0,
                residency_budget: budget,
                known_loaded_chunks: Vec::new(),
            };
            
            let output = run_streaming(input);
            let loaded_chunks: std::collections::HashSet<_> = output.chunks_to_load.iter().cloned().collect();
            
            // Calculate center chunk position in row-major traversal
            let grid_size = (2 * view_distance + 1) as usize;
            let center_position = view_distance as usize * grid_size + view_distance as usize;
            
            // LAW: Center chunk must be loaded if budget covers its traversal position
            if budget as usize > center_position {
                assert!(loaded_chunks.contains(&[0, 0]),
                    "Center chunk law violated: budget={}, view_distance={}, center_position={}", 
                    budget, view_distance, center_position);
            }
        }
    }
}

/// Contract Law 6: Output structure is consistent
#[test]
fn streaming_output_structure_is_consistent() {
    let input = StreamingInput {
        tick: 0,
        player_position: Some([0.0, 0.0, 0.0]),
        view_distance_chunks: 2,
        pending_unload_count: 0,
        residency_budget: 16,
        known_loaded_chunks: Vec::new(),
    };
    
    let output = run_streaming(input);
    
    // LAW: Output structure must be internally consistent
    assert_eq!(output.chunks_to_load.len(), output.load_decisions as usize,
        "Output structure inconsistent: load list length != load decisions");
    
    assert_eq!(output.chunks_to_unload.len(), output.unload_decisions as usize,
        "Output structure inconsistent: unload list length != unload decisions");
    
    // LAW: Duration must be reasonable (not NaN, not negative)
    assert!(!output.duration_ms.is_nan(),
        "Output structure inconsistent: duration is NaN");
    assert!(output.duration_ms >= 0.0,
        "Output structure inconsistent: negative duration");
}
