//! SDK, Editor, and GUI integration tests for ENGENE.
//! 146 tests covering Console, Inspector, DebugUi, SceneHierarchy, Overlays,
//! Profiler, TimeControls, AssetBrowser, ReplayBrowser, RuntimeTruth,
//! SimMetrics, PersistenceDashboard, WorldMap, QuestBoard, EconomyDashboard,
//! CrashLogViewer, EditorSafeMode, EventMonitor, and Doctor.

use engene::core::ecs::Ecs;
use engene::core::replay::recorder::ReplayHeader;
use engene::tools::asset_browser::{AssetBrowser, AssetEntry, SortField};
use engene::tools::console::EngineConsole;
use engene::tools::crash_log_viewer::CrashLogViewer;
use engene::tools::doctor::{Diagnostic, DiagnosticSeverity, DoctorReport};
use engene::tools::economy_dashboard::EconomyDashboard;
use engene::tools::editor_safe_mode::EditorSafeMode;
use engene::tools::event_monitor::{EventLogEntry, EventMonitorState};
use engene::tools::inspector::{InspectorEdit, InspectorState};
use engene::tools::overlays::OverlayState;
use engene::tools::persistence_dashboard::PersistenceDashboard;
use engene::tools::profiler_dashboard::ProfilerState;
use engene::tools::quest_board::QuestBoardPanel;
use engene::tools::replay_browser::ReplayBrowser;
use engene::tools::runtime_truth_dashboard::{ModuleEntry, ModuleStatus, RuntimeTruthDashboard};
use engene::tools::scene_hierarchy::SceneHierarchy;
use engene::tools::sim_metrics_dashboard::SimMetricsDashboard;
use engene::tools::time_controls::TimeControlState;
use engene::tools::world_map::WorldMapPanel;

// ===== 1. CONSOLE (20 tests) =====

#[test]
fn console_new_creates_instance() {
    let console = EngineConsole::new();
    assert!(console.command_count() > 0);
}

#[test]
fn console_execute_help_returns_command_list() {
    let mut console = EngineConsole::new();
    let out = console.execute("help");
    assert!(!out.is_empty());
    assert!(out.contains("help") || out.contains("clear"));
}

#[test]
fn console_execute_clear_clears_history() {
    let mut console = EngineConsole::new();
    console.execute("help");
    assert!(!console.history().is_empty());
    let _ = console.execute("clear");
    assert!(console.history().is_empty());
}

#[test]
fn console_execute_unknown_returns_error_message() {
    let mut console = EngineConsole::new();
    let out = console.execute("unknown_command_xyz");
    assert!(out.contains("Unknown") || out.contains("unknown"));
}

#[test]
fn console_command_count_returns_registered_count() {
    let console = EngineConsole::new();
    let count = console.command_count();
    assert!(count >= 14);
}

#[test]
fn console_history_empty_after_clear() {
    let mut console = EngineConsole::new();
    console.execute("help");
    console.execute("clear");
    assert!(console.history().is_empty());
}

#[test]
fn console_history_after_execute() {
    let mut console = EngineConsole::new();
    let _ = console.execute("help");
    assert_eq!(console.history().len(), 1);
    assert_eq!(console.history()[0].input, "help");
}

#[test]
fn console_register_command_and_execute() {
    let mut console = EngineConsole::new();
    console.register_command(
        "mytest",
        "custom test command",
        "mytest",
        Box::new(|_| "custom output".to_string()),
    );
    let out = console.execute("mytest");
    assert_eq!(out, "custom output");
}

#[test]
fn console_register_command_increases_count() {
    let mut console = EngineConsole::new();
    let before = console.command_count();
    console.register_command("newcmd", "desc", "usage", Box::new(|_| String::new()));
    assert_eq!(console.command_count(), before + 1);
}

#[test]
fn console_execute_empty_returns_empty() {
    let mut console = EngineConsole::new();
    let out = console.execute("");
    assert!(out.is_empty());
}

#[test]
fn console_execute_whitespace_only_returns_empty() {
    let mut console = EngineConsole::new();
    let out = console.execute("   ");
    assert!(out.is_empty());
}

#[test]
fn console_history_multiple_entries() {
    let mut console = EngineConsole::new();
    console.execute("help");
    console.execute("clear");
    console.execute("help");
    assert!(console.history().len() >= 1);
}

#[test]
fn console_default_same_as_new() {
    let default_console = EngineConsole::default();
    let new_console = EngineConsole::new();
    assert_eq!(default_console.command_count(), new_console.command_count());
}

#[test]
fn console_entry_is_error_for_unknown() {
    let mut console = EngineConsole::new();
    let _ = console.execute("nosuchcmd123");
    let last = console.history().last().unwrap();
    assert!(last.is_error);
}

