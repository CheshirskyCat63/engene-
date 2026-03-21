//! Performance Governance Contracts
//! 
//! Tests for quality governor, performance budgets, and adaptive quality systems.
//! Ownership: QA Team
//! Lane: perf
//! Type: Contract + Performance Tests
//! Speed: Heavy

#[cfg(test)]
mod quality_governor_tests {
    use engene::core::quality_governor::{QualityGovernor, PressureLevel};
    use engene::core::runtime_config::{QualityTier, RuntimeConfig};

    #[test]
    fn qg_new_creates_with_frame_budget() {
        let gov = QualityGovernor::new(60);
        assert_eq!(gov.frame_budget_us, 1_000_000 / 60);
    }

    #[test]
    fn qg_new_30fps_budget() {
        let gov = QualityGovernor::new(30);
        assert_eq!(gov.frame_budget_us, 1_000_000 / 30);
    }

    #[test]
    fn qg_new_120fps_budget() {
        let gov = QualityGovernor::new(120);
        assert_eq!(gov.frame_budget_us, 1_000_000 / 120);
    }

    #[test]
    fn qg_initial_pressure_normal() {
        let gov = QualityGovernor::new(60);
        assert_eq!(gov.current_pressure(), PressureLevel::Normal);
    }

    #[test]
    fn qg_pressure_increases_with_slow_frames() {
        let mut gov = QualityGovernor::new(60);
        
        // Simulate slow frames
        for _ in 0..10 {
            gov.record_frame_time(20_000); // 20ms frame (slow for 60fps)
        }
        
        assert!(gov.current_pressure() != PressureLevel::Normal);
    }

    #[test]
    fn qg_pressure_decreases_with_fast_frames() {
        let mut gov = QualityGovernor::new(60);
        
        // First increase pressure
        for _ in 0..10 {
            gov.record_frame_time(20_000);
        }
        
        assert!(gov.current_pressure() != PressureLevel::Normal);
        
        // Then simulate fast frames
        for _ in 0..20 {
            gov.record_frame_time(10_000); // 10ms frame (fast for 60fps)
        }
        
        // Pressure should decrease
        assert!(gov.current_pressure() == PressureLevel::Normal || 
                gov.current_pressure() == PressureLevel::Low);
    }

    #[test]
    fn qg_quality_tier_adjustment() {
        let mut gov = QualityGovernor::new(60);
        
        // Start at high quality
        assert_eq!(gov.current_quality_tier(), QualityTier::High);
        
        // Simulate sustained pressure
        for _ in 0..50 {
            gov.record_frame_time(25_000); // 25ms frames
        }
        
        // Quality should be reduced
        assert!(gov.current_quality_tier() != QualityTier::High);
    }

    #[test]
    fn qg_budget_enforcement() {
        let mut gov = QualityGovernor::new(60);
        let budget = gov.frame_budget_us;
        
        // Record frame within budget
        gov.record_frame_time(budget - 1000);
        assert!(gov.current_pressure() == PressureLevel::Normal);
        
        // Record frame over budget
        gov.record_frame_time(budget + 5000);
        assert!(gov.current_pressure() != PressureLevel::Normal);
    }

    #[test]
    fn qg_adaptive_quality_scaling() {
        let mut gov = QualityGovernor::new(60);
        
        let initial_tier = gov.current_quality_tier();
        
        // Gradually increase pressure
        for pressure_level in 1..5 {
            for _ in 0..10 {
                let frame_time = gov.frame_budget_us * (1 + pressure_level as u32);
                gov.record_frame_time(frame_time);
            }
            
            let current_tier = gov.current_quality_tier();
            assert!(current_tier <= initial_tier);
        }
    }

    #[test]
    fn qg_performance_monitoring() {
        let mut gov = QualityGovernor::new(60);
        
        // Record various frame times
        let frame_times = vec![10_000, 15_000, 20_000, 25_000, 30_000];
        
        for &frame_time in &frame_times {
            gov.record_frame_time(frame_time);
        }
        
        let stats = gov.performance_stats();
        assert!(stats.average_frame_time > 0.0);
        assert!(stats.frame_variance > 0.0);
        assert!(stats.pressure_level != PressureLevel::Normal);
    }

