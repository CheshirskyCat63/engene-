//! Deterministic merge infrastructure for parallel system outputs.
//!
//! When systems run in parallel, each produces thread-local outputs
//! (events, commands, dirty marks). This module merges them in a
//! deterministic order to ensure replay consistency.

use std::collections::BTreeMap;

/// Thread-local output buffer from a parallel system execution.
#[derive(Debug, Default)]
pub struct SystemOutput {
    pub system_name: String,
    pub events: Vec<DeferredEvent>,
    pub spawn_requests: Vec<SpawnRequest>,
    pub despawn_requests: Vec<u64>,
    pub dirty_chunks: Vec<(i32, i32)>,
    pub dirty_entities: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct DeferredEvent {
    pub type_name: String,
    pub source_system: String,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct SpawnRequest {
    pub kind: String,
    pub position: [f32; 3],
}

/// Merges multiple system outputs in deterministic order.
/// Systems are sorted by name, and within each system, events/commands
/// maintain their original ordering.
pub struct DeterministicMerger {
    outputs: BTreeMap<String, SystemOutput>,
}

impl DeterministicMerger {
    pub fn new() -> Self {
        Self {
            outputs: BTreeMap::new(),
        }
    }

    pub fn submit(&mut self, output: SystemOutput) {
        self.outputs.insert(output.system_name.clone(), output);
    }

    pub fn merge(self) -> MergedOutput {
        let mut merged = MergedOutput::default();

        for (_name, output) in self.outputs {
            merged.events.extend(output.events);
            merged.spawn_requests.extend(output.spawn_requests);
            merged.despawn_requests.extend(output.despawn_requests);
            merged.dirty_chunks.extend(output.dirty_chunks);
            merged.dirty_entities.extend(output.dirty_entities);
        }

        merged.despawn_requests.sort();
        merged.despawn_requests.dedup();
        merged.dirty_chunks.sort();
        merged.dirty_chunks.dedup();
        merged.dirty_entities.sort();
        merged.dirty_entities.dedup();

        merged
    }

    pub fn system_count(&self) -> usize {
        self.outputs.len()
    }
}

impl Default for DeterministicMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct MergedOutput {
    pub events: Vec<DeferredEvent>,
    pub spawn_requests: Vec<SpawnRequest>,
    pub despawn_requests: Vec<u64>,
    pub dirty_chunks: Vec<(i32, i32)>,
    pub dirty_entities: Vec<u64>,
}

impl MergedOutput {
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn spawn_count(&self) -> usize {
        self.spawn_requests.len()
    }

    pub fn despawn_count(&self) -> usize {
        self.despawn_requests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
            && self.spawn_requests.is_empty()
            && self.despawn_requests.is_empty()
    }
}

/// Validates that a merge produced consistent results by checking
/// for conflicting operations on the same entity.
pub fn validate_merge(output: &MergedOutput) -> MergeValidation {
    let mut conflicts = Vec::new();

    for &entity in &output.despawn_requests {
        let spawns_same = output
            .spawn_requests
            .iter()
            .any(|s| s.kind == format!("entity_{}", entity));
        if spawns_same {
            conflicts.push(format!(
                "Entity {} both spawned and despawned in same frame",
                entity
            ));
        }
    }

    MergeValidation {
        is_valid: conflicts.is_empty(),
        conflicts,
    }
}

#[derive(Debug)]
pub struct MergeValidation {
    pub is_valid: bool,
    pub conflicts: Vec<String>,
}