#[test]
fn console_help_args_empty_lists_all() {
    let mut console = EngineConsole::new();
    let out = console.execute("help");
    assert!(!out.is_empty());
}

#[test]
fn console_history_preserves_input_output() {
    let mut console = EngineConsole::new();
    let _ = console.execute("help");
    let entry = &console.history()[0];
    assert_eq!(entry.input, "help");
    assert!(!entry.output.is_empty());
}

// ===== 2. INSPECTOR (12 tests) =====

#[test]
fn inspector_state_default_selected_entity_none() {
    let state = InspectorState::default();
    assert_eq!(state.selected_entity, None);
}

#[test]
fn inspector_state_default_search_filter_empty() {
    let state = InspectorState::default();
    assert!(state.search_filter.is_empty());
}

#[test]
fn inspector_state_default_pending_edits_empty() {
    let state = InspectorState::default();
    assert!(state.pending_edits.is_empty());
}

#[test]
fn inspector_state_default_editing_enabled_true() {
    let state = InspectorState::default();
    assert!(state.editing_enabled);
}

#[test]
fn inspector_edit_set_transform_variant() {
    let edit = InspectorEdit::SetTransform {
        entity: 42,
        x: 10.0,
        y: 20.0,
    };
    match &edit {
        InspectorEdit::SetTransform { entity, x, y } => {
            assert_eq!(*entity, 42);
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
        }
        _ => panic!("expected SetTransform"),
    }
}

#[test]
fn inspector_edit_set_health_variant() {
    let edit = InspectorEdit::SetHealth {
        entity: 1,
        value: 0.75,
    };
    match &edit {
        InspectorEdit::SetHealth { entity, value } => {
            assert_eq!(*entity, 1);
            assert!((*value - 0.75).abs() < 0.001);
        }
        _ => panic!("expected SetHealth"),
    }
}

#[test]
fn inspector_edit_set_hunger_variant() {
    let edit = InspectorEdit::SetHunger {
        entity: 2,
        value: 0.5,
    };
    match &edit {
        InspectorEdit::SetHunger { value, .. } => assert!((*value - 0.5).abs() < 0.001),
        _ => panic!("expected SetHunger"),
    }
}

#[test]
fn inspector_edit_set_thirst_variant() {
    let edit = InspectorEdit::SetThirst {
        entity: 3,
        value: 0.25,
    };
    match &edit {
        InspectorEdit::SetThirst { value, .. } => assert!((*value - 0.25).abs() < 0.001),
        _ => panic!("expected SetThirst"),
    }
}

#[test]
fn inspector_edit_set_energy_variant() {
    let edit = InspectorEdit::SetEnergy {
        entity: 4,
        value: 1.0,
    };
    match &edit {
        InspectorEdit::SetEnergy { value, .. } => assert!((*value - 1.0).abs() < 0.001),
        _ => panic!("expected SetEnergy"),
    }
}

#[test]
fn inspector_state_pending_edits_push() {
    let mut state = InspectorState::default();
    state.pending_edits.push(InspectorEdit::SetHealth {
        entity: 1,
        value: 0.5,
    });
    assert_eq!(state.pending_edits.len(), 1);
}

#[test]
fn inspector_state_pending_edits_multiple() {
    let mut state = InspectorState::default();
    state.pending_edits.push(InspectorEdit::SetTransform {
        entity: 1,
        x: 0.0,
        y: 0.0,
    });
    state.pending_edits.push(InspectorEdit::SetHealth {
        entity: 1,
        value: 1.0,
    });
    assert_eq!(state.pending_edits.len(), 2);
}

#[test]
fn inspector_state_search_filter_mutable() {
    let mut state = InspectorState::default();
    state.search_filter = "npc".to_string();
    assert_eq!(state.search_filter, "npc");
}

// ===== 3. DEBUG_UI (12 tests) =====

#[test]
fn debug_ui_sdk_defaults_show_inspector_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_inspector);
}

#[test]
fn debug_ui_sdk_defaults_show_scene_hierarchy_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_scene_hierarchy);
}

#[test]
fn debug_ui_sdk_defaults_show_console_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_console);
}

#[test]
fn debug_ui_sdk_defaults_show_profiler_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_profiler);
}

#[test]
fn debug_ui_sdk_defaults_show_doctor_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_doctor);
}

#[test]
fn debug_ui_sdk_defaults_show_world_map_true() {
    let state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    assert!(state.show_world_map);
}

#[test]
fn debug_ui_should_throttle_when_budget_aware_and_low_factor() {
    let mut state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    state.budget_aware = true;
    state.throttle_factor = 0.5;
    assert!(state.should_throttle());
}

#[test]
fn debug_ui_should_throttle_false_when_factor_one() {
    let mut state = engene::tools::debug_ui::DebugUiState::sdk_defaults();
    state.throttle_factor = 1.0;
    assert!(!state.should_throttle());
}