    #[test]
    fn qg_system_degradation_order() {
        let gov = QualityGovernor::new(60);
        
        // Get degradation order for high pressure
        let degradation_order = gov.get_degradation_order(PressureLevel::High);
        
        // Should have systems to disable in order
        assert!(!degradation_order.is_empty());
        
        // Verify order makes sense (less critical first)
        for (i, system) in degradation_order.iter().enumerate() {
            if i > 0 {
                let prev_priority = degradation_order[i-1].priority;
                let current_priority = system.priority;
                assert!(current_priority >= prev_priority);
            }
        }
    }

    #[test]
    fn qg_config_integration() {
        let mut config = RuntimeConfig::default();
        config.quality_tier = QualityTier::Medium;
        
        let gov = QualityGovernor::with_config(config);
        
        assert_eq!(gov.current_quality_tier(), QualityTier::Medium);
        assert!(gov.frame_budget_us > 0);
    }

    #[test]
    fn qg_stress_recovery() {
        let mut gov = QualityGovernor::new(60);
        
        // Simulate extreme stress
        for _ in 0..100 {
            gov.record_frame_time(50_000); // 50ms frames
        }
        
        assert!(gov.current_pressure() == PressureLevel::Critical);
        assert!(gov.current_quality_tier() == QualityTier::Low);
        
        // Simulate recovery
        for _ in 0..200 {
            gov.record_frame_time(8_000); // 8ms frames
        }
        
        // Should recover somewhat
        assert!(gov.current_pressure() != PressureLevel::Critical);
    }

    #[test]
    fn qg_multithreaded_pressure_tracking() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let gov = Arc::new(Mutex::new(QualityGovernor::new(60)));
        let mut handles = vec![];
        
        // Multiple threads recording frame times
        for i in 0..4 {
            let gov_clone = Arc::clone(&gov);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let frame_time = match i {
                        0 => 10_000, // Fast frames
                        1 => 20_000, // Medium frames
                        2 => 30_000, // Slow frames
                        3 => 40_000, // Very slow frames
                        _ => 15_000,
                    };
                    
                    let mut g = gov_clone.lock().unwrap();
                    g.record_frame_time(frame_time);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_gov = gov.lock().unwrap();
        // Should have elevated pressure due to slow frames
        assert!(final_gov.current_pressure() != PressureLevel::Normal);
    }

    #[test]
    fn qg_memory_pressure_integration() {
        let mut gov = QualityGovernor::new(60);
        
        // Simulate memory pressure
        gov.record_memory_usage(500_000_000); // 500MB
        assert!(gov.memory_pressure() == PressureLevel::Normal);
        
        gov.record_memory_usage(2_000_000_000); // 2GB
        assert!(gov.memory_pressure() != PressureLevel::Normal);
        
        // Memory pressure should affect quality
        let quality_with_memory_pressure = gov.current_quality_tier();
        assert!(quality_with_memory_pressure != QualityTier::High);
    }

    #[test]
    fn qg_gpu_pressure_integration() {
        let mut gov = QualityGovernor::new(60);
        
        // Simulate GPU pressure
        gov.record_gpu_usage(0.5); // 50% GPU usage
        assert!(gov.gpu_pressure() == PressureLevel::Normal);
        
        gov.record_gpu_usage(0.95); // 95% GPU usage
        assert!(gov.gpu_pressure() != PressureLevel::Normal);
        
        // GPU pressure should affect quality
        let quality_with_gpu_pressure = gov.current_quality_tier();
        assert!(quality_with_gpu_pressure != QualityTier::High);
    }

    #[test]
    fn qg_dynamic_budget_adjustment() {
        let mut gov = QualityGovernor::new(60);
        let initial_budget = gov.frame_budget_us;
        
        // Simulate consistent frame times over budget
        for _ in 0..50 {
            gov.record_frame_time(initial_budget + 10_000);
        }
        
        let adjusted_budget = gov.frame_budget_us;
        assert!(adjusted_budget > initial_budget);
        
        // Should now have more lenient budget
        let test_frame_time = adjusted_budget - 1000;
        gov.record_frame_time(test_frame_time);
        assert!(gov.current_pressure() == PressureLevel::Normal);
    }
}

#[cfg(test)]
mod budget_registry_tests {
    use engene::core::budget_registry::{BudgetRegistry, BudgetEntry};

