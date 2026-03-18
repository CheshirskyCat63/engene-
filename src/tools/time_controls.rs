pub struct TimeControlState {
    pub paused: bool,
    pub time_scale: f32,
    pub single_step_requested: bool,
    pub fixed_tick_override: Option<f32>,
}

impl Default for TimeControlState {
    fn default() -> Self {
        Self {
            paused: false,
            time_scale: 1.0,
            single_step_requested: false,
            fixed_tick_override: None,
        }
    }
}

impl TimeControlState {
    pub fn effective_delta(&self, real_delta: f32) -> f32 {
        if self.paused {
            if self.single_step_requested {
                real_delta
            } else {
                0.0
            }
        } else {
            real_delta * self.time_scale
        }
    }

    pub fn consume_step(&mut self) {
        self.single_step_requested = false;
    }
}

pub fn draw_time_controls(ctx: &egui::Context, state: &mut TimeControlState) {
    egui::Window::new("Time Controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui
                .button(if state.paused {
                    "▶ Play"
                } else {
                    "⏸ Pause"
                })
                .clicked()
            {
                state.paused = !state.paused;
            }
            if ui
                .add_enabled(state.paused, egui::Button::new("⏭ Step"))
                .clicked()
            {
                state.single_step_requested = true;
            }
        });
        ui.separator();
        ui.add(
            egui::Slider::new(&mut state.time_scale, 0.1..=10.0)
                .text("Time Scale")
                .logarithmic(true),
        );
        ui.separator();

        let mut use_override = state.fixed_tick_override.is_some();
        if ui
            .checkbox(&mut use_override, "Override fixed tick rate")
            .changed()
        {
            state.fixed_tick_override = if use_override { Some(20.0) } else { None };
        }
        if let Some(ref mut rate) = state.fixed_tick_override {
            ui.add(egui::Slider::new(rate, 1.0..=120.0).text("Tick rate (Hz)"));
        }
    });
}
