//! Enhanced Persistence Integration Tests - CONTRACT lane
//!
//! Tests for complete persistence system integration with streaming owner.

use engine_runtime::phase_runner::PhaseRunner;
use engine_runtime::phase::persistence_enhanced::{
    PersistenceEnhancedPhase, run_persistence_enhanced, PersistenceEnhancedInput,
};
use engine_world::streaming_owner::{StreamingOwner, StreamingConfig};

/// PERSISTENCE ENHANCED INTEGRATION: Complete persistence system works with streaming owner
#[test]
fn test_persistence_enhanced_integration_with_phase_runner() {
    let streaming_config = StreamingConfig::default();
    let streaming_owner = StreamingOwner::new(streaming_config);
    
    // Create phase runner with enhanced persistence
    let mut runner = PhaseRunner::new();
    
    // Override persistence phase with enhanced version
    let enhanced_phases: Vec<Box<dyn engine_runtime::phase::PhaseTrait + Send + Sync>> = vec![
        Box::new(PersistenceEnhancedPhase::new(
            "test_saves".to_string(),
            1, // Current schema version
        )),
    ];
    
    // Execute custom phase list
    let result = runner.execute_custom_phases(&enhanced_phases);
    
    // Should succeed
    assert!(result.success, "Enhanced persistence should succeed");
    assert!(result.duration_ms > 0.0, "Should take some time");
    assert_eq!(runner.tick(), 1, "Tick should advance");
}

/// PERSISTENCE ENHANCED STATE MANAGEMENT: Proper save/load operations
#[test]
fn test_persistence_enhanced_chunk_lifecycle() {
    let streaming_config = StreamingConfig::default();
    let mut streaming_owner = StreamingOwner::new(streaming_config);
    
    // Add some chunks to streaming owner
    streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 0, z: 0 });
    streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 1, z: 0 });
    streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 2, z: 0 });
    
    let input = PersistenceEnhancedInput::from_context(
        1,
        streaming_owner,
        "test_saves".to_string(),
        1,
    );
    
    let result = run_persistence_enhanced(input);
    
    // Should succeed
    assert!(result.success, "Enhanced persistence should succeed");
    assert!(result.duration_ms > 0.0, "Should take some time");
    
    // Verify save directory was created
    let save_path = std::path::Path::new("test_saves");
    assert!(save_path.exists(), "Save directory should be created");
    
    // Verify save files exist
    let entries = std::fs::read_dir("test_saves").unwrap();
    assert!(!entries.is_empty(), "Should have created save files");
}

/// PERSISTENCE ENHANCED ERROR HANDLING: Proper failure recovery
#[test]
fn test_persistence_enhanced_error_handling() {
    let streaming_config = StreamingConfig::default();
    let streaming_owner = StreamingOwner::new(streaming_config);
    
    let input = PersistenceEnhancedInput::from_context(
        1,
        streaming_owner,
        "/invalid/path/that/cannot/be/created".to_string(),
        1,
    );
    
    let result = run_persistence_enhanced(input);
    
    // Should fail gracefully
    assert!(!result.success, "Should fail with invalid path");
    assert!(result.error_message.is_some(), "Should have error message");
}

/// PERSISTENCE ENHANCED SCHEMA VERSIONING: Proper format handling
#[test]
fn test_persistence_enhanced_schema_versioning() {
    let streaming_config = StreamingConfig::default();
    let streaming_owner = StreamingOwner::new(streaming_config);
    
    // Test with different schema versions
    let input_v1 = PersistenceEnhancedInput::from_context(
        1,
        streaming_owner,
        "test_saves".to_string(),
        1, // Version 1
    );
    
    let input_v2 = PersistenceEnhancedInput::from_context(
        2,
        streaming_owner,
        "test_saves".to_string(),
        2, // Version 2
    );
    
    // Both should succeed
    let result_v1 = run_persistence_enhanced(input_v1);
    let result_v2 = run_persistence_enhanced(input_v2);
    
    assert!(result_v1.success, "Version 1 should succeed");
    assert!(result_v2.success, "Version 2 should succeed");
    
    // Verify different files are created
    let entries = std::fs::read_dir("test_saves").unwrap();
    assert!(entries.len() >= 2, "Should have created multiple save files");
}

/// PERSISTENCE ENHANCED PERFORMANCE: Efficient bulk operations
#[test]
fn test_persistence_enhanced_performance() {
    let streaming_config = StreamingConfig::default();
    let mut streaming_owner = StreamingOwner::new(streaming_config);
    
    // Add many chunks for testing
    for i in 0..50 {
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { 
            x: i % 10, 
            z: i / 10 
        });
    }
    
    let start = std::time::Instant::now();
    
    let input = PersistenceEnhancedInput::from_context(
        1,
        streaming_owner,
        "test_saves".to_string(),
        1,
    );
    
    let result = run_persistence_enhanced(input);
    
    let duration = start.elapsed();
    
    // Should complete quickly
    assert!(result.success, "Should succeed");
    assert!(duration.as_millis() < 100, "Should complete in under 100ms");
    
    // Verify all chunks were saved
    let entries = std::fs::read_dir("test_saves").unwrap();
    assert!(!entries.is_empty(), "Should have created save files");
}

/// PERSISTENCE ENHANCED INTEGRATION: Works with streaming owner state
#[test]
fn test_persistence_enhanced_streaming_integration() {
    let streaming_config = StreamingConfig::default();
    let mut streaming_owner = StreamingOwner::new(streaming_config);
    
    // Simulate streaming operations first
    let streaming_result = streaming_owner.update(1, Some([0.0, 0.0, 0.0]));
    
    // Mark some chunks as loaded
    for chunk in &streaming_result.chunks_to_load {
        streaming_owner.mark_chunk_loaded(*chunk, 10);
    }
    
    // Now run persistence
    let input = PersistenceEnhancedInput::from_context(
        1,
        streaming_owner,
        "test_saves".to_string(),
        1,
    );
    
    let result = run_persistence_enhanced(input);
    
    // Should succeed
    assert!(result.success, "Should save loaded chunks");
    
    // Verify persistence state is preserved
    let loaded_chunks = streaming_owner.get_loaded_chunks();
    assert!(loaded_chunks.len() >= streaming_result.load_decisions as usize, 
              "Should preserve loaded chunk count");
}
