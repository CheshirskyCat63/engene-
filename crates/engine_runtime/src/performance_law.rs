//! Performance Law - Engine performance contracts and enforcement.
//!
//! OWNER: engine_runtime
//! This module defines performance contracts that all engine phases must follow.

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Performance law that governs all engine operations
pub struct PerformanceLaw {
    /// Maximum allowed time per phase (in milliseconds)
    pub max_phase_time_ms: f64,
    
    /// Maximum allowed memory usage (in MB)
    pub max_memory_mb: usize,
    
    /// Maximum allowed CPU usage (percentage)
    pub max_cpu_percent: f64,
    
    /// Performance budget for streaming operations
    pub streaming_budget: StreamingPerformanceBudget,
    
    /// Performance budget for persistence operations
    pub persistence_budget: PersistencePerformanceBudget,
    
    /// Performance budget for rendering operations
    pub render_budget: RenderPerformanceBudget,
}

impl Default for PerformanceLaw {
    fn default() -> Self {
        Self {
            max_phase_time_ms: 16.67, // 60 FPS = 16.67ms per frame
            max_memory_mb: 2048, // 2GB limit
            max_cpu_percent: 80.0, // Leave 20% for system
            streaming_budget: StreamingPerformanceBudget::default(),
            persistence_budget: PersistencePerformanceBudget::default(),
            render_budget: RenderPerformanceBudget::default(),
        }
    }
}

/// Streaming performance budget
#[derive(Debug, Clone)]
pub struct StreamingPerformanceBudget {
    /// Maximum chunks to load per frame
    pub max_chunks_per_frame: usize,
    
    /// Maximum time for load operations (in ms)
    pub max_load_time_ms: f64,
    
    /// Maximum time for unload operations (in ms)
    pub max_unload_time_ms: f64,
    
    /// Maximum memory for streaming (in MB)
    pub max_memory_mb: usize,
}

impl Default for StreamingPerformanceBudget {
    fn default() -> Self {
        Self {
            max_chunks_per_frame: 4,
            max_load_time_ms: 2.0,
            max_unload_time_ms: 1.0,
            max_memory_mb: 512, // 512MB for streaming
        }
    }
}

/// Persistence performance budget
#[derive(Debug, Clone)]
pub struct PersistencePerformanceBudget {
    /// Maximum chunks to save per frame
    pub max_chunks_per_frame: usize,
    
    /// Maximum time for save operations (in ms)
    pub max_save_time_ms: f64,
    
    /// Maximum time for load operations (in ms)
    pub max_load_time_ms: f64,
    
    /// Maximum I/O bandwidth (in MB/s)
    pub max_io_bandwidth_mb_per_sec: f64,
}

impl Default for PersistencePerformanceBudget {
    fn default() -> Self {
        Self {
            max_chunks_per_frame: 8,
            max_save_time_ms: 5.0,
            max_load_time_ms: 3.0,
            max_io_bandwidth_mb_per_sec: 10.0, // 10MB/s I/O limit
        }
    }
}

/// Render performance budget
#[derive(Debug, Clone)]
pub struct RenderPerformanceBudget {
    /// Maximum draw calls per frame
    pub max_draw_calls_per_frame: usize,
    
    /// Maximum time for render operations (in ms)
    pub max_render_time_ms: f64,
    
    /// Maximum GPU memory usage (in MB)
    pub max_gpu_memory_mb: usize,
}

impl Default for RenderPerformanceBudget {
    fn default() -> Self {
        Self {
            max_draw_calls_per_frame: 1000,
            max_render_time_ms: 10.0,
            max_gpu_memory_mb: 1024, // 1GB GPU memory
        }
    }
}

/// Performance metrics for a single phase execution
#[derive(Debug, Clone)]
pub struct PhasePerformanceMetrics {
    /// Phase name
    pub phase_name: String,
    
    /// Execution time in milliseconds
    pub execution_time_ms: f64,
    
    /// Memory usage in MB
    pub memory_usage_mb: usize,
    
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    
    /// Whether performance budget was exceeded
    pub budget_exceeded: bool,
    
    /// Number of operations performed
    pub operation_count: usize,
    
    /// Average time per operation
    pub avg_time_per_operation_ms: f64,
}

impl PhasePerformanceMetrics {
    pub fn new(phase_name: String, execution_time_ms: f64) -> Self {
        Self {
            phase_name,
            execution_time_ms,
            memory_usage_mb: 0, // TODO: Implement memory tracking
            cpu_usage_percent: 0.0, // TODO: Implement CPU tracking
            budget_exceeded: false,
            operation_count: 1,
            avg_time_per_operation_ms: execution_time_ms,
        }
    }
    
