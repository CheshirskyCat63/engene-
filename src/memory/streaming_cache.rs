//! Phase 12: LRU cache for streaming chunk/entity data.
//!
//! Provides efficient load/unload by distance with memory bounds.

use std::collections::HashMap;
use std::time::Instant;

/// Cache entry with metadata for LRU eviction.
struct CacheEntry<T> {
    data: T,
    last_access: Instant,
    access_count: u32,
    memory_bytes: u64,
}

/// LRU cache for streaming data with memory bounds.
pub struct StreamingCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    max_entries: usize,
    max_memory_bytes: u64,
    current_memory_bytes: u64,
    hits: u64,
    misses: u64,
}

impl<K: std::hash::Hash + Eq + Clone + std::fmt::Debug, V> StreamingCache<K, V> {
    /// Create a new cache with given capacity limits.
    pub fn new(max_entries: usize, max_memory_mb: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Get a value from the cache, updating access time.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_access = Instant::now();
            entry.access_count += 1;
            self.hits += 1;
            Some(&entry.data)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Get a mutable reference to a value.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_access = Instant::now();
            entry.access_count += 1;
            self.hits += 1;
            Some(&mut entry.data)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Insert a value with estimated memory size.
    pub fn insert(&mut self, key: K, value: V, memory_bytes: u64) {
        // Check if we need to evict
        while self.entries.len() >= self.max_entries 
            || (self.current_memory_bytes + memory_bytes > self.max_memory_bytes && !self.entries.is_empty())
        {
            self.evict_lru();
        }

        self.current_memory_bytes += memory_bytes;
        self.entries.insert(key, CacheEntry {
            data: value,
            last_access: Instant::now(),
            access_count: 1,
            memory_bytes,
        });
    }

    /// Remove a value from the cache.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_memory_bytes -= entry.memory_bytes;
            Some(entry.data)
        } else {
            None
        }
    }

    /// Check if cache contains a key.
    pub fn contains(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    /// Get current entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current memory usage in bytes.
    pub fn memory_usage(&self) -> u64 {
        self.current_memory_bytes
    }

    /// Get cache hit ratio.
    pub fn hit_ratio(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }

    /// Evict the least recently used entry.
    fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Find LRU entry
        let lru_key = self.entries
            .iter()
            .min_by_key(|(_, e)| (e.last_access, e.access_count))
            .map(|(k, _)| k.clone());

        if let Some(key) = lru_key {
            self.remove(&key);
        }
    }

    /// Evict entries that haven't been accessed recently.
    pub fn evict_older_than(&mut self, max_age_secs: u64) -> usize {
        let cutoff = Instant::now() - std::time::Duration::from_secs(max_age_secs);
        
        let to_evict: Vec<K> = self.entries
            .iter()
            .filter(|(_, e)| e.last_access < cutoff)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_evict.len();
        for key in to_evict {
            self.remove(&key);
        }
        count
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_memory_bytes = 0;
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.entries.len(),
            max_entries: self.max_entries,
            memory_bytes: self.current_memory_bytes,
            max_memory_bytes: self.max_memory_bytes,
            hits: self.hits,
            misses: self.misses,
            hit_ratio: self.hit_ratio(),
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub max_entries: usize,
    pub memory_bytes: u64,
    pub max_memory_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f32,
}

impl CacheStats {
    /// Print stats to stdout.
    pub fn print(&self) {
        println!("Cache: {}/{} entries, {}/{} MB, hits: {}, misses: {}, ratio: {:.1}%",
            self.entry_count,
            self.max_entries,
            self.memory_bytes / (1024 * 1024),
            self.max_memory_bytes / (1024 * 1024),
            self.hits,
            self.misses,
            self.hit_ratio * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut cache: StreamingCache<i32, String> = StreamingCache::new(10, 1);
        
        cache.insert(1, "one".to_string(), 100);
        cache.insert(2, "two".to_string(), 100);
        
        assert_eq!(cache.get(&1), Some(&"one".to_string()));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache: StreamingCache<i32, String> = StreamingCache::new(3, 1);
        
        cache.insert(1, "one".to_string(), 100);
        cache.insert(2, "two".to_string(), 100);
        cache.insert(3, "three".to_string(), 100);
        
        // Access 1 to make it more recent
        cache.get(&1);
        
        // Insert 4, should evict 2 (LRU)
        cache.insert(4, "four".to_string(), 100);
        
        assert!(cache.contains(&1));
        assert!(!cache.contains(&2)); // Evicted
        assert!(cache.contains(&3));
        assert!(cache.contains(&4));
    }

    #[test]
    fn test_memory_eviction() {
        let mut cache: StreamingCache<i32, String> = StreamingCache::new(100, 1); // 1 MB limit
        
        // Insert 2 MB of data (should evict)
        cache.insert(1, "a".to_string(), 512 * 1024);
        cache.insert(2, "b".to_string(), 512 * 1024);
        
        // Memory is now at 1 MB
        assert_eq!(cache.memory_usage(), 1024 * 1024);
        
        // Insert more, should evict
        cache.insert(3, "c".to_string(), 512 * 1024);
        
        // Should have evicted to make room
        assert!(cache.memory_usage() <= 1024 * 1024);
    }

    #[test]
    fn test_hit_ratio() {
        let mut cache: StreamingCache<i32, String> = StreamingCache::new(10, 1);
        
        cache.insert(1, "one".to_string(), 100);
        
        // 2 hits
        cache.get(&1);
        cache.get(&1);
        
        // 1 miss
        cache.get(&2);
        
        assert!((cache.hit_ratio() - 0.666).abs() < 0.01);
    }
}
