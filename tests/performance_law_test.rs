//! Performance Law Tests - CONTRACT lane
//!
//! Tests for performance law enforcement and compliance.

use engine_runtime::performance_law::{
    PerformanceLaw, PerformanceLawEnforcer, PerformanceSummary, PerformanceGrade,
    StreamingPerformanceBudget, PersistencePerformanceBudget, RenderPerformanceBudget,
    PerformanceLawProfile,
};

/// PERFORMANCE LAW BASIC FUNCTIONALITY: Default configuration works
#[test]
fn test_performance_law_default() {
    let law = PerformanceLaw::default();
    
    // Check default values
    assert_eq!(law.max_phase_time_ms, 16.67);
    assert_eq!(law.max_memory_mb, 2048);
    assert_eq!(law.max_cpu_percent, 80.0);
    
    // Check default budgets
    let streaming_budget = &law.streaming_budget;
    assert_eq!(streaming_budget.max_chunks_per_frame, 4);
    assert_eq!(streaming_budget.max_load_time_ms, 2.0);
    assert_eq!(streaming_budget.max_unload_time_ms, 1.0);
    assert_eq!(streaming_budget.max_memory_mb, 512);
    
    let persistence_budget = &law.persistence_budget;
    assert_eq!(persistence_budget.max_chunks_per_frame, 8);
    assert_eq!(persistence_budget.max_save_time_ms, 5.0);
    assert_eq!(persistence_budget.max_load_time_ms, 3.0);
    assert_eq!(persistence_budget.max_io_bandwidth_mb_per_sec, 10.0);
    
    let render_budget = &law.render_budget;
    assert_eq!(render_budget.max_draw_calls_per_frame, 1000);
    assert_eq!(render_budget.max_render_time_ms, 10.0);
    assert_eq!(render_budget.max_gpu_memory_mb, 1024);
}

/// PERFORMANCE LAW ENFORCER: Violation detection and enforcement
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
    
    // Record compliant execution
    enforcer.record_phase_metrics("test_compliant".to_string(), PhasePerformanceMetrics::new("test_compliant".to_string(), 10.0));
    
    // Record violation execution
    enforcer.record_phase_metrics("test_violation".to_string(), PhasePerformanceMetrics::new("test_violation".to_string(), 25.0));
    
    let summary = enforcer.get_performance_summary();
    assert_eq!(summary.total_operations, 2);
    assert_eq!(summary.total_violations, 1);
    assert!(!summary.performance_law_compliant);
}

