//! Runtime Profile and Quality Contracts
//! 
//! Tests for runtime profiles, quality governor, degradation policies, and performance contracts.
//! Ownership: Core Runtime Team
//! Lane: Contracts
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod runtime_profile_tests {
    use engene::core::runtime_config::{RuntimeConfig, RuntimeProfile};

    #[test]
    fn runtime_profile_headless_has_zero_render_budget() {
        let config = RuntimeConfig::headless();
        
        assert_eq!(config.profile, RuntimeProfile::Headless);
        assert!(config.is_headless());
        
        // Headless profile should have zero render budget
        assert_eq!(config.render_budget_ms, 0.0);
        assert_eq!(config.max_render_threads, 0);
        assert!(!config.rendering_enabled);
        
        // But should still have simulation budget
        assert!(config.simulation_budget_ms > 0.0);
        assert!(config.max_simulation_threads > 0);
    }

    #[test]
    fn runtime_profile_tools_enables_debug_surface_only_where_allowed() {
        let config = RuntimeConfig::tools();
        
        assert_eq!(config.profile, RuntimeProfile::Tools);
        assert!(config.debug_mode_enabled);
        
        // Tools profile should enable debug surfaces
        assert!(config.debug_surface_enabled);
        assert!(config.profiling_enabled);
        assert!(config.hot_reload_enabled);
        
        // But not production features
        assert!(!config.production_optimizations_enabled);
        assert!(config.developer_cheats_enabled);
    }

    #[test]
    fn runtime_profile_low_spec_reduces_quality() {
        let config = RuntimeConfig::low_spec();
        
        assert_eq!(config.profile, RuntimeProfile::LowSpec);
        
        // Low spec should reduce budgets
        assert!(config.render_budget_ms < 16.0); // Less than 60 FPS
        assert!(config.simulation_budget_ms < 8.0);
        assert!(config.max_render_threads <= 2);
        assert!(config.max_simulation_threads <= 2);
        
        // Should disable expensive features
        assert!(!config.cloth_simulation_enabled);
        assert!(!config.advanced_lighting_enabled);
        assert!(config.shadows_quality == engene::core::runtime_config::ShadowQuality::Low);
    }

    #[test]
    fn runtime_profile_high_spec_maximizes_quality() {
        let config = RuntimeConfig::high_spec();
        
        assert_eq!(config.profile, RuntimeProfile::HighSpec);
        
        // High spec should maximize budgets
        assert!(config.render_budget_ms >= 16.0); // 60 FPS or better
        assert!(config.simulation_budget_ms >= 8.0);
        assert!(config.max_render_threads >= 4);
        assert!(config.max_simulation_threads >= 4);
        
        // Should enable all features
        assert!(config.cloth_simulation_enabled);
        assert!(config.advanced_lighting_enabled);
        assert!(config.shadows_quality == engene::core::runtime_config::ShadowQuality::Ultra);
    }

    #[test]
    fn runtime_config_validation_blocks_invalid_combinations() {
        // Headless with rendering enabled should be invalid
        let mut config = RuntimeConfig::headless();
        config.rendering_enabled = true;
        
        assert!(config.validate().is_err(), "headless with rendering should be invalid");
        
        // Tools profile without debug should be invalid
        let mut config = RuntimeConfig::tools();
        config.debug_mode_enabled = false;
        
        assert!(config.validate().is_err(), "tools without debug should be invalid");
    }
}

#[cfg(test)]
mod quality_governor_tests {
    use engene::core::quality_governor::QualityGovernor;