#[test]
fn debug_ui_set_pressure_high_sets_low_factor() {
    let mut state = engene::tools::debug_ui::DebugUiState::default();
    state.set_pressure(0.9);
    assert_eq!(state.throttle_factor, 0.25);
}

#[test]
fn debug_ui_set_pressure_medium_sets_half() {
    let mut state = engene::tools::debug_ui::DebugUiState::default();
    state.set_pressure(0.6);
    assert_eq!(state.throttle_factor, 0.5);
}

// ===== 4. SCENE_HIERARCHY (15 tests) =====

#[test]
fn scene_hierarchy_new_empty() {
    let h = SceneHierarchy::new();
    assert!(h.root_entities().is_empty());
    assert_eq!(h.entity_count(), 0);
    assert_eq!(h.selected(), None);
}

#[test]
fn scene_hierarchy_rebuild_populates_nodes() {
    let mut h = SceneHierarchy::new();
    let entities = vec![
        (1u32, "root1".to_string(), None),
        (2u32, "child".to_string(), Some(1)),
        (3u32, "root2".to_string(), None),
    ];
    h.rebuild(&entities);
    assert_eq!(h.entity_count(), 3);
}

#[test]
fn scene_hierarchy_select_sets_selected() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[(1, "a".into(), None), (2, "b".into(), None)]);
    h.select(2);
    assert_eq!(h.selected(), Some(2));
}

#[test]
fn scene_hierarchy_selected_none_initially() {
    let h = SceneHierarchy::new();
    assert_eq!(h.selected(), None);
}

#[test]
fn scene_hierarchy_set_filter_hides_non_matching() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[
        (1, "player".into(), None),
        (2, "npc_guard".into(), None),
        (3, "tree".into(), None),
    ]);
    h.set_filter("npc");
    let node = h.get_node(2).unwrap();
    assert!(node.visible);
    let node1 = h.get_node(1).unwrap();
    assert!(!node1.visible);
}

#[test]
fn scene_hierarchy_root_entities_after_rebuild() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[
        (1, "r1".into(), None),
        (2, "c1".into(), Some(1)),
        (3, "r2".into(), None),
    ]);
    let roots = h.root_entities();
    assert!(roots.contains(&1));
    assert!(roots.contains(&3));
    assert!(!roots.contains(&2));
}

#[test]
fn scene_hierarchy_get_node_returns_node() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[(5, "test".into(), None)]);
    let node = h.get_node(5).unwrap();
    assert_eq!(node.entity_id, 5);
    assert_eq!(node.name, "test");
}

#[test]
fn scene_hierarchy_get_node_none_for_missing() {
    let h = SceneHierarchy::new();
    assert!(h.get_node(999).is_none());
}

#[test]
fn scene_hierarchy_entity_count_matches_nodes() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[
        (1, "a".into(), None),
        (2, "b".into(), None),
        (3, "c".into(), None),
    ]);
    assert_eq!(h.entity_count(), 3);
}

#[test]
fn scene_hierarchy_default_same_as_new() {
    let default = SceneHierarchy::default();
    assert_eq!(default.entity_count(), 0);
}

#[test]
fn scene_hierarchy_hierarchy_node_has_children() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[(1, "p".into(), None), (2, "c".into(), Some(1))]);
    let parent = h.get_node(1).unwrap();
    assert!(parent.children.contains(&2));
}

#[test]
fn scene_hierarchy_filter_empty_shows_all() {
    let mut h = SceneHierarchy::new();
    h.rebuild(&[(1, "a".into(), None)]);
    h.set_filter("");
    let node = h.get_node(1).unwrap();
    assert!(node.visible);
}

// ===== 5. OVERLAYS (5 tests) =====

#[test]
fn overlay_state_default_all_false() {
    let state = OverlayState::default();
    assert!(!state.nav_mesh);
    assert!(!state.cover_map);
    assert!(!state.destruction_graph);
    assert!(!state.ai_debug_lines);
    assert!(!state.terrain_damage);
    assert!(!state.simulation_levels);
    assert!(!state.territory_map);
    assert!(!state.fire_grid);
}

#[test]
fn overlay_state_toggle_nav_mesh() {
    let mut state = OverlayState::default();
    state.nav_mesh = true;
    assert!(state.nav_mesh);
}

#[test]
fn overlay_state_toggle_cover_map() {
    let mut state = OverlayState::default();
    state.cover_map = true;
    assert!(state.cover_map);
}

#[test]
fn overlay_state_toggle_simulation_levels() {
    let mut state = OverlayState::default();
    state.simulation_levels = true;
    assert!(state.simulation_levels);
}

#[test]
fn overlay_state_toggle_multiple() {
    let mut state = OverlayState::default();
    state.nav_mesh = true;
    state.fire_grid = true;
    assert!(state.nav_mesh && state.fire_grid);
}

