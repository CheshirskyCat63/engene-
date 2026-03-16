//! Tooling subsystem - editor panels, debug UI, and diagnostics.
//!
//! # Status: partial
//! # Integration: enabled (debug_ui feature)
//! # Tests: integration only
//!
//! ## Modules
//! - `editor_shell`, `inspector` - partial, editor core
//! - `doctor`, `profiler_dashboard` - partial, diagnostics
//! - `debug_ui`, `console` - partial, debug interface
//! - `hot_reload` - partial, runtime shader/config reload (D.2)
//! - `content_hash` - partial, content change detection (D.7)

pub mod asset_browser;
pub mod console;
pub mod debug_editor;
pub mod debug_ui;
pub mod editor_safe_mode;
pub mod editor_shell;
pub mod editor_state;
pub mod doctor;
pub mod event_monitor;
pub mod game_tools;
pub mod hot_reload;
pub mod inspector;
pub mod overlays;
pub mod profiler_dashboard;
pub mod replay_browser;
pub mod scene_hierarchy;
pub mod time_controls;
pub mod runtime_truth_dashboard;
pub mod sim_metrics_dashboard;
pub mod persistence_dashboard;
pub mod world_map;
pub mod quest_board;
pub mod economy_dashboard;
pub mod crash_log_viewer;
pub mod prefab_placer;
pub mod doc_generator;
pub mod content_hash;