    #[test]
    fn budget_registry_default_creation() {
        let registry = BudgetRegistry::default();
        
        // Should have default budgets
        assert!(registry.get_budget("rendering").is_some());
        assert!(registry.get_budget("physics").is_some());
        assert!(registry.get_budget("ai").is_some());
    }

    #[test]
    fn budget_registry_custom_budget() {
        let mut registry = BudgetRegistry::new();
        
        let budget = BudgetEntry {
            name: "test_system".to_string(),
            max_time_us: 1000,
            priority: 1,
            enabled: true,
        };
        
        registry.add_budget(budget);
        
        let retrieved = registry.get_budget("test_system");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().max_time_us, 1000);
    }

    #[test]
    fn budget_registry_enforcement() {
        let mut registry = BudgetRegistry::default();
        
        // Record budget usage
        let result = registry.record_usage("rendering", 500);
        assert!(result.is_ok());
        
        // Exceed budget
        let result = registry.record_usage("rendering", 2000);
        assert!(result.is_err());
    }

    #[test]
    fn budget_registry_priority_handling() {
        let mut registry = BudgetRegistry::new();
        
        // Add budgets with different priorities
        registry.add_budget(BudgetEntry {
            name: "low_priority".to_string(),
            max_time_us: 1000,
            priority: 10,
            enabled: true,
        });
        
        registry.add_budget(BudgetEntry {
            name: "high_priority".to_string(),
            max_time_us: 1000,
            priority: 1,
            enabled: true,
        });
        
        // High priority should be enforced first
        let stats = registry.enforcement_stats();
        assert!(stats.high_priority_enforced > stats.low_priority_enforced);
    }

    #[test]
    fn budget_registry_dynamic_adjustment() {
        let mut registry = BudgetRegistry::default();
        
        let initial_budget = registry.get_budget("rendering").unwrap().max_time_us;
        
        // Simulate consistent overruns
        for _ in 0..10 {
            registry.record_usage("rendering", initial_budget + 1000).ok();
        }
        
        registry.adjust_budgets();
        
        let adjusted_budget = registry.get_budget("rendering").unwrap().max_time_us;
        assert!(adjusted_budget > initial_budget);
    }

    #[test]
    fn budget_registry_system_disabling() {
        let mut registry = BudgetRegistry::default();
        
        // System should be enabled initially
        assert!(registry.get_budget("physics").unwrap().enabled);
        
        // Disable system due to budget pressure
        registry.disable_system("physics");
        assert!(!registry.get_budget("physics").unwrap().enabled);
        
        // Should not record usage for disabled systems
        let result = registry.record_usage("physics", 1000);
        assert!(result.is_err());
    }

    #[test]
    fn budget_registry_aggregate_stats() {
        let mut registry = BudgetRegistry::default();
        
        // Record usage for multiple systems
        registry.record_usage("rendering", 500).ok();
        registry.record_usage("physics", 300).ok();
        registry.record_usage("ai", 200).ok();
        
        let stats = registry.aggregate_stats();
        assert_eq!(stats.total_systems, 3);
        assert_eq!(stats.active_systems, 3);
        assert!(stats.total_usage_us > 0);
    }

    #[test]
    fn budget_registry_performance_tracking() {
        let mut registry = BudgetRegistry::default();
        
        let start = std::time::Instant::now();
        
        // Record many usage events
        for i in 0..1000 {
            let usage = 100 + (i % 500);
            registry.record_usage("rendering", usage).ok();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "1000 budget recordings should be fast");
        
        let stats = registry.performance_stats();
        assert!(stats.average_usage_us > 0.0);
        assert!(stats.usage_variance > 0.0);
    }
}

#[cfg(test)]
mod low_spec_certification_tests {
    use engene::core::perf::low_spec_cert::LowSpecCertifier;
    use engene::core::runtime_config::{QualityTier, RuntimeConfig};

    #[test]
    fn low_spec_certifier_initialization() {
        let certifier = LowSpecCertifier::new();
        
        assert!(certifier.minimum_memory_mb > 0);
        assert!(certifier.minimum_gpu_memory_mb > 0);
        assert!(certifier.minimum_cpu_cores > 0);
    }

    #[test]
    fn low_spec_certifier_memory_check() {
        let certifier = LowSpecCertifier::new();
        
        // High memory system
        assert!(!certifier.is_low_spec_memory(8192)); // 8GB
        
        // Low memory system
        assert!(certifier.is_low_spec_memory(2048)); // 2GB
    }

