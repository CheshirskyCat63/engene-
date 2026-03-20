//! Editor Safe Mode Contracts
//! 
//! Tests for editor safety, panic recovery, and debug surface isolation.
//! Ownership: Tools Team
//! Lane: Contracts
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod editor_safe_mode_tests {
    use engene::tools::editor_safe_mode::EditorSafeMode;
    use std::sync::atomic::{AtomicU32, Ordering};

    static PANIC_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn simulate_panel_panic() {
        PANIC_COUNTER.fetch_add(1, Ordering::SeqCst);
        panic!("Simulated panel panic for testing");
    }

    #[test]
    fn editor_safe_mode_disables_panel_after_three_panics() {
        let mut safe_mode = EditorSafeMode::new();
        let panel_id = "test_panel";
        
        // Panel should be enabled initially
        assert!(safe_mode.is_panel_enabled(panel_id));
        
        // First panic - panel should remain enabled
        let result1 = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                simulate_panel_panic();
            });
        });
        
        assert!(result1.is_err(), "first panic should be caught");
        assert!(safe_mode.is_panel_enabled(panel_id), "panel should remain enabled after 1 panic");
        assert_eq!(safe_mode.panic_count(panel_id), 1);
        
        // Second panic - panel should remain enabled
        let result2 = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                simulate_panel_panic();
            });
        });
        
        assert!(result2.is_err(), "second panic should be caught");
        assert!(safe_mode.is_panel_enabled(panel_id), "panel should remain enabled after 2 panics");
        assert_eq!(safe_mode.panic_count(panel_id), 2);
        
        // Third panic - panel should be disabled
        let result3 = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                simulate_panel_panic();
            });
        });
        
        assert!(result3.is_err(), "third panic should be caught");
        assert!(!safe_mode.is_panel_enabled(panel_id), "panel should be disabled after 3 panics");
        assert_eq!(safe_mode.panic_count(panel_id), 3);
        
        // Fourth operation should be blocked
        let result4 = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                // This should not execute
                unreachable!("panel operation should be blocked");
            });
        });
        
        assert!(result4.is_err(), "blocked operation should return error");
        assert!(!safe_mode.is_panel_enabled(panel_id), "panel should remain disabled");
    }

    #[test]
    fn editor_safe_mode_isolation_between_panels() {
        let mut safe_mode = EditorSafeMode::new();
        
        let panel_a = "panel_a";
        let panel_b = "panel_b";
        let panel_c = "panel_c";
        
        // Panel A panics 3 times (should be disabled)
        for _ in 0..3 {
            let _ = std::panic::catch_unwind(|| {
                safe_mode.handle_panel_operation(panel_a, || {
                    simulate_panel_panic();
                });
            });
        }
        
        assert!(!safe_mode.is_panel_enabled(panel_a));
        
        // Panel B panics 1 time (should remain enabled)
        let _ = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_b, || {
                simulate_panel_panic();
            });
        });
        
        assert!(safe_mode.is_panel_enabled(panel_b));
        
        // Panel C never panics (should remain enabled)
        assert!(safe_mode.is_panel_enabled(panel_c));
        
        // Verify panic counts are isolated
        assert_eq!(safe_mode.panic_count(panel_a), 3);
        assert_eq!(safe_mode.panic_count(panel_b), 1);
        assert_eq!(safe_mode.panic_count(panel_c), 0);
    }

    #[test]
    fn editor_safe_mode_manual_enable_disable() {
        let mut safe_mode = EditorSafeMode::new();
        let panel_id = "manual_panel";
        
        // Initially enabled
        assert!(safe_mode.is_panel_enabled(panel_id));
        
        // Manual disable
        safe_mode.disable_panel(panel_id);
        assert!(!safe_mode.is_panel_enabled(panel_id));
        
        // Operations should be blocked
        let result = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                // Should not execute
                unreachable!("operation should be blocked");
            });
        });
        
        assert!(result.is_err());
        
        // Manual enable
        safe_mode.enable_panel(panel_id);
        assert!(safe_mode.is_panel_enabled(panel_id));
        
        // Panic count should be reset
        assert_eq!(safe_mode.panic_count(panel_id), 0);
        
        // Operations should work again
        let result = std::panic::catch_unwind(|| {
            safe_mode.handle_panel_operation(panel_id, || {
                "operation succeeded"
            });
        });
        
        assert!(result.is_ok());
    }

    #[test]
    fn editor_safe_mode_panic_recovery_with_timeout() {
        let mut safe_mode = EditorSafeMode::new();
        let panel_id = "recovery_panel";
        
        // Trigger 3 panics to disable panel
        for _ in 0..3 {
            let _ = std::panic::catch_unwind(|| {
                safe_mode.handle_panel_operation(panel_id, || {
                    simulate_panel_panic();
                });
            });
        }
        
        assert!(!safe_mode.is_panel_enabled(panel_id));
        
        // Simulate time passing (in real implementation this would be time-based)
        // For testing, we'll use manual recovery
        safe_mode.attempt_recovery(panel_id);
        
        // Panel should remain disabled (recovery failed due to recent panics)
        assert!(!safe_mode.is_panel_enabled(panel_id));
        
        // Simulate successful recovery conditions
        safe_mode.force_recovery(panel_id);
        
        // Panel should be enabled again
        assert!(safe_mode.is_panel_enabled(panel_id));
        assert_eq!(safe_mode.panic_count(panel_id), 0);
    }
}