// ===== 6. PROFILER (8 tests) =====

#[test]
fn profiler_state_default() {
    let state = ProfilerState::default();
    assert!(state.frame_history.is_empty());
    assert_eq!(state.max_history, 120);
}

#[test]
fn profiler_record_frame_ms_appends() {
    let mut state = ProfilerState::default();
    state.record_frame_ms(16.0);
    assert_eq!(state.frame_history.len(), 1);
    assert!((state.frame_history[0] - 16.0).abs() < 0.001);
}

#[test]
fn profiler_record_frame_ms_multiple() {
    let mut state = ProfilerState::default();
    state.record_frame_ms(10.0);
    state.record_frame_ms(20.0);
    state.record_frame_ms(15.0);
    assert_eq!(state.frame_history.len(), 3);
}

#[test]
fn profiler_record_frame_ms_respects_max_history() {
    let mut state = ProfilerState::default();
    state.max_history = 5;
    for i in 0..10 {
        state.record_frame_ms(i as f32);
    }
    assert_eq!(state.frame_history.len(), 5);
}

#[test]
fn profiler_frame_history_values_preserved() {
    let mut state = ProfilerState::default();
    state.record_frame_ms(33.33);
    assert!((state.frame_history[0] - 33.33).abs() < 0.01);
}

#[test]
fn profiler_default_max_history_120() {
    let state = ProfilerState::default();
    assert_eq!(state.max_history, 120);
}

#[test]
fn profiler_history_fifo_when_full() {
    let mut state = ProfilerState::default();
    state.max_history = 3;
    state.record_frame_ms(1.0);
    state.record_frame_ms(2.0);
    state.record_frame_ms(3.0);
    state.record_frame_ms(4.0);
    assert_eq!(state.frame_history[0], 2.0);
    assert_eq!(state.frame_history[1], 3.0);
    assert_eq!(state.frame_history[2], 4.0);
}

// ===== 7. TIME_CONTROLS (10 tests) =====

#[test]
fn time_control_state_default() {
    let state = TimeControlState::default();
    assert!(!state.paused);
    assert!((state.time_scale - 1.0).abs() < 0.001);
    assert!(!state.single_step_requested);
    assert_eq!(state.fixed_tick_override, None);
}

#[test]
fn time_control_effective_delta_when_not_paused() {
    let state = TimeControlState::default();
    let delta = state.effective_delta(1.0);
    assert!((delta - 1.0).abs() < 0.001);
}

#[test]
fn time_control_effective_delta_paused_returns_zero() {
    let mut state = TimeControlState::default();
    state.paused = true;
    let delta = state.effective_delta(1.0);
    assert_eq!(delta, 0.0);
}

#[test]
fn time_control_effective_delta_paused_step_requested() {
    let mut state = TimeControlState::default();
    state.paused = true;
    state.single_step_requested = true;
    let delta = state.effective_delta(2.0);
    assert!((delta - 2.0).abs() < 0.001);
}

#[test]
fn time_control_consume_step_clears_flag() {
    let mut state = TimeControlState::default();
    state.single_step_requested = true;
    state.consume_step();
    assert!(!state.single_step_requested);
}

#[test]
fn time_control_effective_delta_scales() {
    let mut state = TimeControlState::default();
    state.time_scale = 2.0;
    let delta = state.effective_delta(1.0);
    assert!((delta - 2.0).abs() < 0.001);
}

#[test]
fn time_control_paused_behavior() {
    let mut state = TimeControlState::default();
    state.paused = true;
    assert!(state.paused);
}

#[test]
fn time_control_time_scale_mutable() {
    let mut state = TimeControlState::default();
    state.time_scale = 0.5;
    assert!((state.time_scale - 0.5).abs() < 0.001);
}

// ===== 8. ASSET_BROWSER (15 tests) =====

#[test]
fn asset_browser_new_empty() {
    let browser = AssetBrowser::new();
    assert_eq!(browser.asset_count(), 0);
    assert_eq!(browser.total_size(), 0);
    assert!(browser.selected_asset().is_none());
}

#[test]
fn asset_browser_populate() {
    let mut browser = AssetBrowser::new();
    let assets = vec![AssetEntry {
        id: 1,
        name: "test.ron".into(),
        asset_type: "prefab".into(),
        size_bytes: 100,
        path: "/a/b/test.ron".into(),
        cooked: true,
    }];
    browser.populate(assets);
    assert_eq!(browser.asset_count(), 1);
}

#[test]
fn asset_browser_set_filter_type() {
    let mut browser = AssetBrowser::new();
    browser.set_filter_type(Some("texture".to_string()));
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "a.ron".into(),
            asset_type: "prefab".into(),
            size_bytes: 10,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "b.png".into(),
            asset_type: "texture".into(),
            size_bytes: 20,
            path: "".into(),
            cooked: false,
        },
    ]);
    let visible = browser.visible_assets();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].asset_type, "texture");
}