    #[test]
    fn low_spec_certifier_gpu_check() {
        let certifier = LowSpecCertifier::new();
        
        // Good GPU
        assert!(!certifier.is_low_spec_gpu(4096)); // 4GB VRAM
        
        // Weak GPU
        assert!(certifier.is_low_spec_gpu(512)); // 512MB VRAM
    }

    #[test]
    fn low_spec_certifier_cpu_check() {
        let certifier = LowSpecCertifier::new();
        
        // Good CPU
        assert!(!certifier.is_low_spec_cpu(8)); // 8 cores
        
        // Weak CPU
        assert!(certifier.is_low_spec_cpu(2)); // 2 cores
    }

    #[test]
    fn low_spec_certifier_combined_assessment() {
        let certifier = LowSpecCertifier::new();
        
        // High-end system
        let high_end_result = certifier.assess_system(
            8192, // 8GB RAM
            4096, // 4GB VRAM
            8,     // 8 cores
        );
        assert!(!high_end_result.is_low_spec);
        assert_eq!(high_end_result.recommended_tier, QualityTier::High);
        
        // Low-end system
        let low_end_result = certifier.assess_system(
            2048, // 2GB RAM
            512,  // 512MB VRAM
            2,    // 2 cores
        );
        assert!(low_end_result.is_low_spec);
        assert_eq!(low_end_result.recommended_tier, QualityTier::Low);
    }

    #[test]
    fn low_spec_certifier_config_generation() {
        let certifier = LowSpecCertifier::new();
        
        let assessment = certifier.assess_system(2048, 512, 2);
        let config = certifier.generate_config(&assessment);
        
        assert_eq!(config.quality_tier, QualityTier::Low);
        assert!(config.render_distance < 1000.0);
        assert!(config.shadow_quality < 0.5);
        assert!(config.texture_quality < 0.5);
    }

    #[test]
    fn low_spec_certifier_adaptive_settings() {
        let mut certifier = LowSpecCertifier::new();
        
        // Test borderline system
        let assessment = certifier.assess_system(4096, 1024, 4);
        
        // Should adapt settings based on weakest component
        let config = certifier.generate_config(&assessment);
        
        // Should prioritize performance over quality
        assert!(config.quality_tier <= QualityTier::Medium);
        assert!(config.adaptive_quality_enabled);
    }

    #[test]
    fn low_spec_certifier_performance_validation() {
        let certifier = LowSpecCertifier::new();
        
        let start = std::time::Instant::now();
        
        // Assess many configurations
        for memory in [1024, 2048, 4096, 8192] {
            for gpu in [256, 512, 1024, 2048] {
                for cpu in [2, 4, 6, 8] {
                    certifier.assess_system(memory, gpu, cpu);
                }
            }
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "System assessment should be fast");
    }

    #[test]
    fn low_spec_certifier_edge_cases() {
        let certifier = LowSpecCertifier::new();
        
        // Zero values should be handled gracefully
        let zero_result = certifier.assess_system(0, 0, 0);
        assert!(zero_result.is_low_spec);
        
        // Extremely high values should be handled gracefully
        let high_result = certifier.assess_system(65536, 32768, 64);
        assert!(!high_result.is_low_spec);
        assert_eq!(high_result.recommended_tier, QualityTier::Ultra);
    }
}

#[cfg(test)]
mod performance_telemetry_tests {
    use engene::core::perf::telemetry::Telemetry;

    #[test]
    fn telemetry_initialization() {
        let telemetry = Telemetry::new();
        
        assert!(telemetry.frame_count() == 0);
        assert!(telemetry.total_time() == 0.0);
        assert!(telemetry.is_recording() == false);
    }

    #[test]
    fn telemetry_recording_control() {
        let mut telemetry = Telemetry::new();
        
        // Start recording
        telemetry.start_recording();
        assert!(telemetry.is_recording());
        
        // Stop recording
        telemetry.stop_recording();
        assert!(!telemetry.is_recording());
    }

    #[test]
    fn telemetry_frame_recording() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Record some frames
        for i in 0..100 {
            telemetry.record_frame(i as f32 * 1000.0); // Frame time in microseconds
        }
        
        telemetry.stop_recording();
        