    pub fn with_operations(
        phase_name: String,
        execution_time_ms: f64,
        operation_count: usize,
    ) -> Self {
        Self {
            phase_name,
            execution_time_ms,
            memory_usage_mb: 0, // TODO: Implement memory tracking
            cpu_usage_percent: 0.0, // TODO: Implement CPU tracking
            budget_exceeded: false,
            operation_count,
            avg_time_per_operation_ms: execution_time_ms / operation_count as f64,
        }
    }
}

/// Performance law enforcer
pub struct PerformanceLawEnforcer {
    law: PerformanceLaw,
    metrics: HashMap<String, Vec<PhasePerformanceMetrics>>,
}

impl PerformanceLawEnforcer {
    pub fn new(law: PerformanceLaw) -> Self {
        Self {
            law,
            metrics: HashMap::new(),
        }
    }
    
    /// Check if a phase execution violates performance law
    pub fn check_phase_performance(
        &self,
        phase_name: &str,
        execution_time_ms: f64,
        operation_count: usize,
    ) -> bool {
        let max_time = self.law.max_phase_time_ms;
        
        // Check time budget
        if execution_time_ms > max_time {
            return true;
        }
        
        // Check average time per operation
        let avg_time = execution_time_ms / operation_count as f64;
        let max_avg_time = max_time / 2.0; // Allow 2 ops per frame max
        
        if avg_time > max_avg_time && operation_count > 2 {
            return true;
        }
        
        false
    }
    
    /// Record performance metrics for a phase
    pub fn record_phase_metrics(
        &mut self,
        phase_name: String,
        metrics: PhasePerformanceMetrics,
    ) {
        let history = self.metrics.entry(phase_name.clone()).or_insert_with(Vec::new);
        history.push(metrics);
        
        // Keep only last 100 metrics per phase
        if history.len() > 100 {
            history.remove(0);
        }
    }
    
    /// Get performance metrics for a phase
    pub fn get_phase_metrics(&self, phase_name: &str) -> Option<&Vec<PhasePerformanceMetrics>> {
        self.metrics.get(phase_name)
    }
    
    /// Check if overall performance is within acceptable limits
    pub fn check_overall_performance(&self) -> bool {
        let mut total_time = 0.0;
        let mut total_operations = 0;
        let mut violations = 0;
        
        for (phase_name, history) in &self.metrics {
            for metrics in history {
                total_time += metrics.execution_time_ms;
                total_operations += metrics.operation_count;
                
                if metrics.budget_exceeded {
                    violations += 1;
                }
            }
        }
        
        // Calculate overall averages
        let avg_time_per_operation = if total_operations > 0 {
            total_time / total_operations as f64
        } else {
            0.0
        };
        
        // Check overall performance law
        let overall_budget_exceeded = violations > 0 || avg_time_per_operation > self.law.max_phase_time_ms / 2.0;
        
        !overall_budget_exceeded
    }
    
    /// Get performance summary report
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let mut total_time = 0.0;
        let mut total_operations = 0;
        let mut total_violations = 0;
        
        for (phase_name, history) in &self.metrics {
            for metrics in history {
                total_time += metrics.execution_time_ms;
                total_operations += metrics.operation_count;
                
                if metrics.budget_exceeded {
                    total_violations += 1;
                }
            }
        }
        
        PerformanceSummary {
            total_execution_time_ms: total_time,
            total_operations,
            total_violations,
            avg_time_per_operation_ms: if total_operations > 0 {
                total_time / total_operations as f64
            } else {
                0.0
            },
            performance_law_compliant: total_violations == 0,
        }
    }
}

/// Performance summary report
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total execution time across all phases
    pub total_execution_time_ms: f64,
    
    /// Total number of operations performed
    pub total_operations,
    
    /// Total number of performance law violations
    pub total_violations,
    
    /// Average time per operation
    pub avg_time_per_operation_ms: f64,
    
    /// Whether overall performance is within acceptable limits
    pub performance_law_compliant: bool,
}

impl PerformanceSummary {
    /// Get FPS based on total execution time
    pub fn get_fps(&self) -> f64 {
        if self.total_execution_time_ms > 0.0 {
            1000.0 / self.avg_time_per_operation_ms
        } else {
            0.0
        }
    }
    
    /// Get performance grade
    pub fn get_grade(&self) -> PerformanceGrade {
        if self.total_violations > 0 {
            PerformanceGrade::Poor
        } else if self.avg_time_per_operation_ms > 20.0 {
            PerformanceGrade::Fair
        } else if self.avg_time_per_operation_ms > 10.0 {
            PerformanceGrade::Good
        } else if self.avg_time_per_operation_ms > 5.0 {
            PerformanceGrade::Excellent
        } else {
            PerformanceGrade::Exceptional
        }
    }
}

/// Performance grade
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceGrade {
    Exceptional, // < 5ms per operation
    Excellent,   // 5-10ms per operation  
    Good,       // 10-20ms per operation
    Fair,        // 20-50ms per operation
    Poor,        // > 50ms per operation
}

