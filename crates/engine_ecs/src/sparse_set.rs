use std::collections::HashMap;

pub struct SparseSet<T> {
    dense_entities: Vec<u64>,
    dense_data: Vec<T>,
    sparse: HashMap<u64, usize>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense_entities: Vec::new(),
            dense_data: Vec::new(),
            sparse: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: u64, value: T) {
        if let Some(&idx) = self.sparse.get(&entity) {
            self.dense_data[idx] = value;
        } else {
            let idx = self.dense_entities.len();
            self.dense_entities.push(entity);
            self.dense_data.push(value);
            self.sparse.insert(entity, idx);
        }
    }

    pub fn remove(&mut self, entity: &u64) -> Option<T> {
        let idx = self.sparse.remove(entity)?;
        let last = self.dense_entities.len() - 1;
        if idx != last {
            let moved_entity = self.dense_entities[last];
            self.sparse.insert(moved_entity, idx);
            self.dense_entities.swap(idx, last);
            self.dense_data.swap(idx, last);
        }
        self.dense_entities.pop();
        self.dense_data.pop()
    }

    pub fn get(&self, entity: &u64) -> Option<&T> {
        self.sparse.get(entity).map(|&idx| &self.dense_data[idx])
    }

    pub fn get_mut(&mut self, entity: &u64) -> Option<&mut T> {
        self.sparse
            .get(entity)
            .copied()
            .map(move |idx| &mut self.dense_data[idx])
    }

    pub fn contains_key(&self, entity: &u64) -> bool {
        self.sparse.contains_key(entity)
    }

    pub fn len(&self) -> usize {
        self.dense_entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense_entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &T)> {
        self.dense_entities.iter().zip(self.dense_data.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut T)> {
        self.dense_entities.iter().zip(self.dense_data.iter_mut())
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.dense_data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.dense_data.iter_mut()
    }

    pub fn data(&self) -> &[T] {
        &self.dense_data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.dense_data
    }

    pub fn entities(&self) -> &[u64] {
        &self.dense_entities
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut set: SparseSet<i32> = SparseSet::new();

        set.insert(1, 100);
        set.insert(2, 200);
        set.insert(3, 300);

        assert_eq!(set.get(&1), Some(&100));
        assert_eq!(set.get(&2), Some(&200));
        assert_eq!(set.get(&3), Some(&300));
        assert_eq!(set.get(&999), None);
    }

    #[test]
    fn test_insert_updates_existing() {
        let mut set: SparseSet<i32> = SparseSet::new();

        set.insert(1, 100);
        set.insert(1, 999); // Update

        assert_eq!(set.get(&1), Some(&999));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut set: SparseSet<i32> = SparseSet::new();

        set.insert(1, 100);
        set.insert(2, 200);
        set.insert(3, 300);

        let removed = set.remove(&2);
        assert_eq!(removed, Some(200));
        assert_eq!(set.get(&2), None);
        assert_eq!(set.len(), 2);

        // Verify other entries still accessible
        assert_eq!(set.get(&1), Some(&100));
        assert_eq!(set.get(&3), Some(&300));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(1, 100);

        let removed = set.remove(&999);
        assert_eq!(removed, None);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_contains_key() {
        let mut set: SparseSet<i32> = SparseSet::new();

        set.insert(1, 100);

        assert!(set.contains_key(&1));
        assert!(!set.contains_key(&2));
    }

    #[test]
    fn test_iteration() {
        let mut set: SparseSet<i32> = SparseSet::new();

        set.insert(1, 100);
        set.insert(2, 200);
        set.insert(3, 300);

        let values: Vec<_> = set.values().copied().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&100));
        assert!(values.contains(&200));
        assert!(values.contains(&300));
    }

    #[test]
    fn test_swap_remove_maintains_integrity() {
        let mut set: SparseSet<i32> = SparseSet::new();

        // Insert many items
        for i in 0..100 {
            set.insert(i, i as i32);
        }

        // Remove middle item
        set.remove(&50);

        // Verify all other items accessible
        for i in 0..100 {
            if i == 50 {
                assert_eq!(set.get(&i), None);
            } else {
                assert_eq!(set.get(&i), Some(&(i as i32)));
            }
        }

        assert_eq!(set.len(), 99);
    }
}