        assert_eq!(telemetry.frame_count(), 100);
        assert!(telemetry.total_time() > 0.0);
    }

    #[test]
    fn telemetry_statistics_calculation() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Record frames with varying times
        let frame_times = vec![10.0, 15.0, 20.0, 25.0, 30.0];
        
        for &frame_time in &frame_times {
            telemetry.record_frame(frame_time);
        }
        
        telemetry.stop_recording();
        
        let stats = telemetry.calculate_statistics();
        assert!(stats.average_frame_time > 0.0);
        assert!(stats.min_frame_time == 10.0);
        assert!(stats.max_frame_time == 30.0);
        assert!(stats.frame_variance > 0.0);
    }

    #[test]
    fn telemetry_performance_profiling() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Profile different systems
        telemetry.start_profile("rendering");
        std::thread::sleep(std::time::Duration::from_millis(1));
        telemetry.end_profile("rendering");
        
        telemetry.start_profile("physics");
        std::thread::sleep(std::time::Duration::from_millis(1));
        telemetry.end_profile("physics");
        
        telemetry.stop_recording();
        
        let profiles = telemetry.get_profiles();
        assert!(profiles.contains_key("rendering"));
        assert!(profiles.contains_key("physics"));
        
        assert!(profiles["rendering"].total_time > 0.0);
        assert!(profiles["physics"].total_time > 0.0);
    }

    #[test]
    fn telemetry_memory_tracking() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Record memory usage
        telemetry.record_memory_usage(100_000_000); // 100MB
        telemetry.record_memory_usage(150_000_000); // 150MB
        telemetry.record_memory_usage(120_000_000); // 120MB
        
        telemetry.stop_recording();
        
        let memory_stats = telemetry.memory_statistics();
        assert!(memory_stats.peak_usage == 150_000_000);
        assert!(memory_stats.average_usage > 0.0);
        assert!(memory_stats.usage_variance > 0.0);
    }

    #[test]
    fn telemetry_export_import() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Record some data
        for i in 0..50 {
            telemetry.record_frame(i as f32 * 1000.0);
            if i % 10 == 0 {
                telemetry.record_memory_usage(i * 1_000_000);
            }
        }
        
        telemetry.stop_recording();
        
        // Export data
        let exported = telemetry.export_data();
        assert!(!exported.is_empty());
        
        // Import data
        let mut new_telemetry = Telemetry::new();
        let import_result = new_telemetry.import_data(&exported);
        assert!(import_result.is_ok());
        
        // Verify imported data
        assert_eq!(new_telemetry.frame_count(), 50);
        assert!(new_telemetry.total_time() > 0.0);
    }

    #[test]
    fn telemetry_real_time_monitoring() {
        let mut telemetry = Telemetry::new();
        telemetry.start_recording();
        
        // Simulate real-time frame recording
        let start = std::time::Instant::now();
        
        for _ in 0..1000 {
            telemetry.record_frame(16.67); // 60 FPS target
            
            // Check real-time stats periodically
            if telemetry.frame_count() % 100 == 0 {
                let stats = telemetry.calculate_statistics();
                assert!(stats.average_frame_time > 0.0);
            }
        }
        
        telemetry.stop_recording();
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "1000 frame recordings should be fast");
    }
}

// Mock implementations
impl QualityGovernor {
    fn new(target_fps: u32) -> Self {
        Self {
            frame_budget_us: 1_000_000 / target_fps,
            current_pressure: PressureLevel::Normal,
            quality_tier: QualityTier::High,
            frame_times: Vec::new(),
            memory_usage: 0,
            gpu_usage: 0.0,
        }
    }
    
    fn with_config(config: RuntimeConfig) -> Self {
        let target_fps = match config.quality_tier {
            QualityTier::Ultra => 120,
            QualityTier::High => 60,
            QualityTier::Medium => 30,
            QualityTier::Low => 15,
        };
        
        Self {
            frame_budget_us: 1_000_000 / target_fps,
            current_pressure: PressureLevel::Normal,
            quality_tier: config.quality_tier,
            frame_times: Vec::new(),
            memory_usage: 0,
            gpu_usage: 0.0,
        }
    }
    
    fn record_frame_time(&mut self, time_us: u32) {
        self.frame_times.push(time_us);
        
        // Keep only recent frame times
        if self.frame_times.len() > 100 {
            self.frame_times.remove(0);
        }
        
        self.update_pressure();
    }
    