/// PERFORMANCE LAW PROFILES: Different performance targets
#[test]
fn test_performance_law_profiles() {
    // Test production profile
    let production_law = PerformanceLawProfile::create_law(PerformanceLawProfile::Production);
    assert_eq!(production_law.max_phase_time_ms, 16.67);
    assert_eq!(production_law.max_memory_mb, 2048);
    assert_eq!(production_law.max_cpu_percent, 80.0);
    
    // Test development profile
    let development_law = PerformanceLawProfile::create_law(PerformanceLawProfile::Development);
    assert_eq!(development_law.max_phase_time_ms, 33.33);
    assert_eq!(development_law.max_memory_mb, 4096);
    assert_eq!(development_law.max_cpu_percent, 90.0);
    
    // Test custom profile
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

/// PERFORMANCE LAW SUMMARY: Performance grading and reporting
#[test]
fn test_performance_summary() {
    let law = PerformanceLaw::default();
    let mut enforcer = PerformanceLawEnforcer::new(law);
    
    // Record excellent performance
    enforcer.record_phase_metrics("excellent_phase".to_string(), PhasePerformanceMetrics::new("excellent_phase".to_string(), 5.0));
    
    // Record good performance
    enforcer.record_phase_metrics("good_phase".to_string(), PhasePerformanceMetrics::new("good_phase".to_string(), 15.0));
    
    // Record fair performance
    enforcer.record_phase_metrics("fair_phase".to_string(), PhasePerformanceMetrics::new("fair_phase".to_string(), 35.0));
    
    // Record poor performance
    enforcer.record_phase_metrics("poor_phase".to_string(), PhasePerformanceMetrics::new("poor_phase".to_string(), 75.0));
    
    let summary = enforcer.get_performance_summary();
    
    // Check overall summary
    assert_eq!(summary.total_operations, 4);
    assert_eq!(summary.total_violations, 0);
    assert!(summary.performance_law_compliant);
    assert_eq!(summary.get_fps(), 1000.0 / (5.0 + 15.0 + 35.0 + 75.0) / 4.0);
    assert_eq!(summary.get_grade(), PerformanceGrade::Excellent);
    
    // Test with violations
    enforcer.record_phase_metrics("violation_phase".to_string(), PhasePerformanceMetrics::new("violation_phase".to_string(), 100.0));
    
    let summary_with_violations = enforcer.get_performance_summary();
    assert_eq!(summary_with_violations.total_operations, 5);
    assert_eq!(summary_with_violations.total_violations, 1);
    assert!(!summary_with_violations.performance_law_compliant);
    assert_eq!(summary_with_violations.get_grade(), PerformanceGrade::Poor);
}

/// PERFORMANCE LAW BUDGET ENFORCEMENT: Streaming and persistence budgets
#[test]
fn test_performance_budget_enforcement() {
    let law = PerformanceLaw::default();
    let mut enforcer = PerformanceLawEnforcer::new(law);
    
    // Test streaming budget enforcement
    let streaming_violation = enforcer.check_phase_performance("streaming", 10.0, 5);
    assert!(streaming_violation, "Should detect streaming budget violation");
    
    // Test persistence budget enforcement
    let persistence_violation = enforcer.check_phase_performance("persistence", 15.0, 3);
    assert!(persistence_violation, "Should detect persistence budget violation");
    
    // Test render budget enforcement
    let render_violation = enforcer.check_phase_performance("render", 25.0, 10);
    assert!(render_violation, "Should detect render budget violation");
    
    // Test combined budget enforcement
    enforcer.record_phase_metrics("streaming".to_string(), PhasePerformanceMetrics::new("streaming".to_string(), 5.0));
    enforcer.record_phase_metrics("persistence".to_string(), PhasePerformanceMetrics::new("persistence".to_string(), 15.0));
    enforcer.record_phase_metrics("render".to_string(), PhasePerformanceMetrics::new("render".to_string(), 25.0));
    
    let summary = enforcer.get_performance_summary();
    assert_eq!(summary.total_operations, 3);
    assert_eq!(summary.total_violations, 3);
    assert!(!summary.performance_law_compliant);
}

/// PERFORMANCE LAW INTEGRATION: Works with phase runner
#[test]
fn test_performance_law_phase_runner_integration() {
    let law = PerformanceLaw::default();
    let mut enforcer = PerformanceLawEnforcer::new(law);
    
    // Simulate phase execution with performance tracking
    let phase_metrics = PhasePerformanceMetrics::new("test_phase".to_string(), 12.0);
    enforcer.record_phase_metrics("test_phase".to_string(), phase_metrics);
    
    // Check compliance
    let is_compliant = enforcer.check_phase_performance("test_phase", 12.0, 1);
    assert!(is_compliant, "Phase execution should be compliant");
    
    // Get summary
    let summary = enforcer.get_performance_summary();
    assert_eq!(summary.total_operations, 1);
    assert_eq!(summary.total_violations, 0);
    assert!(summary.performance_law_compliant);
    assert_eq!(summary.get_grade(), PerformanceGrade::Good);
}

/// PERFORMANCE LAW REAL-WORLD SCENARIO: Multiple phases with budget constraints
#[test]
fn test_performance_law_real_world_scenario() {
    let law = PerformanceLaw::default();
    let mut enforcer = PerformanceLawEnforcer::new(law);
    
    // Simulate realistic phase execution
    let phases = vec![
        ("tick", 8.0),
        ("streaming", 15.0),
        ("persistence", 25.0),
        ("spatial", 12.0),
        ("audio", 5.0),
        ("editor", 3.0),
        ("render", 18.0),
    ];
    
    let mut total_violations = 0;
    
    for (phase_name, execution_time) in phases {
        let is_compliant = enforcer.check_phase_performance(phase_name, execution_time, 1);
        
        if !is_compliant {
            total_violations += 1;
        }
        
        enforcer.record_phase_metrics(phase_name, PhasePerformanceMetrics::new(phase_name, execution_time));
    }
    
    let summary = enforcer.get_performance_summary();
    
    // Check overall results
    assert_eq!(summary.total_operations, phases.len());
    assert_eq!(summary.total_violations, 3);
    assert!(!summary.performance_law_compliant);
    
    // Check specific violations
    let tick_metrics = enforcer.get_phase_metrics("tick").unwrap();
    let streaming_metrics = enforcer.get_phase_metrics("streaming").unwrap();
    let persistence_metrics = enforcer.get_phase_metrics("persistence").unwrap();
    let render_metrics = enforcer.get_phase_metrics("render").unwrap();
    
    assert!(tick_metrics.execution_time_ms <= law.max_phase_time_ms, "Tick should be within budget");
    assert!(streaming_metrics.budget_exceeded, "Streaming should exceed budget");
    assert!(persistence_metrics.budget_exceeded, "Persistence should exceed budget");
    assert!(render_metrics.execution_time_ms > law.max_phase_time_ms, "Render should exceed budget");
}
