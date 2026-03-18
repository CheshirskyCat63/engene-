use crate::core::events::EventBus;

pub struct EventMonitorState {
    pub log: Vec<EventLogEntry>,
    pub max_entries: usize,
    pub paused: bool,
}

pub struct EventLogEntry {
    pub tick: u64,
    pub event_type: String,
    pub count: usize,
}

impl Default for EventMonitorState {
    fn default() -> Self {
        Self {
            log: Vec::new(),
            max_entries: 200,
            paused: false,
        }
    }
}

impl EventMonitorState {
    pub fn record(&mut self, tick: u64, event_type: &str, count: usize) {
        if self.paused {
            return;
        }
        if self.log.len() >= self.max_entries {
            self.log.remove(0);
        }
        self.log.push(EventLogEntry {
            tick,
            event_type: event_type.to_string(),
            count,
        });
    }
}

pub fn draw_event_monitor(ctx: &egui::Context, state: &mut EventMonitorState, bus: &EventBus) {
    egui::Window::new("Event Monitor")
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if state.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    state.paused = !state.paused;
                }
                if ui.button("Clear").clicked() {
                    state.log.clear();
                }
                ui.label(format!("Channels: {}", bus.channel_count()));
                ui.label(format!("Total dropped: {}", bus.total_dropped()));
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &state.log {
                        ui.horizontal(|ui| {
                            ui.monospace(format!("[{:6}]", entry.tick));
                            ui.label(&entry.event_type);
                            ui.label(format!("x{}", entry.count));
                        });
                    }
                });
        });
}
