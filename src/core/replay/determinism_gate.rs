//! Determinism Gate for Parallel Tick (Phase C.1)
//! 
//! Provides snapshot comparison between sequential and parallel tick modes.
//! Used to verify that parallel execution produces identical results.

use std::collections::HashMap;

/// Snapshot of ECS state for comparison
#[derive(Debug, Clone, Default)]
pub struct EcsSnapshot {
    pub tick: u64,
    pub entity_count: usize,
    pub component_hashes: HashMap<String, u64>,
}

impl EcsSnapshot {
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            entity_count: 0,
            component_hashes: HashMap::new(),
        }
    }

    /// Convert snapshot to bytes for comparison
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.tick.to_le_bytes());
        bytes.extend_from_slice(&self.entity_count.to_le_bytes());
        
        let mut sorted_hashes: Vec<_> = self.component_hashes.iter().collect();
        sorted_hashes.sort_by_key(|(k, _)| *k);
        
        for (name, hash) in sorted_hashes {
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&hash.to_le_bytes());
        }
        
        bytes
    }
}

/// Determinism gate that compares sequential vs parallel tick results
pub struct DeterminismGate {
    /// Number of ticks to run for comparison
    comparison_ticks: u64,
    /// Tolerance for minor differences (0 = exact match required)
    tolerance: f32,
    /// Whether last comparison passed
    last_comparison_passed: bool,
    /// Divergence points detected
    divergences: Vec<DivergenceInfo>,
}

#[derive(Debug, Clone)]
pub struct DivergenceInfo {
    pub tick: u64,
    pub component: String,
    pub expected_hash: u64,
    pub actual_hash: u64,
}

impl DeterminismGate {
    pub fn new(comparison_ticks: u64) -> Self {
        Self {
            comparison_ticks,
            tolerance: 0.0,
            last_comparison_passed: false,
            divergences: Vec::new(),
        }
    }

    pub fn with_tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Compare two snapshots and record any differences
    pub fn compare(&mut self, expected: &EcsSnapshot, actual: &EcsSnapshot) -> bool {
        self.divergences.clear();
        
        if expected.tick != actual.tick {
            self.divergences.push(DivergenceInfo {
                tick: expected.tick,
                component: "tick".to_string(),
                expected_hash: expected.tick,
                actual_hash: actual.tick,
            });
        }

        if expected.entity_count != actual.entity_count {
            self.divergences.push(DivergenceInfo {
                tick: expected.tick,
                component: "entity_count".to_string(),
                expected_hash: expected.entity_count as u64,
                actual_hash: actual.entity_count as u64,
            });
        }

        for (name, expected_hash) in &expected.component_hashes {
            let actual_hash = actual.component_hashes.get(name).copied().unwrap_or(0);
            if *expected_hash != actual_hash {
                self.divergences.push(DivergenceInfo {
                    tick: expected.tick,
                    component: name.clone(),
                    expected_hash: *expected_hash,
                    actual_hash,
                });
            }
        }

        self.last_comparison_passed = self.divergences.is_empty();
        self.last_comparison_passed
    }

    /// Check if last comparison passed
    pub fn passed(&self) -> bool {
        self.last_comparison_passed
    }

    /// Get divergences from last comparison
    pub fn divergences(&self) -> &[DivergenceInfo] {
        &self.divergences
    }

    /// Get comparison tick count
    pub fn comparison_ticks(&self) -> u64 {
        self.comparison_ticks
    }

    /// Check if divergences are within tolerance
    pub fn within_tolerance(&self) -> bool {
        if self.tolerance <= 0.0 {
            return self.divergences.is_empty();
        }
        
        let total_components = self.divergences.len();
        let allowed = (total_components as f32 * self.tolerance).ceil() as usize;
        self.divergences.len() <= allowed
    }
}

impl Default for DeterminismGate {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Simple hash function for component data
pub fn hash_component_data(data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_to_bytes() {
        let mut snapshot = EcsSnapshot::new(42);
        snapshot.entity_count = 100;
        snapshot.component_hashes.insert("transforms".to_string(), 12345);
        
        let bytes = snapshot.to_bytes();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_determinism_gate_identical() {
        let mut gate = DeterminismGate::new(100);
        
        let mut expected = EcsSnapshot::new(10);
        expected.entity_count = 50;
        expected.component_hashes.insert("transforms".to_string(), 100);
        
        let actual = expected.clone();
        
        assert!(gate.compare(&expected, &actual));
        assert!(gate.passed());
    }

    #[test]
    fn test_determinism_gate_different() {
        let mut gate = DeterminismGate::new(100);
        
        let mut expected = EcsSnapshot::new(10);
        expected.entity_count = 50;
        expected.component_hashes.insert("transforms".to_string(), 100);
        
        let mut actual = expected.clone();
        actual.component_hashes.insert("transforms".to_string(), 200);
        
        assert!(!gate.compare(&expected, &actual));
        assert!(!gate.passed());
        assert_eq!(gate.divergences().len(), 1);
    }

    #[test]
    fn test_hash_component_data() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";
        
        let hash1 = hash_component_data(data1);
        let hash2 = hash_component_data(data2);
        let hash3 = hash_component_data(data3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
