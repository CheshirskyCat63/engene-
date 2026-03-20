//! EditorShell owns all editor panel states and provides a single `draw()` method
//! that can be called from the main render loop.
//! All 17 panels are wired through the menu system (Track L4).

use crate::core::ecs::Ecs;
use crate::core::engine::Engine;
use crate::core::events::EventBus;
use crate::core::perf::telemetry::Telemetry;
use crate::tools::asset_browser::AssetBrowser;
use crate::tools::console::EngineConsole;
use crate::tools::crash_log_viewer::CrashLogViewer;
use crate::tools::debug_ui::{draw_main_menu, DebugUiState};
use crate::tools::doctor::DoctorReport;
use crate::tools::economy_dashboard::EconomyDashboard;
use crate::tools::editor_safe_mode::EditorSafeMode;
use crate::tools::event_monitor::{draw_event_monitor, EventMonitorState};
use crate::tools::inspector::{draw_inspector, InspectorEdit, InspectorState};
use crate::tools::overlays::{draw_overlay_panel, OverlayState};
use crate::tools::persistence_dashboard::PersistenceDashboard;
use crate::tools::prefab_placer::{draw_prefab_placer, PrefabPlacerState};
use crate::tools::profiler_dashboard::{draw_profiler, ProfilerState};
use crate::tools::quest_board::QuestBoardPanel;
use crate::tools::replay_browser::ReplayBrowser;
use crate::tools::runtime_truth_dashboard::RuntimeTruthDashboard;
use crate::tools::scene_hierarchy::SceneHierarchy;
use crate::tools::sim_metrics_dashboard::SimMetricsDashboard;
use crate::tools::time_controls::{draw_time_controls, TimeControlState};
use crate::tools::world_map::WorldMapPanel;

pub struct EditorShell {
    pub debug_ui: DebugUiState,
    pub inspector: InspectorState,
    pub overlays: OverlayState,
    pub profiler: ProfilerState,
    pub scene_hierarchy: SceneHierarchy,
    pub event_monitor: EventMonitorState,
    pub time_controls: TimeControlState,
    pub console: EngineConsole,
    pub asset_browser: AssetBrowser,
    pub replay_browser: ReplayBrowser,
    pub runtime_truth: RuntimeTruthDashboard,
    pub sim_metrics: SimMetricsDashboard,
    pub persistence: PersistenceDashboard,
    pub world_map: WorldMapPanel,
    pub quest_board: QuestBoardPanel,
    pub economy: EconomyDashboard,
    pub crash_log: CrashLogViewer,
    pub prefab_placer: PrefabPlacerState,
    pub safe_mode: EditorSafeMode,
    pub last_doctor_report: Option<DoctorReport>,
    pub read_only: bool,
}

#[derive(Debug, Default, Clone)]
pub struct InspectorApplyResult {
    pub spatial_dirty: bool,
    pub spatial_moved_entities: Vec<crate::core::ecs::Entity>,
}