#[cfg(test)]
mod debug_surface_isolation_tests {
    use engene::tools::debug_surface::DebugSurface;
    use engene::tools::debug_registry::DebugCategory;

    #[test]
    fn debug_surface_isolates_categories() {
        let mut surface = DebugSurface::new();
        
        // Register views in different categories
        surface.register_view("nav_mesh", DebugCategory::World);
        surface.register_view("ai_goals", DebugCategory::AI);
        surface.register_view("budgets", DebugCategory::Budgets);
        surface.register_view("physics_debug", DebugCategory::Physics);
        
        // Enable one category
        surface.enable_category(DebugCategory::World);
        
        // World category views should be enabled
        assert!(surface.is_view_enabled("nav_mesh"));
        
        // Other categories should be disabled
        assert!(!surface.is_view_enabled("ai_goals"));
        assert!(!surface.is_view_enabled("budgets"));
        assert!(!surface.is_view_enabled("physics_debug"));
        
        // Enable AI category
        surface.enable_category(DebugCategory::AI);
        
        // Both World and AI should be enabled
        assert!(surface.is_view_enabled("nav_mesh"));
        assert!(surface.is_view_enabled("ai_goals"));
        
        // Budgets and Physics should still be disabled
        assert!(!surface.is_view_enabled("budgets"));
        assert!(!surface.is_view_enabled("physics_debug"));
    }

    #[test]
    fn debug_surface_prevents_cross_category_pollution() {
        let mut surface = DebugSurface::new();
        
        // Register views
        surface.register_view("world_view1", DebugCategory::World);
        surface.register_view("world_view2", DebugCategory::World);
        surface.register_view("ai_view1", DebugCategory::AI);
        
        // Enable World category
        surface.enable_category(DebugCategory::World);
        
        // Try to enable AI view directly (should fail or be ignored)
        surface.enable_view("ai_view1");
        
        // AI view should still be disabled (category-level control)
        assert!(!surface.is_view_enabled("ai_view1"));
        
        // World views should remain enabled
        assert!(surface.is_view_enabled("world_view1"));
        assert!(surface.is_view_enabled("world_view2"));
        
        // Disable World category
        surface.disable_category(DebugCategory::World);
        
        // All World views should be disabled
        assert!(!surface.is_view_enabled("world_view1"));
        assert!(!surface.is_view_enabled("world_view2"));
        
        // AI view should still be disabled
        assert!(!surface.is_view_enabled("ai_view1"));
    }

