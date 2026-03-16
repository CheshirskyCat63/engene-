//! Phase 13: Memory budget tracking and accounting.
//!
//! Provides per-subsystem memory tracking, budget enforcement, and leak detection.

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Subsystem identifier for memory tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Subsystem {
    Physics,
    AI,
    World,
    Graphics,
    Audio,
    Content,
    Streaming,
    ECS,
    Other,
}

impl Subsystem {
    /// Get default budget in bytes for this subsystem.
    pub fn default_budget(&self) -> u64 {
        match self {
            Subsystem::Physics => 64 * 1024 * 1024,    // 64 MB
            Subsystem::AI => 32 * 1024 * 1024,         // 32 MB
            Subsystem::World => 128 * 1024 * 1024,     // 128 MB
            Subsystem::Graphics => 512 * 1024 * 1024,  // 512 MB
            Subsystem::Audio => 64 * 1024 * 1024,      // 64 MB
            Subsystem::Content => 256 * 1024 * 1024,   // 256 MB
            Subsystem::Streaming => 128 * 1024 * 1024, // 128 MB
            Subsystem::ECS => 64 * 1024 * 1024,        // 64 MB
            Subsystem::Other => 32 * 1024 * 1024,      // 32 MB
        }
    }

    /// Get subsystem name as string.
    pub fn name(&self) -> &'static str {
        match self {
            Subsystem::Physics => "physics",
            Subsystem::AI => "ai",
            Subsystem::World => "world",
            Subsystem::Graphics => "graphics",
            Subsystem::Audio => "audio",
            Subsystem::Content => "content",
            Subsystem::Streaming => "streaming",
            Subsystem::ECS => "ecs",
            Subsystem::Other => "other",
        }
    }
}

/// Per-subsystem memory statistics.
#[derive(Debug, Default)]
pub struct SubsystemStats {
    pub current_bytes: AtomicU64,
    pub peak_bytes: AtomicU64,
    pub alloc_count: AtomicU64,
    pub free_count: AtomicU64,
    pub budget_bytes: u64,
}

impl SubsystemStats {
    pub fn new(budget: u64) -> Self {
        Self {
            budget_bytes: budget,
            ..Default::default()
        }
    }

    fn record_alloc(&self, bytes: u64) {
        self.current_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
        
        // Update peak
        let current = self.current_bytes.load(Ordering::Relaxed);
        let mut peak = self.peak_bytes.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_bytes.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    fn record_free(&self, bytes: u64) {
        self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
        self.free_count.fetch_add(1, Ordering::Relaxed);
    }

    fn current_usage(&self) -> u64 {
        self.current_bytes.load(Ordering::Relaxed)
    }

    fn is_over_budget(&self) -> bool {
        self.current_usage() > self.budget_bytes
    }

    fn usage_ratio(&self) -> f32 {
        let current = self.current_usage();
        if self.budget_bytes == 0 {
            0.0
        } else {
            current as f32 / self.budget_bytes as f32
        }
    }
}

/// Global memory manager for tracking allocations per subsystem.
pub struct MemoryManager {
    subsystems: RwLock<HashMap<Subsystem, SubsystemStats>>,
    total_allocated: AtomicU64,
    total_freed: AtomicU64,
}

impl MemoryManager {
    /// Create a new memory manager with default budgets.
    pub fn new() -> Self {
        let mut subsystems = HashMap::new();
        for ss in [
            Subsystem::Physics,
            Subsystem::AI,
            Subsystem::World,
            Subsystem::Graphics,
            Subsystem::Audio,
            Subsystem::Content,
            Subsystem::Streaming,
            Subsystem::ECS,
            Subsystem::Other,
        ] {
            subsystems.insert(ss, SubsystemStats::new(ss.default_budget()));
        }
        
        Self {
            subsystems: RwLock::new(subsystems),
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
        }
    }

    /// Record an allocation for a subsystem.
    pub fn record_alloc(&self, subsystem: Subsystem, bytes: u64) {
        if let Some(stats) = self.subsystems.read().get(&subsystem) {
            stats.record_alloc(bytes);
        }
        self.total_allocated.fetch_add(bytes, Ordering::Relaxed);
        
        // Warn if over budget
        if let Some(stats) = self.subsystems.read().get(&subsystem) {
            if stats.is_over_budget() {
                tracing::warn!(
                    "[memory] {} is over budget: {} / {} bytes ({:.1}%)",
                    subsystem.name(),
                    stats.current_usage(),
                    stats.budget_bytes,
                    stats.usage_ratio() * 100.0
                );
            }
        }
    }