    #[test]
    fn degradation_order_contains_collision_navigation_identity_as_never_cut() {
        let governor = QualityGovernor::new();
        
        let degradation_order = governor.get_degradation_order();
        
        // Critical systems should never be cut
        assert!(degradation_order.never_cut_contains("CollisionSystem"));
        assert!(degradation_order.never_cut_contains("NavigationSystem"));
        assert!(degradation_order.never_cut_contains("IdentitySystem"));
        
        // These should be at the top of priority (never cut)
        let collision_index = degradation_order.get_priority_index("CollisionSystem");
        let navigation_index = degradation_order.get_priority_index("NavigationSystem");
        let identity_index = degradation_order.get_priority_index("IdentitySystem");
        
        assert!(collision_index.is_some());
        assert!(navigation_index.is_some());
        assert!(identity_index.is_some());
        
        // Should be highest priority (lowest index)
        assert!(collision_index.unwrap() < 10);
        assert!(navigation_index.unwrap() < 10);
        assert!(identity_index.unwrap() < 10);
    }

    #[test]
    fn quality_governor_adapts_to_frame_time_pressure() {
        let mut governor = QualityGovernor::new();
        
        // Start with high quality
        assert_eq!(governor.current_quality_level(), engene::core::quality_governor::QualityLevel::High);
        
        // Simulate frame time pressure
        governor.report_frame_time(25.0); // 40 FPS - should trigger degradation
        
        // Should degrade quality
        assert!(governor.current_quality_level() != engene::core::quality_governor::QualityLevel::High);
        
        // Should disable some systems
        assert!(governor.is_system_disabled("AdvancedParticles"));
        assert!(governor.is_system_disabled("ClothSimulation"));
        
        // Critical systems should remain enabled
        assert!(!governor.is_system_disabled("CollisionSystem"));
        assert!(!governor.is_system_disabled("NavigationSystem"));
    }

    #[test]
    fn quality_governor_recovers_when_performance_improves() {
        let mut governor = QualityGovernor::new();
        
        // Force degradation
        governor.report_frame_time(33.0); // 30 FPS - severe pressure
        governor.update_quality();
        
        assert!(governor.current_quality_level() != engene::core::quality_governor::QualityLevel::High);
        
        // Report good performance
        for _ in 0..10 {
            governor.report_frame_time(10.0); // 100 FPS - excellent
        }
        governor.update_quality();
        
        // Should recover quality
        assert_eq!(governor.current_quality_level(), engene::core::quality_governor::QualityLevel::High);
        
        // Systems should be re-enabled
        assert!(!governor.is_system_disabled("AdvancedParticles"));
        assert!(!governor.is_system_disabled("ClothSimulation"));
    }

    #[test]
    fn quality_governor_respects_minimum_quality_floor() {
        let mut governor = QualityGovernor::new();
        
        // Simulate extreme pressure
        for _ in 0..100 {
            governor.report_frame_time(100.0); // 10 FPS - extreme
        }
        governor.update_quality();
        
        // Should not go below minimum quality
        assert!(governor.current_quality_level() != engene::core::quality_governor::QualityLevel::Minimum);
        assert!(governor.current_quality_level() >= engene::core::quality_governor::QualityLevel::Low);
        
        // Even at minimum, critical systems should work
        assert!(!governor.is_system_disabled("CollisionSystem"));
        assert!(!governor.is_system_disabled("NavigationSystem"));
        assert!(!governor.is_system_disabled("IdentitySystem"));
    }
}

#[cfg(test)]
mod runtime_manifest_tests {
    use engene::core::runtime_manifest::RuntimeManifest;

    #[test]
    fn runtime_manifest_cannot_enable_render_in_headless() {
        let mut manifest = RuntimeManifest::headless();
        
        // Headless manifest should have rendering disabled
        assert!(!manifest.feature_enabled("rendering"));
        assert!(!manifest.feature_enabled("graphics"));
        
        // Attempting to enable rendering should fail
        let result = manifest.enable_feature("rendering");
        assert!(result.is_err(), "cannot enable rendering in headless manifest");
        
        // Rendering should still be disabled
        assert!(!manifest.feature_enabled("rendering"));
    }

