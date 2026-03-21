//! # Streaming Resource Laws
//! 
//! These tests verify RESOURCE BOUNDARY GUARANTEES that must never change.
//! They test budget compliance, caps, and resource management invariants.

use engine_runtime::phase::streaming::{run_streaming, StreamingInput};

/// Resource Law 1: Budget is never exceeded under any circumstances
#[test]
fn streaming_never_exceeds_budget_under_any_circumstances() {
    let budgets = [0, 1, 5, 10, 16, 100];
    let view_distances = [1, 2, 3, 4, 5, 10];
    
    for &budget in &budgets {
        for &view_distance in &view_distances {
            let input = StreamingInput {
                tick: 0,
                player_position: Some([0.0, 0.0, 0.0]),
                view_distance_chunks: view_distance,
                pending_unload_count: 0,
                residency_budget: budget,
                known_loaded_chunks: Vec::new(),
            };
            
            let output = run_streaming(input);
            
            // RESOURCE: Load decisions must never exceed budget
            assert!(output.load_decisions <= budget,
                "Budget law violated: {} > {} for budget={}, view_distance={}", 
                output.load_decisions, budget, budget, view_distance);
            
            // RESOURCE: Budget saturation flag must be mathematically correct
            assert_eq!(output.budget_saturation, output.load_decisions >= budget,
                "Budget saturation flag incorrect for budget={}, view_distance={}", 
                budget, view_distance);
        }
    }
}

/// Resource Law 2: Unload count is strictly capped by pending count
#[test]
fn streaming_unload_count_strictly_capped_by_pending() {
    let pending_counts = [0, 1, 5, 10, 20, 50];
    let known_chunks = vec![
        [0, 0], [1, 0], [0, 1], [-1, 0], [0, -1],
        [5, 5], [6, 5], [5, 6], [7, 7], [8, 8],
        [2, 0], [0, 2], [-2, 0], [0, -2], [3, 3],
        [10, 10], [-10, -10], [15, 0], [0, 15], [-15, -15]
    ];
    
    for &pending_count in &pending_counts {
        let input = StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: 2,
            pending_unload_count: pending_count,
            residency_budget: 16,
            known_loaded_chunks: known_chunks.clone(),
        };
        
        let output = run_streaming(input);
        
        // RESOURCE: Unload decisions must never exceed pending count
        assert!(output.unload_decisions <= pending_count,
            "Unload cap violated: {} > {} for pending={}", 
            output.unload_decisions, pending_count, pending_count);
        
        // RESOURCE: Unload list length must match unload decisions
        assert_eq!(output.chunks_to_unload.len(), output.unload_decisions as usize,
            "Unload list length mismatch for pending={}", pending_count);
    }
}

/// Resource Law 3: Known chunks exclusion reduces effective load requirements
#[test]
fn streaming_known_chunks_reduces_effective_load_requirements() {
    let base_scenarios = vec![
        // Scenario 1: Small view distance
        (2, vec![[0, 0], [1, 0], [0, 1]]),
        // Scenario 2: Medium view distance  
        (3, vec![[0, 0], [1, 1], [-1, -1], [2, 0]]),
        // Scenario 3: Large view distance
        (5, vec![[0, 0], [2, 2], [-2, -2], [3, 1], [-3, -1]]),
    ];
    
    for (view_distance, known_chunks) in base_scenarios {
        let budget = 50; // Large enough to not be limiting
        
        // Case A: No known chunks
        let input_a = StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: view_distance,
            pending_unload_count: 0,
            residency_budget: budget,
            known_loaded_chunks: Vec::new(),
        };
        
        // Case B: With known chunks
        let input_b = StreamingInput {
            tick: 0,
            player_position: Some([0.0, 0.0, 0.0]),
            view_distance_chunks: view_distance,
            pending_unload_count: 0,
            residency_budget: budget,
            known_loaded_chunks: known_chunks.clone(),
        };
        
        let output_a = run_streaming(input_a);
        let output_b = run_streaming(input_b);
        
        // RESOURCE: Known chunks should reduce or equal load decisions
        assert!(output_b.load_decisions <= output_a.load_decisions,
            "Known chunks should reduce load requirements: view_distance={}, known={:?}",
            view_distance, known_chunks);
        
        // RESOURCE: Known chunks must not appear in load list
        let loaded_chunks_b: std::collections::HashSet<_> = output_b.chunks_to_load.iter().cloned().collect();
        for known_chunk in &known_chunks {
            assert!(!loaded_chunks_b.contains(known_chunk),
                "Known chunk {:?} appeared in load list for view_distance={}",
                known_chunk, view_distance);
        }
    }
}

/// Resource Law 4: Budget truncation is mathematically precise
#[test]
fn streaming_budget_truncation_is_mathematically_precise() {
    let scenarios = vec![
        // (view_distance, expected_total_chunks)
        (1, 9),   // 3x3 grid
        (2, 25),  // 5x5 grid  
        (3, 49),  // 7x7 grid
        (4, 81),  // 9x9 grid
        (5, 121), // 11x11 grid
    ];
    
    for (view_distance, total_chunks) in scenarios {
        let budgets = [0, 1, 5, 10, 16, 25, 50, 100];
        
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
            
            // RESOURCE: Load decisions must be exact min(total, budget)
            let expected_load_decisions = std::cmp::min(total_chunks, budget);
            assert_eq!(output.load_decisions, expected_load_decisions,
                "Budget truncation imprecise: view_distance={}, total={}, budget={}, expected={}, actual={}",
                view_distance, total_chunks, budget, expected_load_decisions, output.load_decisions);
            
            // RESOURCE: Load list length must match load decisions
            assert_eq!(output.chunks_to_load.len(), expected_load_decisions as usize,
                "Load list length mismatch for view_distance={}, budget={}",
                view_distance, budget);
        }
    }
}

/// Resource Law 5: Resource usage is monotonic with budget increases
#[test]
fn streaming_resource_usage_monotonic_with_budget() {
    let view_distance = 3; // 7x7 = 49 chunks total
    let budgets = [0, 5, 10, 16, 25, 49, 100];
    
    let mut previous_load_decisions = 0;
    
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
        
        // RESOURCE: Load decisions should be monotonic (non-decreasing) with budget
        assert!(output.load_decisions >= previous_load_decisions,
            "Resource usage not monotonic: budget={} gave {} < previous budget {} gave {}",
            budget, output.load_decisions, budgets[0], previous_load_decisions);
        
        // RESOURCE: Load decisions should never exceed total available chunks
        assert!(output.load_decisions <= 49, // 7x7 grid
            "Load decisions exceed total available: {} > 49", output.load_decisions);
        
        previous_load_decisions = output.load_decisions;
    }
}