/// Performance law configuration for different scenarios
#[derive(Debug, Clone)]
pub enum PerformanceLawProfile {
    /// Strict profile for production (60 FPS target)
    Production,
    
    /// Relaxed profile for development (30 FPS target)
    Development,
    
    /// Custom profile with specific limits
    Custom {
        max_phase_time_ms: f64,
        max_memory_mb: usize,
        max_cpu_percent: f64,
    },
}

impl PerformanceLawProfile {
    pub fn create_law(profile: PerformanceLawProfile) -> PerformanceLaw {
        match profile {
            PerformanceLawProfile::Production => PerformanceLaw::default(),
            PerformanceLawProfile::Development => PerformanceLaw {
                max_phase_time_ms: 33.33, // 30 FPS target
                max_memory_mb: 4096, // 4GB limit
                max_cpu_percent: 90.0, // Leave 10% for system
                streaming_budget: StreamingPerformanceBudget {
                    max_chunks_per_frame: 2,
                    max_load_time_ms: 5.0,
                    max_unload_time_ms: 2.0,
                    max_memory_mb: 256,
                },
                persistence_budget: PersistencePerformanceBudget {
                    max_chunks_per_frame: 4,
                    max_save_time_ms: 10.0,
                    max_load_time_ms: 5.0,
                    max_io_bandwidth_mb_per_sec: 5.0,
                },
                render_budget: RenderPerformanceBudget {
                    max_draw_calls_per_frame: 500,
                    max_render_time_ms: 20.0,
                    max_gpu_memory_mb: 512,
                },
            },
            PerformanceLawProfile::Custom { 
                max_phase_time_ms,
                max_memory_mb,
                max_cpu_percent,
                streaming_budget: StreamingPerformanceBudget::default(),
                persistence_budget: PersistencePerformanceBudget::default(),
                render_budget: RenderPerformanceBudget::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_law_default() {
        let law = PerformanceLaw::default();
        assert_eq!(law.max_phase_time_ms, 16.67);
        assert_eq!(law.max_memory_mb, 2048);
        assert_eq!(law.max_cpu_percent, 80.0);
    }
    
    #[test]
    fn test_performance_law_enforcer() {
        let law = PerformanceLaw::default();
        let mut enforcer = PerformanceLawEnforcer::new(law);
        
        // Test compliant execution
        let compliant = !enforcer.check_phase_performance("test", 10.0, 2);
        assert!(compliant, "Should be compliant");
        
        // Test violation
        let violation = enforcer.check_phase_performance("test", 25.0, 2);
        assert!(violation, "Should detect violation");
        
        // Record metrics
        enforcer.record_phase_metrics("test".to_string(), PhasePerformanceMetrics::new("test".to_string(), 10.0));
        
        let summary = enforcer.get_performance_summary();
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.total_violations, 1);
        assert!(!summary.performance_law_compliant);
    }
    
    #[test]
    fn test_performance_summary() {
        let summary = PerformanceSummary {
            total_execution_time_ms: 100.0,
            total_operations: 10,
            total_violations: 0,
            avg_time_per_operation_ms: 10.0,
            performance_law_compliant: true,
        };
        
        assert_eq!(summary.get_fps(), 100.0);
        assert_eq!(summary.get_grade(), PerformanceGrade::Excellent);
        
        let summary_with_violations = PerformanceSummary {
            total_execution_time_ms: 200.0,
            total_operations: 10,
            total_violations: 2,
            avg_time_per_operation_ms: 20.0,
            performance_law_compliant: false,
        };
        
        assert_eq!(summary_with_violations.get_fps(), 50.0);
        assert_eq!(summary_with_violations.get_grade(), PerformanceGrade::Poor);
    }
    
    #[test]
    fn test_performance_law_profiles() {
        let production_law = PerformanceLawProfile::create_law(PerformanceLawProfile::Production);
        assert_eq!(production_law.max_phase_time_ms, 16.67);
        
        let development_law = PerformanceLawProfile::create_law(PerformanceLawProfile::Development);
        assert_eq!(development_law.max_phase_time_ms, 33.33);
        
        let custom_law = PerformanceLawProfile::create_law(PerformanceLawProfile::Custom {
            max_phase_time_ms: 50.0,
            max_memory_mb: 8192,
            max_cpu_percent: 95.0,
            streaming_budget: StreamingPerformanceBudget::default(),
            persistence_budget: PersistencePerformanceBudget::default(),
            render_budget: RenderPerformanceBudget::default(),
        });
        
        assert_eq!(custom_law.max_phase_time_ms, 50.0);
        assert_eq!(custom_law.max_memory_mb, 8192);
        assert_eq!(custom_law.max_cpu_percent, 95.0);
    }
}
