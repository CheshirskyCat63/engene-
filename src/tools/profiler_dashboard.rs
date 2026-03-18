use crate::core::perf::telemetry::Telemetry;

pub struct ProfilerState {
    pub frame_history: Vec<f32>,
    pub max_history: usize,
}

impl Default for ProfilerState {
    fn default() -> Self {
        Self {
            frame_history: Vec::new(),
            max_history: 120,
        }
    }
}

impl ProfilerState {
    pub fn record_frame_ms(&mut self, ms: f32) {
        if self.frame_history.len() >= self.max_history {
            self.frame_history.remove(0);
        }
        self.frame_history.push(ms);
    }
}

pub fn draw_profiler(ctx: &egui::Context, state: &mut ProfilerState, telemetry: &Telemetry) {
    egui::Window::new("Profiler Dashboard")
        .default_width(500.0)
        .show(ctx, |ui| {
            let frame_us = telemetry.frame_time_us();
            let frame_ms = frame_us as f32 / 1000.0;
            let fps = if frame_ms > 0.0 {
                1000.0 / frame_ms
            } else {
                0.0
            };

            ui.heading(format!("Frame: {:.1}ms  FPS: {:.0}", frame_ms, fps));
            ui.separator();

            if !state.frame_history.is_empty() {
                let avg =
                    state.frame_history.iter().sum::<f32>() / state.frame_history.len() as f32;
                let max_val = state.frame_history.iter().cloned().fold(0.0f32, f32::max);
                let min_val = state.frame_history.iter().cloned().fold(f32::MAX, f32::min);
                ui.label(format!(
                    "Avg: {:.1}ms  Min: {:.1}ms  Max: {:.1}ms",
                    avg, min_val, max_val
                ));

                let bar_height = 60.0;
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(ui.available_width(), bar_height),
                    egui::Sense::hover(),
                );
                let rect = response.rect;
                let n = state.frame_history.len();
                let bar_w = rect.width() / n as f32;
                let scale = if max_val > 0.0 {
                    bar_height / max_val
                } else {
                    1.0
                };

                for (i, &ms) in state.frame_history.iter().enumerate() {
                    let h = ms * scale;
                    let color = if ms > 33.0 {
                        egui::Color32::RED
                    } else if ms > 16.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::GREEN
                    };
                    let x = rect.left() + i as f32 * bar_w;
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(x, rect.bottom() - h),
                            egui::pos2(x + bar_w - 1.0, rect.bottom()),
                        ),
                        0.0,
                        color,
                    );
                }
            }
            ui.separator();

            ui.heading("System Timings");
            let mut timings: Vec<_> = telemetry.system_timings().values().collect();
            timings.sort_by(|a, b| {
                b.avg_us
                    .partial_cmp(&a.avg_us)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            egui::Grid::new("system_timings_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("System");
                    ui.strong("Last (us)");
                    ui.strong("Avg (us)");
                    ui.strong("Max (us)");
                    ui.end_row();

                    for timing in &timings {
                        ui.label(&timing.system_name);
                        ui.label(format!("{}", timing.last_us));
                        ui.label(format!("{:.0}", timing.avg_us));
                        ui.label(format!("{}", timing.max_us));
                        ui.end_row();
                    }
                });
        });
}
