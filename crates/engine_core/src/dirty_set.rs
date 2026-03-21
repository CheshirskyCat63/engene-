use std::collections::HashSet;

/// Generic dirty-set that tracks which cells/entities/chunks have been modified
/// this frame and need processing. Avoids global scans when only a handful
/// of items changed.
#[derive(Clone, Debug)]
pub struct DirtySet<T: std::hash::Hash + Eq + Clone> {
    current: HashSet<T>,
    total_marks: u64,
}

impl<T: std::hash::Hash + Eq + Clone> DirtySet<T> {
    pub fn new() -> Self {
        Self {
            current: HashSet::new(),
            total_marks: 0,
        }
    }

    pub fn mark(&mut self, item: T) {
        self.current.insert(item);
        self.total_marks += 1;
    }

    pub fn is_dirty(&self, item: &T) -> bool {
        self.current.contains(item)
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.current.drain().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.current.iter()
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    pub fn clear(&mut self) {
        self.current.clear();
    }

    pub fn total_marks(&self) -> u64 {
        self.total_marks
    }

    /// Merge another dirty set into this one (for deterministic parallel merge).
    pub fn merge(&mut self, other: &DirtySet<T>) {
        for item in &other.current {
            self.current.insert(item.clone());
        }
        self.total_marks += other.total_marks;
    }
}

impl<T: std::hash::Hash + Eq + Clone> Default for DirtySet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial dirty set keyed by chunk coordinates.
#[derive(Clone, Debug, Default)]
pub struct ChunkDirtySet(pub DirtySet<(i32, i32)>);

/// Entity-level dirty set keyed by entity ID.
#[derive(Clone, Debug, Default)]
pub struct EntityDirtySet(pub DirtySet<u64>);

/// Nav cell dirty set keyed by grid cell.
#[derive(Clone, Debug, Default)]
pub struct NavDirtySet(pub DirtySet<(i32, i32)>);
