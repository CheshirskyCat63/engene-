//! Persistence Dashboard — SDK panel showing chunk save/load state and relink health.

use crate::world::chunk_persistence::RelinkReport;

#[derive(Default)]
pub struct PersistenceDashboard {
    pub last_relink_reports: Vec<RelinkReport>,
    pub total_saves: u32,
    pub total_loads: u32,
    pub total_orphans_resolved: u32,
    pub total_orphans_remaining: u32,
    pub schema_version_save: u32,
    pub schema_version_chunk: u32,
    pub schema_warnings: Vec<String>,
}

impl PersistenceDashboard {
    pub fn new() -> Self {
        Self {
            schema_version_save: crate::core::build_manifest::SCHEMA_VERSION_SAVE,
            schema_version_chunk: crate::core::build_manifest::SCHEMA_VERSION_CHUNK,
            ..Default::default()
        }
    }

    pub fn record_save(&mut self) {
        self.total_saves += 1;
    }

    pub fn record_load(&mut self, report: RelinkReport) {
        self.total_loads += 1;
        self.total_orphans_resolved += report.social_ties_resolved as u32;
        self.total_orphans_remaining +=
            (report.social_ties_dangling + report.social_ties_dead) as u32;
        self.last_relink_reports.push(report);
        if self.last_relink_reports.len() > 50 {
            self.last_relink_reports.remove(0);
        }
    }

    pub fn has_unresolved_orphans(&self) -> bool {
        self.total_orphans_remaining > 0
    }

    pub fn health_summary(&self) -> String {
        format!(
            "saves:{} loads:{} orphans_resolved:{} orphans_remaining:{} schema:v{}/v{}",
            self.total_saves,
            self.total_loads,
            self.total_orphans_resolved,
            self.total_orphans_remaining,
            self.schema_version_save,
            self.schema_version_chunk,
        )
    }

    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("Persistence").default_width(380.0).show(ctx, |ui| {
            ui.label(format!("Saves: {}  Loads: {}", self.total_saves, self.total_loads));
            ui.label(format!("Orphans resolved: {}  remaining: {}",
                self.total_orphans_resolved, self.total_orphans_remaining));
            ui.label(format!("Schema: save v{}  chunk v{}",
                self.schema_version_save, self.schema_version_chunk));
            if !self.schema_warnings.is_empty() {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "Schema warnings:");
                for w in &self.schema_warnings {
                    ui.label(w);
                }
            }
            if !self.last_relink_reports.is_empty() {
                ui.separator();
                ui.label(format!("Last {} relink reports:", self.last_relink_reports.len()));
            }
        });
    }
}