    #[test]
    fn debug_surface_handles_category_view_counts() {
        let mut surface = DebugSurface::new();
        
        // Add multiple views to each category
        surface.register_view("world_1", DebugCategory::World);
        surface.register_view("world_2", DebugCategory::World);
        surface.register_view("world_3", DebugCategory::World);
        
        surface.register_view("ai_1", DebugCategory::AI);
        surface.register_view("ai_2", DebugCategory::AI);
        
        surface.register_view("budget_1", DebugCategory::Budgets);
        
        // Check view counts
        assert_eq!(surface.view_count_in_category(DebugCategory::World), 3);
        assert_eq!(surface.view_count_in_category(DebugCategory::AI), 2);
        assert_eq!(surface.view_count_in_category(DebugCategory::Budgets), 1);
        
        // Enable World category
        surface.enable_category(DebugCategory::World);
        
        // Active view counts should reflect enabled categories
        assert_eq!(surface.active_view_count(), 3);
        
        // Enable AI category
        surface.enable_category(DebugCategory::AI);
        
        assert_eq!(surface.active_view_count(), 5); // 3 World + 2 AI
        
        // Disable World category
        surface.disable_category(DebugCategory::World);
        
        assert_eq!(surface.active_view_count(), 2); // Only AI views
    }
}

#[cfg(test)]
mod profiler_isolation_tests {
    use engene::tools::profiler::Profiler;
    use engene::tools::profiler::ProfileCategory;

    #[test]
    fn profiler_isolates_profile_categories() {
        let mut profiler = Profiler::new();
        
        // Start profiling in different categories
        profiler.start_profile("render_frame", ProfileCategory::Render);
        profiler.start_profile("physics_update", ProfileCategory::Physics);
        profiler.start_profile("ai_think", ProfileCategory::AI);
        
        // Enable only Render profiling
        profiler.enable_category(ProfileCategory::Render);
        
        // Render should be collecting data
        assert!(profiler.is_category_enabled(ProfileCategory::Render));
        assert!(!profiler.is_category_enabled(ProfileCategory::Physics));
        assert!(!profiler.is_category_enabled(ProfileCategory::AI));
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        // Stop profiles
        let render_time = profiler.stop_profile("render_frame").unwrap();
        let physics_time = profiler.stop_profile("physics_update").unwrap();
        let ai_time = profiler.stop_profile("ai_think").unwrap();
        
        // Render should have meaningful data
        assert!(render_time.as_millis() > 0);
        
        // Physics and AI should have zero or minimal data (disabled)
        assert!(physics_time.as_millis() == 0);
        assert!(ai_time.as_millis() == 0);
    }

    #[test]
    fn profiler_prevents_category_cross_contamination() {
        let mut profiler = Profiler::new();
        
        // Enable Render category
        profiler.enable_category(ProfileCategory::Render);
        
        // Start profile in disabled category
        profiler.start_profile("disabled_physics", ProfileCategory::Physics);
        
        // Try to manually enable physics profile (should be ignored)
        profiler.enable_profile("disabled_physics");
        
        // Category should still be disabled
        assert!(!profiler.is_category_enabled(ProfileCategory::Physics));
        assert!(!profiler.is_profile_enabled("disabled_physics"));
        
        // Start profile in enabled category
        profiler.start_profile("enabled_render", ProfileCategory::Render);
        
        assert!(profiler.is_profile_enabled("enabled_render"));
        
        // Disable the entire Render category
        profiler.disable_category(ProfileCategory::Render);
        
        // Both profiles should be disabled
        assert!(!profiler.is_profile_enabled("enabled_render"));
        assert!(!profiler.is_profile_enabled("disabled_physics"));
    }

    #[test]
    fn profiler_handles_profile_hierarchy_isolation() {
        let mut profiler = Profiler::new();
        
        // Create hierarchical profiles
        profiler.start_profile("root", ProfileCategory::General);
        profiler.start_profile("root/child1", ProfileCategory::General);
        profiler.start_profile("root/child2", ProfileCategory::General);
        profiler.start_profile("root/child1/grandchild", ProfileCategory::General);
        
        // Enable General category
        profiler.enable_category(ProfileCategory::General);
        
        // All profiles should be enabled
        assert!(profiler.is_profile_enabled("root"));
        assert!(profiler.is_profile_enabled("root/child1"));
        assert!(profiler.is_profile_enabled("root/child2"));
        assert!(profiler.is_profile_enabled("root/child1/grandchild"));
        
        // Disable parent profile
        profiler.disable_profile("root/child1");
        
        // Child should be disabled (parent disabled)
        assert!(!profiler.is_profile_enabled("root/child1"));
        assert!(!profiler.is_profile_enabled("root/child1/grandchild"));
        
        // Sibling should remain enabled
        assert!(profiler.is_profile_enabled("root/child2"));
        
        // Root should remain enabled
        assert!(profiler.is_profile_enabled("root"));
    }
}

