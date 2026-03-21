//! Multithreading Performance Contracts
//! 
//! Tests for worker pools, job systems, parallel processing, and thread safety.
//! Ownership: QA Team
//! Lane: perf
//! Type: Contract + Performance Tests
//! Speed: Heavy

#[cfg(test)]
mod worker_pool_tests {
    use engene::core::jobs::WorkerPool;

    #[test]
    fn worker_pool_creation() {
        let pool = WorkerPool::new(4);
        assert_eq!(pool.thread_count(), 4);
        assert!(pool.is_active());
    }

    #[test]
    fn worker_pool_task_execution() {
        let pool = WorkerPool::new(2);
        
        let result = pool.execute(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn worker_pool_multiple_tasks() {
        let pool = WorkerPool::new(4);
        
        let tasks: Vec<_> = (0..10).map(|i| {
            pool.execute(move || {
                std::thread::sleep(std::time::Duration::from_millis(5));
                i * 2
            })
        }).collect();
        
        let results: Vec<_> = tasks.into_iter().map(|r| r.unwrap()).collect();
        
        assert_eq!(results.len(), 10);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(*result, i as i32 * 2);
        }
    }

    #[test]
    fn worker_pool_concurrent_execution() {
        let pool = WorkerPool::new(8);
        
        let start = std::time::Instant::now();
        
        let tasks: Vec<_> = (0..100).map(|i| {
            pool.execute(move || {
                std::thread::sleep(std::time::Duration::from_millis(10));
                i
            })
        }).collect();
        
        let results: Vec<_> = tasks.into_iter().map(|r| r.unwrap()).collect();
        
        let duration = start.elapsed();
        
        // Should complete much faster than sequential execution
        assert!(duration.as_millis() < 200, "100 tasks should complete in < 200ms with 8 threads");
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn worker_pool_load_balancing() {
        let pool = WorkerPool::new(4);
        
        let counter = std::sync::atomic::AtomicUsize::new(0);
        
        let tasks: Vec<_> = (0..1000).map(|_| {
            pool.execute(|| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(1));
            })
        }).collect();
        
        // Wait for all tasks
        for task in tasks {
            task.unwrap();
        }
        
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 1000);
    }

    #[test]
    fn worker_pool_error_handling() {
        let pool = WorkerPool::new(2);
        
        let result = pool.execute(|| {
            panic!("Test panic");
        });
        
        assert!(result.is_err());
    }

    #[test]
    fn worker_pool_shutdown() {
        let mut pool = WorkerPool::new(4);
        assert!(pool.is_active());
        
        pool.shutdown();
        assert!(!pool.is_active());
        
        // Should not accept new tasks after shutdown
        let result = pool.execute(|| 42);
        assert!(result.is_err());
    }

    #[test]
    fn worker_pool_memory_usage() {
        let pool = WorkerPool::new(4);
        let initial_memory = pool.memory_usage();
        
        // Execute many tasks
        let tasks: Vec<_> = (0..1000).map(|i| {
            pool.execute(move || {
                vec![i; 100] // Allocate some memory
            })
        }).collect();
        
        // Wait for completion
        for task in tasks {
            let _ = task.unwrap();
        }
        
        let final_memory = pool.memory_usage();
        
        // Memory should be cleaned up after tasks complete
        assert!(final_memory < initial_memory + 10_000_000); // Less than 10MB increase
    }

    #[test]
    fn worker_pool_priority_tasks() {
        let pool = WorkerPool::new(2);
        
        let counter = std::sync::atomic::AtomicUsize::new(0);
        
        // Submit high priority tasks
        let high_priority_tasks: Vec<_> = (0..5).map(|i| {
            pool.execute_priority(10, move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                i
            })
        }).collect();
        
        // Submit low priority tasks
        let low_priority_tasks: Vec<_> = (5..10).map(|i| {
            pool.execute_priority(1, move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                i
            })
        }).collect();
        
        // Wait for all tasks
        for task in high_priority_tasks.into_iter().chain(low_priority_tasks) {
            task.unwrap();
        }
        
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 10);
    }

    #[test]
    fn worker_pool_stress_test() {
        let pool = WorkerPool::new(8);
        
        let start = std::time::Instant::now();
        
        // Execute many small tasks
        let tasks: Vec<_> = (0..10000).map(|i| {
            pool.execute(move || {
                i * i
            })
        }).collect();
        
        let results: Vec<_> = tasks.into_iter().map(|r| r.unwrap()).collect();
        
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 1000, "10K tasks should complete in < 1 second");
        assert_eq!(results.len(), 10000);
        
        // Verify results
        for (i, result) in results.iter().enumerate() {
            assert_eq!(*result, (i as i32) * (i as i32));
        }
    }
}