    /// Record a deallocation for a subsystem.
    pub fn record_free(&self, subsystem: Subsystem, bytes: u64) {
        if let Some(stats) = self.subsystems.read().get(&subsystem) {
            stats.record_free(bytes);
        }
        self.total_freed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get current memory usage for a subsystem.
    pub fn current_usage(&self, subsystem: Subsystem) -> u64 {
        self.subsystems
            .read()
            .get(&subsystem)
            .map(|s| s.current_usage())
            .unwrap_or(0)
    }

    /// Get total memory usage across all subsystems.
    pub fn total_usage(&self) -> u64 {
        self.total_allocated.load(Ordering::Relaxed) - self.total_freed.load(Ordering::Relaxed)
    }

    /// Check if any subsystem is over budget.
    pub fn has_over_budget(&self) -> bool {
        self.subsystems.read().values().any(|s| s.is_over_budget())
    }

    /// Get list of subsystems over budget.
    pub fn over_budget_subsystems(&self) -> Vec<Subsystem> {
        self.subsystems
            .read()
            .iter()
            .filter(|(_, s)| s.is_over_budget())
            .map(|(ss, _)| *ss)
            .collect()
    }

    /// Generate a memory report.
    pub fn report(&self) -> MemoryReport {
        let subsystems = self.subsystems.read();
        let mut entries = Vec::new();
        
        for (ss, stats) in subsystems.iter() {
            entries.push(SubsystemReport {
                subsystem: *ss,
                current_bytes: stats.current_usage(),
                peak_bytes: stats.peak_bytes.load(Ordering::Relaxed),
                budget_bytes: stats.budget_bytes,
                alloc_count: stats.alloc_count.load(Ordering::Relaxed),
                free_count: stats.free_count.load(Ordering::Relaxed),
            });
        }
        
        MemoryReport {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_freed: self.total_freed.load(Ordering::Relaxed),
            current_total: self.total_usage(),
            subsystems: entries,
        }
    }

    /// Set budget for a subsystem.
    pub fn set_budget(&self, subsystem: Subsystem, bytes: u64) {
        if let Some(stats) = self.subsystems.write().get_mut(&subsystem) {
            stats.budget_bytes = bytes;
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Report for a single subsystem.
#[derive(Debug, Clone)]
pub struct SubsystemReport {
    pub subsystem: Subsystem,
    pub current_bytes: u64,
    pub peak_bytes: u64,
    pub budget_bytes: u64,
    pub alloc_count: u64,
    pub free_count: u64,
}

/// Full memory report.
#[derive(Debug, Clone)]
pub struct MemoryReport {
    pub total_allocated: u64,
    pub total_freed: u64,
    pub current_total: u64,
    pub subsystems: Vec<SubsystemReport>,
}

impl MemoryReport {
    /// Print the report to stdout.
    pub fn print(&self) {
        println!("=== Memory Report ===");
        println!("Total: {} bytes allocated, {} bytes freed, {} bytes current",
            self.total_allocated, self.total_freed, self.current_total);
        println!();
        println!("{:12} {:>12} {:>12} {:>12} {:>8}", 
            "Subsystem", "Current", "Peak", "Budget", "Usage%");
        println!("{}", "-".repeat(60));
        
        for entry in &self.subsystems {
            let usage_pct = if entry.budget_bytes > 0 {
                entry.current_bytes as f64 / entry.budget_bytes as f64 * 100.0
            } else {
                0.0
            };
            println!("{:12} {:>12} {:>12} {:>12} {:>7.1}%",
                entry.subsystem.name(),
                entry.current_bytes,
                entry.peak_bytes,
                entry.budget_bytes,
                usage_pct
            );
        }
    }
}

/// Global memory manager instance.
static MEMORY_MANAGER: std::sync::OnceLock<MemoryManager> = std::sync::OnceLock::new();

/// Get the global memory manager.
pub fn global() -> &'static MemoryManager {
    MEMORY_MANAGER.get_or_init(MemoryManager::new)
}

/// Convenience function to record an allocation.
pub fn record_alloc(subsystem: Subsystem, bytes: u64) {
    global().record_alloc(subsystem, bytes);
}

/// Convenience function to record a deallocation.
pub fn record_free(subsystem: Subsystem, bytes: u64) {
    global().record_free(subsystem, bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tracking() {
        let mm = MemoryManager::new();
        
        mm.record_alloc(Subsystem::Physics, 1024);
        assert_eq!(mm.current_usage(Subsystem::Physics), 1024);
        
        mm.record_alloc(Subsystem::Physics, 2048);
        assert_eq!(mm.current_usage(Subsystem::Physics), 3072);
        
        mm.record_free(Subsystem::Physics, 1024);
        assert_eq!(mm.current_usage(Subsystem::Physics), 2048);
    }

    #[test]
    fn test_budget_warning() {
        let mm = MemoryManager::new();
        mm.set_budget(Subsystem::AI, 100);
        
        mm.record_alloc(Subsystem::AI, 50);
        assert!(!mm.has_over_budget());
        
        mm.record_alloc(Subsystem::AI, 100);
        assert!(mm.has_over_budget());
    }

    #[test]
    fn test_report() {
        let mm = MemoryManager::new();
        mm.record_alloc(Subsystem::Graphics, 1024 * 1024);
        mm.record_alloc(Subsystem::AI, 512 * 1024);
        
        let report = mm.report();
        assert_eq!(report.current_total, 1024 * 1024 + 512 * 1024);
        assert_eq!(report.subsystems.len(), 9);
    }
}
