pub struct DebugUiState {
    pub show_inspector: bool,
    pub show_overlays: bool,
    pub show_scene_hierarchy: bool,
    pub show_event_monitor: bool,
    pub show_time_controls: bool,
    pub show_profiler: bool,
    pub show_doctor: bool,
    pub show_console: bool,
    pub show_asset_browser: bool,
    pub show_replay_browser: bool,
    pub show_runtime_truth: bool,
    pub show_sim_metrics: bool,
    pub show_persistence: bool,
    pub show_world_map: bool,
    pub show_quest_board: bool,
    pub show_economy: bool,
    pub show_crash_log: bool,
    pub show_prefab_placer: bool,
    pub budget_aware: bool,
    pub throttle_factor: f32,
}

impl Default for DebugUiState {
    fn default() -> Self {
        Self {
            show_inspector: false,
            show_overlays: false,
            show_scene_hierarchy: false,
            show_event_monitor: false,
            show_time_controls: false,
            show_profiler: false,
            show_doctor: false,
            show_console: false,
            show_asset_browser: false,
            show_replay_browser: false,
            show_runtime_truth: false,
            show_sim_metrics: false,
            show_persistence: false,
            show_world_map: false,
            show_quest_board: false,
            show_economy: false,
            show_crash_log: false,
            show_prefab_placer: false,
            budget_aware: true,
            throttle_factor: 1.0,
        }
    }
}

impl DebugUiState {
    pub fn sdk_defaults() -> Self {
        Self {
            show_inspector: true,
            show_scene_hierarchy: true,
            show_event_monitor: true,
            show_time_controls: true,
            show_profiler: true,
            show_doctor: true,
            show_console: true,
            show_runtime_truth: true,
            show_persistence: true,
            show_world_map: true,
            ..Default::default()
        }
    }

    pub fn should_throttle(&self) -> bool {
        self.budget_aware && self.throttle_factor < 1.0
    }

    pub fn set_pressure(&mut self, pressure: f32) {
        self.throttle_factor = if pressure > 0.8 {
            0.25
        } else if pressure > 0.5 {
            0.5
        } else {
            1.0
        };
    }
}

pub fn draw_main_menu(ctx: &egui::Context, state: &mut DebugUiState) {
    egui::TopBottomPanel::top("debug_menu").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Debug", |ui| {
                ui.checkbox(&mut state.show_inspector, "Inspector");
                ui.checkbox(&mut state.show_overlays, "Overlays");
                ui.checkbox(&mut state.show_scene_hierarchy, "Scene Hierarchy");
                ui.checkbox(&mut state.show_event_monitor, "Event Monitor");
                ui.checkbox(&mut state.show_time_controls, "Time Controls");
                ui.checkbox(&mut state.show_profiler, "Profiler");
                ui.checkbox(&mut state.show_doctor, "Engine Doctor");
                ui.checkbox(&mut state.show_console, "Console");
                ui.separator();
                ui.checkbox(&mut state.budget_aware, "Budget-aware throttling");
            });
            ui.menu_button("Tools", |ui| {
                ui.checkbox(&mut state.show_asset_browser, "Asset Browser");
                ui.checkbox(&mut state.show_replay_browser, "Replay Browser");
                ui.checkbox(&mut state.show_world_map, "World Map");
                ui.checkbox(&mut state.show_prefab_placer, "Prefab Placer");
            });
            ui.menu_button("Dashboards", |ui| {
                ui.checkbox(&mut state.show_runtime_truth, "Runtime Truth");
                ui.checkbox(&mut state.show_sim_metrics, "Sim Metrics");
                ui.checkbox(&mut state.show_persistence, "Persistence");
                ui.checkbox(&mut state.show_quest_board, "Quest Board");
                ui.checkbox(&mut state.show_economy, "Economy");
                ui.checkbox(&mut state.show_crash_log, "Crash Logs");
            });
        });
    });
}
