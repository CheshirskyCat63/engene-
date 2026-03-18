//! Crash Log Viewer — SDK panel showing recent crash bundles from crash_telemetry.

use crate::core::crash_telemetry::CrashBundle;
use std::path::PathBuf;

pub struct CrashLogViewer {
    pub recent_crashes: Vec<CrashLogEntry>,
    pub max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct CrashLogEntry {
    pub path: PathBuf,
    pub timestamp: String,
    pub message: String,
    pub location: Option<String>,
}

impl CrashLogViewer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            recent_crashes: Vec::new(),
            max_entries,
        }
    }

    pub fn refresh(&mut self) {
        let paths = CrashBundle::list_recent(self.max_entries);
        self.recent_crashes.clear();

        for path in paths {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(bundle) = ron::from_str::<CrashBundle>(&contents) {
                    self.recent_crashes.push(CrashLogEntry {
                        path: path.clone(),
                        timestamp: bundle.timestamp,
                        message: bundle.panic_message,
                        location: bundle.location,
                    });
                }
            }
        }
    }

    pub fn crash_count(&self) -> usize {
        self.recent_crashes.len()
    }

    pub fn has_recent_crashes(&self) -> bool {
        !self.recent_crashes.is_empty()
    }
}

impl CrashLogViewer {
    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("Crash Logs")
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.label(format!("Recent crashes: {}", self.crash_count()));
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for entry in &self.recent_crashes {
                            ui.group(|ui| {
                                ui.label(format!("Time: {}", entry.timestamp));
                                ui.colored_label(egui::Color32::RED, &entry.message);
                                if let Some(loc) = &entry.location {
                                    ui.weak(loc);
                                }
                            });
                        }
                    });
            });
    }
}

impl Default for CrashLogViewer {
    fn default() -> Self {
        Self::new(20)
    }
}