    fn current_pressure(&self) -> PressureLevel {
        self.current_pressure
    }
    
    fn current_quality_tier(&self) -> QualityTier {
        self.quality_tier
    }
    
    fn performance_stats(&self) -> PerformanceStats {
        if self.frame_times.is_empty() {
            return PerformanceStats::default();
        }
        
        let sum: u32 = self.frame_times.iter().sum();
        let average = sum as f64 / self.frame_times.len() as f64;
        
        let variance = self.frame_times.iter()
            .map(|&time| {
                let diff = time as f64 - average;
                diff * diff
            })
            .sum::<f64>() / self.frame_times.len() as f64;
        
        PerformanceStats {
            average_frame_time: average,
            frame_variance: variance,
            pressure_level: self.current_pressure,
        }
    }
    
    fn get_degradation_order(&self, pressure: PressureLevel) -> Vec<SystemToDisable> {
        // Mock degradation order
        vec![
            SystemToDisable { name: "particles".to_string(), priority: 10 },
            SystemToDisable { name: "shadows".to_string(), priority: 8 },
            SystemToDisable { name: "post_processing".to_string(), priority: 6 },
            SystemToDisable { name: "reflections".to_string(), priority: 4 },
            SystemToDisable { name: "lighting".to_string(), priority: 2 },
        ]
    }
    
    fn record_memory_usage(&mut self, bytes: u64) {
        self.memory_usage = bytes;
        self.update_pressure();
    }
    
    fn memory_pressure(&self) -> PressureLevel {
        if self.memory_usage > 2_000_000_000 { // 2GB
            PressureLevel::Critical
        } else if self.memory_usage > 1_000_000_000 { // 1GB
            PressureLevel::High
        } else if self.memory_usage > 500_000_000 { // 500MB
            PressureLevel::Medium
        } else {
            PressureLevel::Normal
        }
    }
    
    fn record_gpu_usage(&mut self, usage: f32) {
        self.gpu_usage = usage.clamp(0.0, 1.0);
        self.update_pressure();
    }
    
    fn gpu_pressure(&self) -> PressureLevel {
        if self.gpu_usage > 0.9 {
            PressureLevel::Critical
        } else if self.gpu_usage > 0.8 {
            PressureLevel::High
        } else if self.gpu_usage > 0.6 {
            PressureLevel::Medium
        } else {
            PressureLevel::Normal
        }
    }
    
    fn update_pressure(&mut self) {
        let pressures = vec![
            self.calculate_frame_pressure(),
            self.memory_pressure(),
            self.gpu_pressure(),
        ];
        
        self.current_pressure = pressures.into_iter().max().unwrap_or(PressureLevel::Normal);
        
        // Adjust quality tier based on pressure
        if self.current_pressure == PressureLevel::Critical {
            self.quality_tier = QualityTier::Low;
        } else if self.current_pressure == PressureLevel::High {
            self.quality_tier = QualityTier::Medium;
        }
    }
    
    fn calculate_frame_pressure(&self) -> PressureLevel {
        if self.frame_times.is_empty() {
            return PressureLevel::Normal;
        }
        
        let average = self.frame_times.iter().sum::<u32>() as f64 / self.frame_times.len() as f64;
        let budget_ratio = average / self.frame_budget_us as f64;
        
        if budget_ratio > 2.0 {
            PressureLevel::Critical
        } else if budget_ratio > 1.5 {
            PressureLevel::High
        } else if budget_ratio > 1.2 {
            PressureLevel::Medium
        } else {
            PressureLevel::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PressureLevel {
    Normal,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QualityTier {
    Low,
    Medium,
    High,
    Ultra,
}

struct QualityGovernor {
    frame_budget_us: u32,
    current_pressure: PressureLevel,
    quality_tier: QualityTier,
    frame_times: Vec<u32>,
    memory_usage: u64,
    gpu_usage: f32,
}

#[derive(Debug)]
struct PerformanceStats {
    average_frame_time: f64,
    frame_variance: f64,
    pressure_level: PressureLevel,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            average_frame_time: 0.0,
            frame_variance: 0.0,
            pressure_level: PressureLevel::Normal,
        }
    }
}

#[derive(Debug)]
struct SystemToDisable {
    name: String,
    priority: u32,
}

struct RuntimeConfig {
    quality_tier: QualityTier,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            quality_tier: QualityTier::High,
        }
    }
}