impl EditorShell {
    pub fn new() -> Self {
        let mut safe_mode = EditorSafeMode::new();
        let panel_names = [
            "Inspector",
            "Overlays",
            "Profiler",
            "SceneHierarchy",
            "EventMonitor",
            "TimeControls",
            "Doctor",
            "Console",
            "AssetBrowser",
            "ReplayBrowser",
            "RuntimeTruth",
            "SimMetrics",
            "Persistence",
            "WorldMap",
            "QuestBoard",
            "Economy",
            "CrashLog",
            "PrefabPlacer",
        ];
        for name in &panel_names {
            safe_mode.register_panel(name);
        }

        Self {
            debug_ui: DebugUiState::sdk_defaults(),
            inspector: InspectorState::default(),
            overlays: OverlayState::default(),
            profiler: ProfilerState::default(),
            scene_hierarchy: SceneHierarchy::new(),
            event_monitor: EventMonitorState::default(),
            time_controls: TimeControlState::default(),
            console: EngineConsole::new(),
            asset_browser: AssetBrowser::new(),
            replay_browser: ReplayBrowser::new(),
            runtime_truth: RuntimeTruthDashboard::new(),
            sim_metrics: SimMetricsDashboard::default(),
            persistence: PersistenceDashboard::new(),
            world_map: WorldMapPanel::new(),
            quest_board: QuestBoardPanel::new(),
            economy: EconomyDashboard::default(),
            crash_log: CrashLogViewer::default(),
            prefab_placer: PrefabPlacerState::default(),
            safe_mode,
            last_doctor_report: None,
            read_only: false,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, ecs: &Ecs, telemetry: &Telemetry) {
        draw_main_menu(ctx, &mut self.debug_ui);

        if self.debug_ui.show_inspector {
            draw_inspector(ctx, ecs, &mut self.inspector);
        }
        if self.debug_ui.show_overlays {
            draw_overlay_panel(ctx, &mut self.overlays);
        }
        if self.debug_ui.show_profiler {
            draw_profiler(ctx, &mut self.profiler, telemetry);
        }
        if self.debug_ui.show_scene_hierarchy {
            #[cfg(feature = "debug_ui")]
            {
                let entities: Vec<(u32, String, Option<u32>)> = ecs
                    .alive()
                    .iter()
                    .map(|&e| {
                        let name = ecs
                            .names()
                            .get(&e)
                            .map(|n| n.0.as_str())
                            .unwrap_or("unnamed")
                            .to_string();
                        (e as u32, name, None)
                    })
                    .collect();
                self.scene_hierarchy.rebuild(&entities);
                egui::SidePanel::right("scene_hierarchy")
                    .default_width(280.0)
                    .show(ctx, |ui| {
                        ui.heading("Scene Hierarchy");
                        ui.separator();
                        self.scene_hierarchy.draw(ui);
                    });
            }
        }
        if self.debug_ui.show_time_controls {
            draw_time_controls(ctx, &mut self.time_controls);
        }
        if self.debug_ui.show_doctor {
            if let Some(ref report) = self.last_doctor_report {
                crate::tools::doctor::draw_doctor(ctx, report);
            }
        }
        if self.debug_ui.show_console {
            egui::Window::new("Console")
                .default_width(500.0)
                .show(ctx, |ui| {
                    self.console.draw(ui);
                });
        }
        if self.debug_ui.show_asset_browser {
            egui::Window::new("Asset Browser")
                .default_width(400.0)
                .show(ctx, |ui| {
                    self.asset_browser.draw(ui);
                });
        }
        if self.debug_ui.show_replay_browser {
            egui::Window::new("Replay Browser")
                .default_width(400.0)
                .show(ctx, |ui| {
                    self.replay_browser.draw(ui);
                });
        }
        if self.debug_ui.show_runtime_truth {
            self.runtime_truth.draw_ui(ctx);
        }
        if self.debug_ui.show_sim_metrics {
            self.sim_metrics.draw_ui(ctx);
        }
        if self.debug_ui.show_persistence {
            self.persistence.draw_ui(ctx);
        }
        if self.debug_ui.show_world_map {
            self.world_map.draw_ui(ctx);
        }
        if self.debug_ui.show_quest_board {
            self.quest_board.draw_ui(ctx);
        }
        if self.debug_ui.show_economy {
            self.economy.draw_ui(ctx);
        }
        if self.debug_ui.show_crash_log {
            self.crash_log.draw_ui(ctx);
        }
        if self.debug_ui.show_prefab_placer {
            draw_prefab_placer(ctx, &mut self.prefab_placer, None);
        }
    }

    pub fn draw_with_event_bus(
        &mut self,
        ctx: &egui::Context,
        ecs: &Ecs,
        telemetry: &Telemetry,
        bus: &EventBus,
    ) {
        self.draw(ctx, ecs, telemetry);
        if self.debug_ui.show_event_monitor {
            draw_event_monitor(ctx, &mut self.event_monitor, bus);
        }
    }

    /// Feed live engine data into dashboard panels.
    pub fn update_dashboards(&mut self, engine: &Engine) {
        let ecs = &engine.ecs;

        let telemetry = crate::core::perf::telemetry::Telemetry::new();
        self.runtime_truth.update_from_engine(engine, &telemetry);

        self.sim_metrics
            .record_snapshot(ecs, engine.time.month, engine.time.day);

        self.economy.record_snapshot(ecs, engine.time.month);

        self.world_map.update_from_ecs(ecs);

        self.crash_log.refresh();
    }

    /// Apply pending inspector edits to the engine.
    pub fn apply_inspector_edits(&mut self, engine: &mut Engine) -> InspectorApplyResult {
        let mut result = InspectorApplyResult::default();
        let edits: Vec<InspectorEdit> = self.inspector.pending_edits.drain(..).collect();
        for edit in edits {
            match edit {
                InspectorEdit::SetTransform { entity, x, y } => {
                    if let Some(t) = engine.ecs.get_transform_mut(entity) {
                        if t.x != x || t.y != y {
                            result.spatial_dirty = true;
                            result.spatial_moved_entities.push(entity);
                        }
                        t.x = x;
                        t.y = y;
                    }
                }
                InspectorEdit::SetHealth { entity, value } => {
                    if let Some(n) = engine.ecs.get_needs_mut(entity) {
                        n.health = value;
                    }
                }
                InspectorEdit::SetHunger { entity, value } => {
                    if let Some(n) = engine.ecs.get_needs_mut(entity) {
                        n.hunger = value;
                    }
                }
                InspectorEdit::SetThirst { entity, value } => {
                    if let Some(n) = engine.ecs.get_needs_mut(entity) {
                        n.thirst = value;
                    }
                }
                InspectorEdit::SetEnergy { entity, value } => {
                    if let Some(n) = engine.ecs.get_needs_mut(entity) {
                        n.energy = value;
                    }
                }
            }
        }
        result
    }
}

impl Default for EditorShell {
    fn default() -> Self {
        Self::new()
    }
}