#[test]
fn asset_browser_set_filter_text() {
    let mut browser = AssetBrowser::new();
    browser.set_filter_text("test");
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "other.ron".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "test_asset.ron".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
    ]);
    let visible = browser.visible_assets();
    assert_eq!(visible.len(), 1);
    assert!(visible[0].name.contains("test"));
}

#[test]
fn asset_browser_set_sort_name() {
    let mut browser = AssetBrowser::new();
    browser.set_sort(SortField::Name);
    browser.populate(vec![
        AssetEntry {
            id: 2,
            name: "zebra".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 1,
            name: "apple".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
    ]);
    let vis = browser.visible_assets();
    assert_eq!(vis[0].name, "apple");
}

#[test]
fn asset_browser_select() {
    let mut browser = AssetBrowser::new();
    browser.populate(vec![AssetEntry {
        id: 42,
        name: "x".into(),
        asset_type: "y".into(),
        size_bytes: 0,
        path: "".into(),
        cooked: false,
    }]);
    browser.select(42);
    let sel = browser.selected_asset().unwrap();
    assert_eq!(sel.id, 42);
}

#[test]
fn asset_browser_visible_assets_no_filter() {
    let mut browser = AssetBrowser::new();
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "a".into(),
            asset_type: "t".into(),
            size_bytes: 100,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "b".into(),
            asset_type: "t".into(),
            size_bytes: 200,
            path: "".into(),
            cooked: false,
        },
    ]);
    assert_eq!(browser.visible_assets().len(), 2);
}

#[test]
fn asset_browser_total_size() {
    let mut browser = AssetBrowser::new();
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "a".into(),
            asset_type: "x".into(),
            size_bytes: 100,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "b".into(),
            asset_type: "x".into(),
            size_bytes: 300,
            path: "".into(),
            cooked: false,
        },
    ]);
    assert_eq!(browser.total_size(), 400);
}

#[test]
fn asset_browser_asset_count() {
    let mut browser = AssetBrowser::new();
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "a".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "b".into(),
            asset_type: "x".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
    ]);
    assert_eq!(browser.asset_count(), 2);
}

#[test]
fn asset_browser_asset_types_deduped() {
    let mut browser = AssetBrowser::new();
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "a".into(),
            asset_type: "prefab".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "b".into(),
            asset_type: "prefab".into(),
            size_bytes: 0,
            path: "".into(),
            cooked: false,
        },
    ]);
    let types = browser.asset_types();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0], "prefab");
}

#[test]
fn asset_browser_sort_field_variants() {
    assert_eq!(SortField::Name, SortField::Name);
    assert_eq!(SortField::Type, SortField::Type);
    assert_eq!(SortField::Size, SortField::Size);
}

#[test]
fn asset_browser_default_same_as_new() {
    let def = AssetBrowser::default();
    assert_eq!(def.asset_count(), 0);
}

#[test]
fn asset_browser_selected_asset_none_when_empty() {
    let browser = AssetBrowser::new();
    assert!(browser.selected_asset().is_none());
}

#[test]
fn asset_browser_set_sort_size_descending() {
    let mut browser = AssetBrowser::new();
    browser.set_sort(SortField::Size);
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "small".into(),
            asset_type: "x".into(),
            size_bytes: 10,
            path: "".into(),
            cooked: false,
        },
        AssetEntry {
            id: 2,
            name: "large".into(),
            asset_type: "x".into(),
            size_bytes: 1000,
            path: "".into(),
            cooked: false,
        },
    ]);
    let vis = browser.visible_assets();
    assert_eq!(vis[0].size_bytes, 1000);
}

// ===== 9. REPLAY_BROWSER (8 tests) =====

#[test]
fn replay_browser_new_empty() {
    let browser = ReplayBrowser::new();
    assert_eq!(browser.recording_count(), 0);
    assert!(browser.selected_entry().is_none());
}

#[test]
fn replay_browser_add_recording() {
    let mut browser = ReplayBrowser::new();
    let header = ReplayHeader {
        seed: 12345,
        tick_rate: 60.0,
        version: 1,
        frame_count: 100,
    };
    browser.add_recording("test_run", header, 100);
    assert_eq!(browser.recording_count(), 1);
}

#[test]
fn replay_browser_add_recording_multiple() {
    let mut browser = ReplayBrowser::new();
    browser.add_recording(
        "r1",
        ReplayHeader {
            seed: 1,
            tick_rate: 60.0,
            version: 1,
            frame_count: 50,
        },
        50,
    );
    browser.add_recording(
        "r2",
        ReplayHeader {
            seed: 2,
            tick_rate: 30.0,
            version: 1,
            frame_count: 200,
        },
        200,
    );
    assert_eq!(browser.recording_count(), 2);
}