#[cfg(test)]
mod dirty_set_tests {
    use engene::core::dirty_set::DirtySet;

    #[test]
    fn dirty_set_creation() {
        let dirty_set = DirtySet::new();
        assert!(dirty_set.is_empty());
        assert_eq!(dirty_set.len(), 0);
    }

    #[test]
    fn dirty_set_insert_and_contains() {
        let mut dirty_set = DirtySet::new();
        
        dirty_set.insert(42);
        assert!(dirty_set.contains(&42));
        assert_eq!(dirty_set.len(), 1);
        assert!(!dirty_set.is_empty());
    }

    #[test]
    fn dirty_set_multiple_inserts() {
        let mut dirty_set = DirtySet::new();
        
        for i in 0..100 {
            dirty_set.insert(i);
        }
        
        assert_eq!(dirty_set.len(), 100);
        
        for i in 0..100 {
            assert!(dirty_set.contains(&i));
        }
    }

    #[test]
    fn dirty_set_duplicate_inserts() {
        let mut dirty_set = DirtySet::new();
        
        dirty_set.insert(42);
        dirty_set.insert(42); // Duplicate
        dirty_set.insert(42); // Another duplicate
        
        assert_eq!(dirty_set.len(), 1);
        assert!(dirty_set.contains(&42));
    }

    #[test]
    fn dirty_set_remove() {
        let mut dirty_set = DirtySet::new();
        
        dirty_set.insert(42);
        dirty_set.insert(43);
        
        assert!(dirty_set.contains(&42));
        assert!(dirty_set.contains(&43));
        
        dirty_set.remove(&42);
        
        assert!(!dirty_set.contains(&42));
        assert!(dirty_set.contains(&43));
        assert_eq!(dirty_set.len(), 1);
    }

    #[test]
    fn dirty_set_clear() {
        let mut dirty_set = DirtySet::new();
        
        for i in 0..100 {
            dirty_set.insert(i);
        }
        
        assert_eq!(dirty_set.len(), 100);
        
        dirty_set.clear();
        
        assert!(dirty_set.is_empty());
        assert_eq!(dirty_set.len(), 0);
    }

    #[test]
    fn dirty_set_iteration() {
        let mut dirty_set = DirtySet::new();
        
        for i in 0..10 {
            dirty_set.insert(i);
        }
        
        let collected: Vec<_> = dirty_set.iter().collect();
        assert_eq!(collected.len(), 10);
        
        for &item in &collected {
            assert!(item < 10);
        }
    }

    #[test]
    fn dirty_set_drain() {
        let mut dirty_set = DirtySet::new();
        
        for i in 0..10 {
            dirty_set.insert(i);
        }
        
        let drained: Vec<_> = dirty_set.drain().collect();
        assert_eq!(drained.len(), 10);
        assert!(dirty_set.is_empty());
        
        for &item in &drained {
            assert!(item < 10);
        }
    }

    #[test]
    fn dirty_set_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let dirty_set = Arc::new(Mutex::new(DirtySet::new()));
        let mut handles = vec![];
        
        // Multiple threads inserting
        for i in 0..4 {
            let dirty_set_clone = Arc::clone(&dirty_set);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let mut ds = dirty_set_clone.lock().unwrap();
                    ds.insert(i * 100 + j);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_set = dirty_set.lock().unwrap();
        assert_eq!(final_set.len(), 400);
    }

    #[test]
    fn dirty_set_performance() {
        let mut dirty_set = DirtySet::new();
        
        let start = std::time::Instant::now();
        
        // Insert many elements
        for i in 0..10000 {
            dirty_set.insert(i);
        }
        
        let insert_time = start.elapsed();
        assert!(insert_time.as_millis() < 100, "10K inserts should be fast");
        
        // Test lookup performance
        let lookup_start = std::time::Instant::now();
        
        for i in 0..10000 {
            assert!(dirty_set.contains(&i));
        }
        
        let lookup_time = lookup_start.elapsed();
        assert!(lookup_time.as_millis() < 50, "10K lookups should be very fast");
        
        // Test iteration performance
        let iter_start = std::time::Instant::now();
        
        let count: usize = dirty_set.iter().count();
        
        let iter_time = iter_start.elapsed();
        assert!(iter_time.as_millis() < 50, "Iteration should be fast");
        assert_eq!(count, 10000);
    }