#[cfg(test)]
mod dashboard_isolation_tests {
    use engene::tools::dashboard::Dashboard;
    use engene::tools::dashboard::PanelType;

    #[test]
    fn dashboard_isolates_panel_types() {
        let mut dashboard = Dashboard::new();
        
        // Add different panel types
        dashboard.add_panel("world_editor", PanelType::WorldEditor);
        dashboard.add_panel("property_editor", PanelType::PropertyEditor);
        dashboard.add_panel("console", PanelType::Console);
        dashboard.add_panel("profiler", PanelType::Profiler);
        
        // Enable only WorldEditor panels
        dashboard.enable_panel_type(PanelType::WorldEditor);
        
        // WorldEditor should be enabled
        assert!(dashboard.is_panel_visible("world_editor"));
        
        // Other panel types should be hidden
        assert!(!dashboard.is_panel_visible("property_editor"));
        assert!(!dashboard.is_panel_visible("console"));
        assert!(!dashboard.is_panel_visible("profiler"));
        
        // Enable Console panels
        dashboard.enable_panel_type(PanelType::Console);
        
        // WorldEditor and Console should be visible
        assert!(dashboard.is_panel_visible("world_editor"));
        assert!(dashboard.is_panel_visible("console"));
        
        // PropertyEditor and Profiler should remain hidden
        assert!(!dashboard.is_panel_visible("property_editor"));
        assert!(!dashboard.is_panel_visible("profiler"));
    }

    #[test]
    fn dashboard_handles_panel_layout_isolation() {
        let mut dashboard = Dashboard::new();
        
        // Add panels to different layout regions
        dashboard.add_panel_to_region("left_panel", PanelType::PropertyEditor, "left");
        dashboard.add_panel_to_region("right_panel", PanelType::Profiler, "right");
        dashboard.add_panel_to_region("bottom_panel", PanelType::Console, "bottom");
        
        // Enable specific regions
        dashboard.enable_region("left");
        dashboard.enable_region("bottom");
        
        // Left and bottom panels should be visible
        assert!(dashboard.is_panel_visible("left_panel"));
        assert!(dashboard.is_panel_visible("bottom_panel"));
        
        // Right panel should be hidden (region disabled)
        assert!(!dashboard.is_panel_visible("right_panel"));
        
        // Disable entire left region
        dashboard.disable_region("left");
        
        // Left panel should be hidden
        assert!(!dashboard.is_panel_visible("left_panel"));
        
        // Bottom panel should remain visible
        assert!(dashboard.is_panel_visible("bottom_panel"));
        
        // Right panel should still be hidden
        assert!(!dashboard.is_panel_visible("right_panel"));
    }

    #[test]
    fn dashboard_prevents_panel_state_pollution() {
        let mut dashboard = Dashboard::new();
        
        // Add panels
        dashboard.add_panel("panel_a", PanelType::WorldEditor);
        dashboard.add_panel("panel_b", PanelType::WorldEditor);
        dashboard.add_panel("panel_c", PanelType::PropertyEditor);
        
        // Set panel states
        dashboard.set_panel_state("panel_a", "focused");
        dashboard.set_panel_state("panel_b", "minimized");
        dashboard.set_panel_state("panel_c", "hidden");
        
        // Disable WorldEditor panel type
        dashboard.disable_panel_type(PanelType::WorldEditor);
        
        // WorldEditor panels should be hidden regardless of individual state
        assert!(!dashboard.is_panel_visible("panel_a"));
        assert!(!dashboard.is_panel_visible("panel_b"));
        
        // PropertyEditor should retain its state
        assert!(!dashboard.is_panel_visible("panel_c")); // Still hidden by individual state
        
        // Re-enable WorldEditor
        dashboard.enable_panel_type(PanelType::WorldEditor);
        
        // WorldEditor panels should restore their individual states
        assert!(dashboard.is_panel_visible("panel_a")); // Was focused
        assert!(dashboard.is_panel_visible("panel_b")); // Was minimized (still visible)
        
        // States should be preserved
        assert_eq!(dashboard.get_panel_state("panel_a"), Some("focused".to_string()));
        assert_eq!(dashboard.get_panel_state("panel_b"), Some("minimized".to_string()));
    }
}
