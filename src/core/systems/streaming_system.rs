
//! Streaming system - handles chunk load/unload and persistence.

use crate::core::engine::Engine;
use crate::world::chunk_persistence::ChunkPersistenceService;
use crate::world::streaming::WorldStreamer;

/// Handles world streaming - loading/unloading chunks based on camera position.
pub struct StreamingSystem {
    streamer: WorldStreamer,
    persistence: ChunkPersistenceService,
}

impl StreamingSystem {
    /// Create new streaming system with given persistence service.
    pub fn new(streamer: WorldStreamer, persistence: ChunkPersistenceService) -> Self {
        Self {
            streamer,
            persistence,
        }
    }

    /// Update streaming based on camera position.
    /// Returns (chunks_loaded, chunks_unloaded) counts.
    pub fn update(&mut self, engine: &mut Engine, cam_x: f32, cam_z: f32) -> (usize, usize) {
        let (to_load, to_unload) = self.streamer.update(cam_x, cam_z);

        for coord in &to_load {
            self.streamer.mark_loaded(*coord);
        }

        let mut loaded_count = 0usize;
        let mut unloaded_count = 0usize;

        if !to_unload.is_empty() || !to_load.is_empty() {
            let tick = engine.ecs.tick;

            for coord in &to_unload {
                let saved = self
                    .persistence
                    .save_and_unload(*coord, &mut engine.ecs, tick);
                if saved > 0 {
                    tracing::debug!(
                        "streamer: unloaded chunk ({},{}) — {} entities saved",
                        coord.x,
                        coord.z,
                        saved
                    );
                    unloaded_count += 1;
                }
            }

            for coord in &to_load {
                let loaded = self
                    .persistence
                    .load_chunk_entities(*coord, &mut engine.ecs);
                if loaded > 0 {
                    tracing::debug!(
                        "streamer: loaded chunk ({},{}) — {} entities restored",
                        coord.x,
                        coord.z,
                        loaded
                    );
                    loaded_count += 1;
                }
            }
        }

        (loaded_count, unloaded_count)
    }

    /// Get reference to streamer.
    pub fn streamer(&self) -> &WorldStreamer {
        &self.streamer
    }

    /// Get mutable reference to streamer.
    pub fn streamer_mut(&mut self) -> &mut WorldStreamer {
        &mut self.streamer
    }

    /// Get reference to persistence service.
    pub fn persistence(&self) -> &ChunkPersistenceService {
        &self.persistence
    }
}