#[test]
fn replay_browser_select() {
    let mut browser = ReplayBrowser::new();
    browser.add_recording(
        "r1",
        ReplayHeader {
            seed: 0,
            tick_rate: 60.0,
            version: 1,
            frame_count: 0,
        },
        10,
    );
    browser.select(0);
    let entry = browser.selected_entry().unwrap();
    assert_eq!(entry.name, "r1");
}

#[test]
fn replay_browser_selected_entry_none_initially() {
    let browser = ReplayBrowser::new();
    assert!(browser.selected_entry().is_none());
}

#[test]
fn replay_browser_recording_count() {
    let mut browser = ReplayBrowser::new();
    for i in 0..5 {
        browser.add_recording(
            &format!("rec_{}", i),
            ReplayHeader {
                seed: i,
                tick_rate: 60.0,
                version: 1,
                frame_count: 0,
            },
            0,
        );
    }
    assert_eq!(browser.recording_count(), 5);
}

#[test]
fn replay_browser_replay_header_fields() {
    let header = ReplayHeader {
        seed: 999,
        tick_rate: 120.0,
        version: 2,
        frame_count: 500,
    };
    assert_eq!(header.seed, 999);
    assert!((header.tick_rate - 120.0).abs() < 0.001);
    assert_eq!(header.version, 2);
    assert_eq!(header.frame_count, 500);
}

// ===== 10. RUNTIME_TRUTH (6 tests) =====

#[test]
fn runtime_truth_dashboard_new() {
    let dash = RuntimeTruthDashboard::new();
    assert!(!dash.modules.is_empty());
}

#[test]
fn runtime_truth_dashboard_active_count() {
    let dash = RuntimeTruthDashboard::new();
    let active = dash.active_count();
    assert!(active > 0);
}

#[test]
fn runtime_truth_dashboard_frozen_count() {
    let dash = RuntimeTruthDashboard::new();
    let frozen = dash.frozen_count();
    assert!(frozen <= dash.modules.len());
}

#[test]
fn runtime_truth_module_status_active() {
    assert_eq!(format!("{}", ModuleStatus::Active), "ACTIVE");
}

#[test]
fn runtime_truth_module_status_frozen() {
    assert_eq!(format!("{}", ModuleStatus::Frozen), "FROZEN");
}

#[test]
fn runtime_truth_module_entry_has_name() {
    let entry = ModuleEntry {
        name: "Test".into(),
        path: "src/test.rs".into(),
        status: ModuleStatus::Active,
        notes: "notes".into(),
    };
    assert_eq!(entry.name, "Test");
}

// ===== 11. SIM_METRICS (8 tests) =====

#[test]
fn sim_metrics_dashboard_new() {
    let dash = SimMetricsDashboard::new(100);
    assert!(dash.latest().is_none());
}

#[test]
fn sim_metrics_dashboard_latest_when_empty() {
    let dash = SimMetricsDashboard::new(50);
    assert!(dash.latest().is_none());
}

#[test]
fn sim_metrics_dashboard_record_snapshot() {
    let mut dash = SimMetricsDashboard::new(10);
    let ecs = Ecs::new();
    dash.record_snapshot(&ecs, 1, 15);
    let latest = dash.latest().unwrap();
    assert_eq!(latest.month, 1);
    assert_eq!(latest.day, 15);
}

#[test]
fn sim_metrics_dashboard_population_trend_empty() {
    let dash = SimMetricsDashboard::new(10);
    let trend = dash.population_trend(5);
    assert!(trend.is_empty());
}

#[test]
fn sim_metrics_dashboard_population_trend_after_record() {
    let mut dash = SimMetricsDashboard::new(10);
    let mut ecs = Ecs::new();
    ecs.spawn();
    dash.record_snapshot(&ecs, 1, 1);
    let trend = dash.population_trend(5);
    assert_eq!(trend.len(), 1);
}

#[test]
fn sim_metrics_dashboard_default_max_history() {
    let mut dash = SimMetricsDashboard::default();
    dash.record_snapshot(&Ecs::new(), 1, 1);
    assert!(dash.latest().is_some());
}

// ===== 12. PERSISTENCE_DASHBOARD (5 tests) =====

#[test]
fn persistence_dashboard_new() {
    let dash = PersistenceDashboard::new();
    assert_eq!(dash.total_saves, 0);
    assert_eq!(dash.total_loads, 0);
}

#[test]
fn persistence_dashboard_record_save() {
    let mut dash = PersistenceDashboard::new();
    dash.record_save();
    dash.record_save();
    assert_eq!(dash.total_saves, 2);
}