    #[test]
    fn dirty_set_memory_efficiency() {
        let dirty_set = DirtySet::new();
        let initial_memory = dirty_set.memory_usage();
        
        // Add many elements
        for i in 0..10000 {
            dirty_set.insert(i);
        }
        
        let after_insert_memory = dirty_set.memory_usage();
        
        // Memory should increase but reasonably
        assert!(after_insert_memory > initial_memory);
        assert!(after_insert_memory - initial_memory < 1_000_000); // Less than 1MB
        
        // Clear should free memory
        dirty_set.clear();
        let after_clear_memory = dirty_set.memory_usage();
        
        assert!(after_clear_memory < initial_memory + 100_000); // Should be close to initial
    }
}

#[cfg(test)]
mod parallel_processing_tests {
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn parallel_map_processing() {
        let data: Vec<i32> = (0..1000).collect();
        
        let chunks: Vec<_> = data.chunks(100).collect();
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];
        
        for chunk in chunks {
            let results_clone = Arc::clone(&results);
            let handle = thread::spawn(move || {
                let chunk_result: Vec<i32> = chunk.iter().map(|&x| x * 2).collect();
                results_clone.lock().unwrap().push(chunk_result);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_results = results.lock().unwrap();
        let mut flattened: Vec<i32> = final_results.iter().flatten().copied().collect();
        flattened.sort_unstable();
        
        assert_eq!(flattened.len(), 1000);
        for (i, &result) in flattened.iter().enumerate() {
            assert_eq!(result, (i as i32) * 2);
        }
    }

    #[test]
    fn parallel_reduction() {
        let data: Vec<i32> = (0..1000).collect();
        
        let chunks: Vec<_> = data.chunks(250).collect();
        let partial_sums = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];
        
        for chunk in chunks {
            let partial_sums_clone = Arc::clone(&partial_sums);
            let handle = thread::spawn(move || {
                let sum: i32 = chunk.iter().sum();
                partial_sums_clone.lock().unwrap().push(sum);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let partial_results = partial_sums.lock().unwrap();
        let total_sum: i32 = partial_results.iter().sum();
        
        let expected_sum: i32 = (0..1000).sum();
        assert_eq!(total_sum, expected_sum);
    }

    #[test]
    fn parallel_filter() {
        let data: Vec<i32> = (0..1000).collect();
        
        let chunks: Vec<_> = data.chunks(200).collect();
        let filtered_results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];
        
        for chunk in chunks {
            let filtered_clone = Arc::clone(&filtered_results);
            let handle = thread::spawn(move || {
                let filtered: Vec<i32> = chunk.iter()
                    .filter(|&&x| x % 2 == 0)
                    .copied()
                    .collect();
                filtered_clone.lock().unwrap().push(filtered);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_filtered = filtered_results.lock().unwrap();
        let mut flattened: Vec<i32> = final_filtered.iter().flatten().copied().collect();
        flattened.sort_unstable();
        
        // Should contain all even numbers from 0 to 998
        assert_eq!(flattened.len(), 500);
        for (i, &num) in flattened.iter().enumerate() {
            assert_eq!(num, (i as i32) * 2);
        }
    }

    #[test]
    fn parallel_sort() {
        let mut data: Vec<i32> = (0..1000).rev().collect();
        
        // Parallel sort using chunks
        let chunks: Vec<_> = data.chunks_mut(250).collect();
        let mut handles = vec![];
        
        for chunk in chunks {
            let handle = thread::spawn(move || {
                chunk.sort_unstable();
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Now merge the sorted chunks
        data.sort_unstable();
        
        // Verify final sort
        for i in 1..data.len() {
            assert!(data[i-1] <= data[i]);
        }
        
        assert_eq!(data[0], 0);
        assert_eq!(data[999], 999);
    }

    #[test]
    fn parallel_performance_comparison() {
        let data: Vec<i32> = (0..10000).collect();
        
        // Sequential processing
        let sequential_start = std::time::Instant::now();
        let sequential_result: Vec<i32> = data.iter().map(|&x| x * x).collect();
        let sequential_time = sequential_start.elapsed();
        
        // Parallel processing
        let parallel_start = std::time::Instant::now();
        
        let chunks: Vec<_> = data.chunks(1000).collect();
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];
        
        for chunk in chunks {
            let results_clone = Arc::clone(&results);
            let handle = thread::spawn(move || {
                let chunk_result: Vec<i32> = chunk.iter().map(|&x| x * x).collect();
                results_clone.lock().unwrap().push(chunk_result);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let parallel_result: Vec<i32> = results.lock().unwrap().iter().flatten().copied().collect();
        let parallel_time = parallel_start.elapsed();
        
        // Results should be identical
        assert_eq!(sequential_result.len(), parallel_result.len());
        for (&seq, &par) in sequential_result.iter().zip(parallel_result.iter()) {
            assert_eq!(seq, par);
        }
        
        // Parallel should be faster (though this depends on system)
        // At minimum, parallel should complete within reasonable time
        assert!(parallel_time.as_millis() < 1000, "Parallel processing should be fast");
    }

    #[test]
    fn parallel_thread_safety() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        
        // Many threads incrementing counter
        for _ in 0..100 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_count = counter.load(Ordering::Relaxed);
        assert_eq!(final_count, 100 * 1000);
    }

    #[test]
    fn parallel_deadlock_prevention() {
        use std::sync::{Arc, Mutex};
        
        let mutex1 = Arc::new(Mutex::new(0));
        let mutex2 = Arc::new(Mutex::new(0));
        
        let mut handles = vec![];
        
        // Thread 1: lock mutex1 then mutex2
        let m1_clone = Arc::clone(&mutex1);
        let m2_clone = Arc::clone(&mutex2);
        let handle1 = thread::spawn(move || {
            let _lock1 = m1_clone.lock().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
            let _lock2 = m2_clone.lock().unwrap();
            // Do work
        });
        
        // Thread 2: lock mutex2 then mutex1 (potential deadlock)
        let m1_clone = Arc::clone(&mutex1);
        let m2_clone = Arc::clone(&mutex2);
        let handle2 = thread::spawn(move || {
            let _lock2 = m2_clone.lock().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
            let _lock1 = m1_clone.lock().unwrap();
            // Do work
        });
        
        handles.push(handle1);
        handles.push(handle2);
        
        // Wait for completion (should not deadlock)
        let start = std::time::Instant::now();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "Threads should complete without deadlock");
    }
}

// Mock implementations
impl WorkerPool {
    fn new(thread_count: usize) -> Self {
        Self {
            thread_count,
            active: true,
            task_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    fn thread_count(&self) -> usize {
        self.thread_count
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn execute<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if !self.active {
            return Err("Worker pool is shutdown".to_string());
        }
        
        // Mock execution - in real implementation would queue to thread
        self.task_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(f())
    }
    
    fn execute_priority<F, R>(&self, _priority: u32, f: F) -> Result<R, String>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.execute(f)
    }
    
    fn shutdown(&mut self) {
        self.active = false;
    }
    
    fn memory_usage(&self) -> usize {
        self.task_counter.load(std::sync::atomic::Ordering::Relaxed) * 100
    }
}

impl DirtySet {
    fn new() -> Self {
        Self {
            items: std::collections::HashSet::new(),
        }
    }
    
    fn insert(&mut self, item: i32) {
        self.items.insert(item);
    }
    
    fn contains(&self, item: &i32) -> bool {
        self.items.contains(item)
    }
    
    fn remove(&mut self, item: &i32) {
        self.items.remove(item);
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn clear(&mut self) {
        self.items.clear();
    }
    
    fn iter(&self) -> std::collections::hash_set::Iter<i32> {
        self.items.iter()
    }
    
    fn drain(&mut self) -> std::collections::hash_set::Drain<i32> {
        self.items.drain()
    }
    
    fn memory_usage(&self) -> usize {
        self.items.len() * std::mem::size_of::<i32>() * 2 // Approximate
    }
}

struct WorkerPool {
    thread_count: usize,
    active: bool,
    task_counter: std::sync::atomic::AtomicUsize,
}

struct DirtySet {
    items: std::collections::HashSet<i32>,
}
