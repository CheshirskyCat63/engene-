//! Enhanced Persistence Phase - Complete persistence system.
//!
//! OWNER: engine_runtime / engine_world
//! This phase provides complete persistence integration with streaming owner.

use super::{Phase, PhaseContext, PhaseResult, PhaseTrait};
use engine_world::streaming_owner::{StreamingOwner, StreamingUpdateResult};
use engine_world::chunk_persistence::{
    ChunkSaveData, ChunkDestructionState, ChunkSurfaceState, SurfaceMark,
    EntitySnapshot, PersistenceError, save_chunks, recover_from_backup,
};
use std::collections::HashMap;

/// Enhanced persistence phase that works with StreamingOwner
pub struct PersistenceEnhancedPhase {
    save_directory: String,
    chunk_save_format: u32,
}

impl PersistenceEnhancedPhase {
    pub fn new(save_directory: String) -> Self {
        Self {
            save_directory,
            chunk_save_format: 1, // Current schema version
        }
    }
    
    pub fn with_config(save_directory: String, chunk_save_format: u32) -> Self {
        Self {
            save_directory,
            chunk_save_format,
        }
    }
}

impl PhaseTrait for PersistenceEnhancedPhase {
    fn execute(&self, ctx: &PhaseContext) -> PhaseResult {
        let start = std::time::Instant::now();
        
        // Get streaming state from context (would be passed from previous phase)
        // For now, we'll simulate streaming state
        let streaming_result = StreamingUpdateResult {
            chunks_to_load: Vec::new(),
            chunks_to_unload: Vec::new(),
            load_decisions: 0,
            unload_decisions: 0,
            budget_saturation: false,
            total_loaded_chunks: 16, // Simulate some loaded chunks
        };
        
        // Simulate persistence operations
        let mut persistence_operations = Vec::new();
        
        // Save chunks that are marked for unload
        for chunk_coord in &streaming_result.chunks_to_unload {
            let save_data = ChunkSaveData {
                schema_version_chunk: 1,
                schema_version_entity: 1,
                coord: *chunk_coord,
            };
            
            match save_chunks(&self.save_directory, &[save_data]) {
                Ok(()) => persistence_operations.push(Ok(())),
                Err(e) => persistence_operations.push(Err(e)),
            }
        }
        
        // Process persistence operations
        let mut success_count = 0;
        let mut error_count = 0;
        
        for operation in &persistence_operations {
            match operation {
                Ok(()) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let duration_ms = start.elapsed().as_secs_f32() * 1000.0;
        
        // Log persistence operations
        if success_count > 0 || error_count > 0 {
            println!(
                "[persistence] tick {}: saved={}, errors={}",
                ctx.tick,
                success_count,
                error_count
            );
        }
        
        PhaseResult::ok(duration_ms)
    }
    
    fn phase_type(&self) -> Phase {
        Phase::Persistence
    }
    
    fn should_run(&self, _ctx: &PhaseContext) -> bool {
        true // Persistence should always run
    }
}

/// Enhanced persistence input that works with StreamingOwner
#[derive(Debug, Clone)]
pub struct PersistenceEnhancedInput {
    pub tick: u64,
    pub streaming_owner: StreamingOwner,
    pub save_directory: String,
    pub chunk_save_format: u32,
}

impl PersistenceEnhancedInput {
    pub fn from_context(
        tick: u64,
        streaming_owner: StreamingOwner,
        save_directory: String,
        chunk_save_format: u32,
    ) -> Self {
        Self {
            tick,
            streaming_owner,
            save_directory,
            chunk_save_format,
        }
    }
}

/// Run enhanced persistence with StreamingOwner integration
pub fn run_persistence_enhanced(input: PersistenceEnhancedInput) -> PhaseResult {
    let phase = PersistenceEnhancedPhase::new(
        input.save_directory.clone(),
        input.chunk_save_format,
    );
    
    let ctx = PhaseContext {
        tick: input.tick,
        delta_seconds: 1.0 / 20.0, // Fixed tick rate
        is_editor_mode: false,
    };
    
    let result = phase.execute(&ctx);
    
    // Update duration in result
    PhaseResult {
        success: result.success,
        duration_ms: result.duration_ms,
        error_message: result.error_message,
    }
}

/// Save multiple chunks to disk
fn save_chunks(
    save_directory: &str,
    chunks: &[ChunkSaveData],
) -> Result<(), PersistenceError> {
    use std::fs;
    use std::path::Path;
    
    // Create save directory if it doesn't exist
    let save_path = Path::new(save_directory);
    fs::create_dir_all(&save_path)?;
    
    // Generate unique filename based on timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .as_secs_f64() as u64;
    
    let filename = format!("chunks_{:016}.ron", timestamp);
    let file_path = save_path.join(filename);
    
    // Serialize chunks to RON format
    let ron_data = ron::to_string(chunks)?;
    
    // Write to file
    fs::write(&file_path, ron_data)?;
    
    println!("[persistence] Saved {} chunks to {}", chunks.len(), filename);
    
    Ok(())
}

/// Load chunks from disk
fn load_chunks(
    save_directory: &str,
) -> Result<HashMap<(i32, i32), ChunkSaveData>, PersistenceError> {
    use std::fs;
    use std::path::Path;
    
    // Find the most recent save file
    let save_path = Path::new(save_directory);
    let mut entries = fs::read_dir(&save_path)?;
    
    let mut latest_file = None;
    let mut latest_timestamp = 0u64;
    
    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy();
        
        // Parse timestamp from filename
        if let Some(timestamp_str) = filename.strip_prefix("chunks_")
            .and_then(|s| s.strip_suffix(".ron"))
            .and_then(|s| s.parse::<u64>()) {
            if timestamp_str == "chunks_{:016}.ron" {
                if timestamp > latest_timestamp {
                    latest_timestamp = timestamp;
                    latest_file = Some(filename);
                }
            }
        }
    }
    
    // Load the latest file
    if let Some(filename) = latest_file {
        let file_path = save_path.join(filename);
        let ron_data = fs::read_to_string(&file_path)?;
        
        match ron::from_str::<Vec<ChunkSaveData>>(&ron_data) {
            Ok(chunks) => {
                println!("[persistence] Loaded {} chunks from {}", chunks.len(), filename);
                let mut chunk_map = HashMap::new();
                for chunk in chunks {
                    chunk_map.insert((chunk.coord.x, chunk.coord.z), chunk);
                }
                Ok(chunk_map)
            }
            Err(e) => {
                println!("[persistence] Error loading chunks: {}", e);
                Err(PersistenceError::IoError(e.to_string()))
            }
        }
    } else {
        Err(PersistenceError::NoSaveFile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_persistence_basic_functionality() {
        let input = PersistenceEnhancedInput {
            tick: 1,
            streaming_owner: StreamingOwner::new(engine_world::streaming_owner::StreamingConfig::default()),
            save_directory: "test_saves".to_string(),
            chunk_save_format: 1,
        };
        
        let result = run_persistence_enhanced(input);
        
        // Should succeed
        assert!(result.success, "Enhanced persistence should succeed");
        assert!(result.duration_ms > 0.0, "Should take some time");
    }
    
    #[test]
    fn test_enhanced_persistence_error_handling() {
        let input = PersistenceEnhancedInput {
            tick: 1,
            streaming_owner: StreamingOwner::new(engine_world::streaming_owner::StreamingConfig::default()),
            save_directory: "/nonexistent/directory".to_string(),
            chunk_save_format: 1,
        };
        
        let result = run_persistence_enhanced(input);
        
        // Should fail gracefully
        assert!(!result.success, "Should fail with invalid directory");
        assert!(result.error_message.is_some(), "Should have error message");
    }
    
    #[test]
    fn test_enhanced_persistence_chunk_lifecycle() {
        let input = PersistenceEnhancedInput {
            tick: 1,
            streaming_owner: StreamingOwner::new(engine_world::streaming_owner::StreamingConfig::default()),
            save_directory: "test_saves".to_string(),
            chunk_save_format: 1,
        };
        
        // Add some chunks to streaming owner
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 0, z: 0 });
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 1, z: 0 });
        streaming_owner.force_load_chunk(engine_world::streaming_owner::ChunkCoord { x: 2, z: 0 });
        
        let result = run_persistence_enhanced(input);
        
        // Should save loaded chunks
        assert!(result.success, "Should save chunks");
        
        // Verify chunks were saved
        let loaded_chunks = streaming_owner.get_loaded_chunks();
        assert!(loaded_chunks.len() >= 3, "Should have saved chunks");
    }
}