#[test]
fn persistence_dashboard_has_unresolved_orphans_false_initially() {
    let dash = PersistenceDashboard::new();
    assert!(!dash.has_unresolved_orphans());
}

#[test]
fn persistence_dashboard_has_unresolved_orphans_true() {
    let mut dash = PersistenceDashboard::new();
    dash.total_orphans_remaining = 1;
    assert!(dash.has_unresolved_orphans());
}

#[test]
fn persistence_dashboard_health_summary() {
    let dash = PersistenceDashboard::new();
    let summary = dash.health_summary();
    assert!(summary.contains("saves:0"));
    assert!(summary.contains("loads:0"));
}

// ===== 13. WORLD_MAP (4 tests) =====

#[test]
fn world_map_panel_new() {
    let panel = WorldMapPanel::new();
    assert!(panel.cells.is_empty());
}

#[test]
fn world_map_panel_total_entities_empty() {
    let panel = WorldMapPanel::new();
    assert_eq!(panel.total_entities(), 0);
}

#[test]
fn world_map_panel_update_from_ecs() {
    let mut panel = WorldMapPanel::new();
    let ecs = Ecs::new();
    panel.update_from_ecs(&ecs);
    assert_eq!(panel.total_entities(), 0);
}

#[test]
fn world_map_panel_grid_size() {
    let panel = WorldMapPanel::new();
    assert!(panel.grid_size > 0);
}

// ===== 14. QUEST_BOARD (4 tests) =====

#[test]
fn quest_board_panel_new() {
    let panel = QuestBoardPanel::new();
    assert!(panel.entries.is_empty());
    assert_eq!(panel.total_generated, 0);
}

#[test]
fn quest_board_panel_active_count() {
    let panel = QuestBoardPanel::new();
    assert_eq!(panel.active_count(), 0);
}

#[test]
fn quest_board_panel_completion_rate_zero_when_empty() {
    let panel = QuestBoardPanel::new();
    assert!((panel.completion_rate() - 0.0).abs() < 0.001);
}

#[test]
fn quest_board_panel_completion_rate_with_data() {
    let mut panel = QuestBoardPanel::new();
    panel.total_completed = 8;
    panel.total_failed = 2;
    panel.total_expired = 0;
    let rate = panel.completion_rate();
    assert!((rate - 0.8).abs() < 0.001);
}

// ===== 15. ECONOMY_DASHBOARD (5 tests) =====

#[test]
fn economy_dashboard_new() {
    let dash = EconomyDashboard::new(50);
    assert!(dash.latest().is_none());
}

#[test]
fn economy_dashboard_latest_when_empty() {
    let dash = EconomyDashboard::new(100);
    assert!(dash.latest().is_none());
}

#[test]
fn economy_dashboard_wealth_trend_empty() {
    let dash = EconomyDashboard::new(50);
    let trend = dash.wealth_trend(10);
    assert!(trend.is_empty());
}

#[test]
fn economy_dashboard_record_snapshot() {
    let mut dash = EconomyDashboard::new(10);
    let ecs = Ecs::new();
    dash.record_snapshot(&ecs, 1);
    let latest = dash.latest().unwrap();
    assert_eq!(latest.month, 1);
}

#[test]
fn economy_dashboard_default() {
    let dash = EconomyDashboard::default();
    assert!(dash.latest().is_none());
}

// ===== 16. CRASH_LOG_VIEWER (4 tests) =====

#[test]
fn crash_log_viewer_new() {
    let viewer = CrashLogViewer::new(10);
    assert_eq!(viewer.max_entries, 10);
    assert_eq!(viewer.crash_count(), 0);
}

#[test]
fn crash_log_viewer_crash_count() {
    let viewer = CrashLogViewer::new(5);
    assert_eq!(viewer.crash_count(), 0);
}

#[test]
fn crash_log_viewer_has_recent_crashes_false_initially() {
    let viewer = CrashLogViewer::new(20);
    assert!(!viewer.has_recent_crashes());
}

#[test]
fn crash_log_viewer_default_max_entries() {
    let viewer = CrashLogViewer::default();
    assert_eq!(viewer.max_entries, 20);
}

// ===== 17. EDITOR_SAFE_MODE (10 tests) =====

#[test]
fn editor_safe_mode_new() {
    let safe = EditorSafeMode::new();
    assert!(!safe.is_safe_mode());
}

#[test]
fn editor_safe_mode_register_panel() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel("TestPanel");
    assert!(safe.is_panel_enabled("TestPanel"));
}

#[test]
fn editor_safe_mode_is_panel_enabled_unknown_false() {
    let safe = EditorSafeMode::new();
    assert!(!safe.is_panel_enabled("UnknownPanel"));
}

#[test]
fn editor_safe_mode_re_enable_panel() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel("P");
    safe.re_enable_panel("P");
    assert!(safe.is_panel_enabled("P"));
}

