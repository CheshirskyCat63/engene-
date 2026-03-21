//! Streaming Owner Integration Tests - CONTRACT lane
//!
//! Tests for complete streaming owner integration with canonical phase runner.

use engine_runtime::phase_runner::PhaseRunner;
use engine_runtime::phase::streaming_enhanced::{
    StreamingEnhancedPhase, StreamingEnhancedInput, run_streaming_enhanced,
};
use engine_world::streaming_owner::{StreamingConfig, StreamingOwner};

/// STREAMING OWNER INTEGRATION: Complete streaming system works with phase runner
#[test]
fn test_streaming_owner_integration_with_phase_runner() {
    let config = StreamingConfig {
        max_loaded_chunks: 16,
        load_distance_chunks: 4,
        unload_distance_chunks: 8,
        async_load_batch_size: 4,
        priority_distance_weight: 1.0,
    };
    
    // Create phase runner with enhanced streaming
    let mut runner = PhaseRunner::new();
    
    // Override streaming phase with enhanced version
    let enhanced_phases: Vec<Box<dyn engine_runtime::phase::PhaseTrait + Send + Sync>> = vec![
        Box::new(StreamingEnhancedPhase::new(config)),
    ];
    
    // Execute custom phase list
    let result = runner.execute_custom_phases(&enhanced_phases);
    
    // Should succeed
    assert!(result.success, "Enhanced streaming should succeed");
    assert!(result.duration_ms > 0.0, "Should take some time");
    assert_eq!(runner.tick(), 1, "Tick should advance");
}

/// STREAMING OWNER STATE MANAGEMENT: Proper state tracking across ticks
#[test]
fn test_streaming_owner_state_persistence() {
    let config = StreamingConfig::default();
    let mut owner = StreamingOwner::new(config);
    
    // Simulate multiple ticks
    for tick in 1..=5 {
        let player_pos = if tick % 2 == 0 {
            Some([tick as f32 * 10.0, 0.0, 0.0])
        } else {
            Some([0.0, 0.0, tick as f32 * 10.0])
        };
        
        let result = owner.update(tick, player_pos);
        
        if tick == 1 {
            // First tick - should load chunks
            assert!(result.load_decisions > 0, "Should load chunks on first tick");
            assert_eq!(result.total_loaded_chunks, result.load_decisions as usize, "Loaded chunks should match load decisions");
        } else if tick == 3 {
            // Player moved - should load new chunks and unload old ones
            assert!(result.load_decisions > 0, "Should load new chunks when player moves");
            assert!(result.unload_decisions > 0, "Should unload old chunks when player moves");
        }
    }
    
    // Final state should have chunks loaded
    let final_state = owner.get_loaded_chunks();
    assert!(!final_state.is_empty(), "Should have loaded chunks");
}

/// STREAMING OWNER BUDGET ENFORCEMENT: Proper resource limits
#[test]
fn test_streaming_owner_budget_enforcement() {
    let config = StreamingConfig {
        max_loaded_chunks: 4,
        load_distance_chunks: 10,
        unload_distance_chunks: 12,
        async_load_batch_size: 2,
        priority_distance_weight: 1.0,
    };
    
    let mut owner = StreamingOwner::new(config);
    
    // Load many chunks beyond budget
    for i in 0..8 {
        owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: i, z: 0 });
    }
    
    let result = owner.update(1, Some([0.0, 0.0, 0.0]));
    
    // Should respect budget limit
    assert!(result.total_loaded_chunks <= 4, "Should not exceed budget");
    assert!(result.budget_saturation, "Budget should be saturated");
    assert!(result.load_decisions <= 2, "Load decisions should be limited by batch size");
}

/// STREAMING OWNER PRIORITY SYSTEM: Proper load ordering
#[test]
fn test_streaming_owner_priority_system() {
    let config = StreamingConfig::default();
    let mut owner = StreamingOwner::new(config);
    
    // Add chunks at different distances
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 0, z: 0 }); // Distance 0
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 2, z: 0 }); // Distance 2
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 4, z: 0 }); // Distance 4
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 6, z: 0 }); // Distance 6
    
    let result = owner.update(1, Some([0.0, 0.0, 0.0]));
    
    // Should prioritize closer chunks
    assert!(result.load_decisions >= 2, "Should load at least batch size chunks");
    assert!(result.load_decisions <= 4, "Should not exceed batch size");
    
    // Check that closer chunks have higher priority
    let loaded_chunks = owner.get_loaded_chunks();
    assert!(loaded_chunks.len() >= 2, "Should have loaded chunks");
    
    // Verify priority ordering (closer chunks should be loaded first)
    let mut distances = Vec::new();
    for chunk in &loaded_chunks {
        distances.push(owner.get_chunk(chunk).unwrap().load_priority);
    }
    
    // Sort to check priority order (lower distance = higher priority)
    distances.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    
    // First chunks should have highest priority (lowest distance)
    for i in 0..2.min(distances.len()) {
        assert!(distances[i] >= distances[0], "Priority should be properly ordered");
    }
}

/// STREAMING OWNER EDITOR MODE: Proper behavior in editor
#[test]
fn test_streaming_owner_editor_mode() {
    let config = StreamingConfig::default();
    let mut owner = StreamingOwner::new(config);
    
    // Load some chunks
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 0, z: 0 });
    owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 1, z: 0 });
    
    // Update without player position (editor mode)
    let result = owner.update(1, None);
    
    // Should unload chunks in editor mode
    assert!(result.unload_decisions > 0, "Should unload chunks in editor mode");
    assert!(result.load_decisions == 0, "Should not load chunks in editor mode");
    assert!(!result.budget_saturation, "Budget should not be saturated when unloading");
}

/// STREAMING OWNER PERFORMANCE: Efficient execution
#[test]
fn test_streaming_owner_performance() {
    let config = StreamingConfig::default();
    let mut owner = StreamingOwner::new(config);
    
    let start = std::time::Instant::now();
    
    // Simulate many ticks
    for tick in 1..=100 {
        let player_pos = Some([(tick % 10) as f32, 0.0, (tick % 20) as f32]);
        let _ = owner.update(tick, player_pos);
    }
    
    let duration = start.elapsed();
    
    // Should complete quickly
    assert!(duration.as_millis() < 100, "100 ticks should complete in under 100ms");
    
    // Should handle many operations efficiently
    let final_state = owner.get_loaded_chunks();
    assert!(final_state.len() <= config.max_loaded_chunks, "Should not exceed budget");
}
