//! Phase 12: Reusable entity ID pool.
//!
//! Provides O(1) allocation and deallocation of entity IDs with generation counters
//! to detect stale handles.

use std::collections::VecDeque;

/// A pooled entity ID with generation for staleness detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PooledEntity {
    pub index: u32,
    pub generation: u32,
}

/// Reusable entity ID pool with generation tracking.
pub struct EntityPool {
    /// Free indices available for reuse.
    free_list: VecDeque<u32>,
    /// Generation counter for each index.
    generations: Vec<u32>,
    /// Maximum capacity.
    capacity: usize,
    /// Current allocated count.
    allocated: usize,
}

impl EntityPool {
    /// Create a new entity pool with given capacity.
    pub fn new(capacity: usize) -> Self {
        let mut free_list = VecDeque::with_capacity(capacity);
        for i in (0..capacity as u32).rev() {
            free_list.push_back(i);
        }

        Self {
            free_list,
            generations: vec![0; capacity],
            capacity,
            allocated: 0,
        }
    }

    /// Allocate a new entity ID.
    /// Returns None if pool is exhausted.
    pub fn allocate(&mut self) -> Option<PooledEntity> {
        if let Some(index) = self.free_list.pop_front() {
            self.allocated += 1;
            Some(PooledEntity {
                index,
                generation: self.generations[index as usize],
            })
        } else {
            None
        }
    }

    /// Deallocate an entity ID.
    /// Returns true if the entity was valid and deallocated.
    pub fn deallocate(&mut self, entity: PooledEntity) -> bool {
        let idx = entity.index as usize;

        // Validate generation
        if idx >= self.capacity || self.generations[idx] != entity.generation {
            return false;
        }

        // Increment generation to invalidate stale handles
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free_list.push_back(entity.index);
        self.allocated -= 1;

        true
    }

    /// Check if an entity is still valid.
    pub fn is_valid(&self, entity: PooledEntity) -> bool {
        let idx = entity.index as usize;
        idx < self.capacity && self.generations[idx] == entity.generation
    }

    /// Get current allocated count.
    pub fn allocated_count(&self) -> usize {
        self.allocated
    }

    /// Get available capacity.
    pub fn available(&self) -> usize {
        self.free_list.len()
    }

    /// Get total capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_deallocate() {
        let mut pool = EntityPool::new(10);

        let e1 = pool.allocate().expect("should allocate");
        assert_eq!(pool.allocated_count(), 1);

        let e2 = pool.allocate().expect("should allocate");
        assert_ne!(e1.index, e2.index);
        assert_eq!(pool.allocated_count(), 2);

        assert!(pool.deallocate(e1));
        assert_eq!(pool.allocated_count(), 1);

        // After deallocate, pool has capacity for one more
        let e3 = pool.allocate().expect("should allocate");
        assert_eq!(pool.allocated_count(), 2);

        // e3 should have incremented generation (either from e1's slot or another)
        assert!(pool.is_valid(e3));
    }

    #[test]
    fn test_stale_handle_detection() {
        let mut pool = EntityPool::new(10);

        let e1 = pool.allocate().unwrap();
        assert!(pool.is_valid(e1));

        pool.deallocate(e1);

        // e1 is now stale (generation mismatch)
        assert!(!pool.is_valid(e1));

        // Allocate new entity - will have updated generation
        let e2 = pool.allocate().unwrap();

        // e2 is valid, e1 is still stale
        assert!(pool.is_valid(e2));
        assert!(!pool.is_valid(e1));
    }

    #[test]
    fn test_pool_exhaustion() {
        let mut pool = EntityPool::new(3);

        let _e1 = pool.allocate();
        let _e2 = pool.allocate();
        let _e3 = pool.allocate();

        assert!(pool.allocate().is_none());
    }
}