#[test]
fn editor_safe_mode_exit_safe_mode() {
    let mut safe = EditorSafeMode::new();
    safe.exit_safe_mode();
    assert!(!safe.is_safe_mode());
}

#[test]
fn editor_safe_mode_is_safe_mode_initially_false() {
    let safe = EditorSafeMode::new();
    assert!(!safe.is_safe_mode());
}

#[test]
fn editor_safe_mode_panel_report() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel("ReportPanel");
    let report = safe.panel_report();
    assert!(report.contains("ReportPanel"));
    assert!(report.contains("Safe mode"));
}

#[test]
fn editor_safe_mode_panel_budgets() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel("BudgetPanel");
    let budgets = safe.panel_budgets();
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].0, "BudgetPanel");
}

#[test]
fn editor_safe_mode_register_panel_with_budget() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel_with_budget("HeavyPanel", 5.0);
    assert!(safe.is_panel_enabled("HeavyPanel"));
}

#[test]
fn editor_safe_mode_safe_mode_active_blocks_panels() {
    let mut safe = EditorSafeMode::new();
    safe.register_panel("P");
    assert!(safe.is_panel_enabled("P"));
    safe.end_frame(100.0);
    assert!(safe.is_safe_mode());
    assert!(!safe.is_panel_enabled("P"));
}

// ===== 18. EVENT_MONITOR (5 tests) =====

#[test]
fn event_monitor_state_default() {
    let state = EventMonitorState::default();
    assert!(state.log.is_empty());
    assert_eq!(state.max_entries, 200);
    assert!(!state.paused);
}

#[test]
fn event_monitor_state_record() {
    let mut state = EventMonitorState::default();
    state.record(100, "TestEvent", 5);
    assert_eq!(state.log.len(), 1);
    assert_eq!(state.log[0].tick, 100);
    assert_eq!(state.log[0].event_type, "TestEvent");
    assert_eq!(state.log[0].count, 5);
}

#[test]
fn event_monitor_state_record_multiple() {
    let mut state = EventMonitorState::default();
    state.max_entries = 5;
    for i in 0..3 {
        state.record(i as u64, "E", 1);
    }
    assert_eq!(state.log.len(), 3);
}

#[test]
fn event_log_entry_fields() {
    let entry = EventLogEntry {
        tick: 42,
        event_type: "Spawn".into(),
        count: 10,
    };
    assert_eq!(entry.tick, 42);
    assert_eq!(entry.event_type, "Spawn");
    assert_eq!(entry.count, 10);
}

#[test]
fn event_monitor_state_paused_skips_record() {
    let mut state = EventMonitorState::default();
    state.paused = true;
    state.record(1, "E", 1);
    assert!(state.log.is_empty());
}

// ===== 19. DOCTOR (6 tests) =====

#[test]
fn doctor_report_construction() {
    let report = DoctorReport {
        diagnostics: vec![],
        risk_heatmap: std::collections::HashMap::new(),
    };
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 0);
}

#[test]
fn doctor_report_error_count() {
    let report = DoctorReport {
        diagnostics: vec![
            Diagnostic {
                severity: DiagnosticSeverity::Error,
                category: "test",
                message: "err".into(),
            },
            Diagnostic {
                severity: DiagnosticSeverity::Warning,
                category: "test",
                message: "warn".into(),
            },
        ],
        risk_heatmap: std::collections::HashMap::new(),
    };
    assert_eq!(report.error_count(), 1);
}

#[test]
fn doctor_report_warning_count() {
    let report = DoctorReport {
        diagnostics: vec![
            Diagnostic {
                severity: DiagnosticSeverity::Warning,
                category: "a",
                message: "w1".into(),
            },
            Diagnostic {
                severity: DiagnosticSeverity::Warning,
                category: "b",
                message: "w2".into(),
            },
        ],
        risk_heatmap: std::collections::HashMap::new(),
    };
    assert_eq!(report.warning_count(), 2);
}

#[test]
fn doctor_diagnostic_severity_info() {
    let d = Diagnostic {
        severity: DiagnosticSeverity::Info,
        category: "cat",
        message: "msg".into(),
    };
    assert!(matches!(d.severity, DiagnosticSeverity::Info));
}

#[test]
fn doctor_diagnostic_severity_warning() {
    let d = Diagnostic {
        severity: DiagnosticSeverity::Warning,
        category: "cat",
        message: "msg".into(),
    };
    assert!(matches!(d.severity, DiagnosticSeverity::Warning));
}

#[test]
fn doctor_diagnostic_severity_error() {
    let d = Diagnostic {
        severity: DiagnosticSeverity::Error,
        category: "cat",
        message: "msg".into(),
    };
    assert!(matches!(d.severity, DiagnosticSeverity::Error));
}