    #[test]
    fn runtime_manifest_feature_toggle_isolation() {
        let mut manifest = RuntimeManifest::default();
        
        // Features should be independent
        assert!(manifest.feature_enabled("destruction"));
        assert!(manifest.feature_enabled("gore"));
        assert!(!manifest.feature_enabled("replay_capture"));
        
        // Toggle one feature
        manifest.toggle_feature("destruction").unwrap();
        assert!(!manifest.feature_enabled("destruction"));
        assert!(manifest.feature_enabled("gore")); // Should be unaffected
        assert!(!manifest.feature_enabled("replay_capture")); // Should be unaffected
    }

    #[test]
    fn runtime_manifest_validates_feature_dependencies() {
        let mut manifest = RuntimeManifest::default();
        
        // Advanced features should depend on basic ones
        let result = manifest.enable_feature("advanced_lighting");
        assert!(result.is_ok());
        
        // Should automatically enable dependencies
        assert!(manifest.feature_enabled("basic_lighting"));
        assert!(manifest.feature_enabled("rendering"));
        
        // Disabling dependency should disable dependent features
        manifest.disable_feature("basic_lighting").unwrap();
        assert!(!manifest.feature_enabled("advanced_lighting"));
    }

    #[test]
    fn runtime_manifest_persists_across_sessions() {
        let manifest = RuntimeManifest::default();
        
        // Enable some features
        let mut modified = manifest.clone();
        modified.enable_feature("experimental_physics").unwrap();
        modified.disable_feature("gore").unwrap();
        
        // Serialize and deserialize
        let serialized = serde_json::to_string(&modified).unwrap();
        let deserialized: RuntimeManifest = serde_json::from_str(&serialized).unwrap();
        
        // Should preserve state
        assert!(deserialized.feature_enabled("experimental_physics"));
        assert!(!deserialized.feature_enabled("gore"));
        assert_eq!(deserialized.feature_enabled("destruction"), manifest.feature_enabled("destruction"));
    }
}

#[cfg(test)]
mod performance_contract_tests {
    use engene::core::performance_contract::PerformanceContract;

    #[test]
    fn performance_contract_enforces_budget_limits() {
        let contract = PerformanceContract::new(engene::core::runtime_config::RuntimeProfile::Medium);
        
        // Should enforce render budget
        assert!(contract.check_render_budget(15.0).is_ok(), "15ms should be within medium budget");
        assert!(contract.check_render_budget(25.0).is_err(), "25ms should exceed medium budget");
        
        // Should enforce simulation budget
        assert!(contract.check_simulation_budget(5.0).is_ok(), "5ms should be within medium budget");
        assert!(contract.check_simulation_budget(15.0).is_err(), "15ms should exceed medium budget");
    }

    #[test]
    fn performance_contract_tracks_violations() {
        let mut contract = PerformanceContract::new(engene::core::runtime_config::RuntimeProfile::Low);
        
        // Generate some violations
        contract.check_render_budget(50.0);
        contract.check_simulation_budget(25.0);
        contract.check_render_budget(60.0);
        
        let violations = contract.get_violations();
        
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].system_type, engene::core::performance_contract::SystemType::Render);
        assert_eq!(violations[1].system_type, engene::core::performance_contract::SystemType::Simulation);
        assert_eq!(violations[2].system_type, engene::core::performance_contract::SystemType::Render);
        
        // Should track severity
        assert!(violations[0].severity > violations[1].severity);
    }

    #[test]
    fn performance_contract_adapts_to_sustained_violations() {
        let mut contract = PerformanceContract::new(engene::core::runtime_config::RuntimeProfile::High);
        
        // Initial budget should be high
        assert!(contract.get_render_budget() > 16.0);
        
        // Sustained violations should trigger adaptation
        for _ in 0..10 {
            contract.check_render_budget(100.0);
        }
        
        // Budget should be reduced
        let adapted_budget = contract.get_render_budget();
        assert!(adapted_budget < 16.0);
        
        // But should not go below minimum
        assert!(adapted_budget >= 8.0);
    }
}
