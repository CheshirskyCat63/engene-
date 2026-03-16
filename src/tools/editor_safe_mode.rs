use std::collections::HashMap;
use std::time::Instant;

/// Per-panel cost budget and failure isolation for editor UI.
/// If a panel panics or exceeds its time budget, it gets disabled
/// rather than crashing the entire editor/engine.

const DEFAULT_PANEL_BUDGET_MS: f64 = 2.0;
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

#[derive(Clone, Debug)]
pub struct PanelHealth {
    pub name: String,
    pub enabled: bool,
    pub budget_ms: f64,
    pub last_draw_ms: f64,
    pub consecutive_failures: u32,
    pub total_failures: u32,
    pub disabled_reason: Option<String>,
}

impl PanelHealth {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            budget_ms: DEFAULT_PANEL_BUDGET_MS,
            last_draw_ms: 0.0,
            consecutive_failures: 0,
            total_failures: 0,
            disabled_reason: None,
        }
    }
}

pub struct EditorSafeMode {
    panels: HashMap<String, PanelHealth>,
    safe_mode_active: bool,
    total_editor_budget_ms: f64,
    last_frame_editor_ms: f64,
}

impl EditorSafeMode {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            safe_mode_active: false,
            total_editor_budget_ms: 8.0, // 8ms total for all editor UI
            last_frame_editor_ms: 0.0,
        }
    }

    pub fn register_panel(&mut self, name: &str) {
        self.panels.entry(name.to_string())
            .or_insert_with(|| PanelHealth::new(name));
    }

    pub fn register_panel_with_budget(&mut self, name: &str, budget_ms: f64) {
        let mut health = PanelHealth::new(name);
        health.budget_ms = budget_ms;
        self.panels.insert(name.to_string(), health);
    }

    pub fn is_panel_enabled(&self, name: &str) -> bool {
        if self.safe_mode_active {
            return false;
        }
        self.panels.get(name).map_or(false, |p| p.enabled)
    }

    /// Wrap a panel draw call with timing + panic isolation.
    /// Returns true if the panel drew successfully.
    pub fn draw_panel<F>(&mut self, name: &str, draw_fn: F) -> bool
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        if !self.is_panel_enabled(name) {
            return false;
        }

        let start = Instant::now();
        let result = std::panic::catch_unwind(draw_fn);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        if let Some(health) = self.panels.get_mut(name) {
            health.last_draw_ms = elapsed_ms;

            match result {
                Ok(()) => {
                    health.consecutive_failures = 0;
                    if elapsed_ms > health.budget_ms {
                        tracing::warn!(
                            "panel '{}' exceeded budget: {:.2}ms > {:.2}ms",
                            name, elapsed_ms, health.budget_ms
                        );
                    }
                    true
                }
                Err(_panic) => {
                    health.consecutive_failures += 1;
                    health.total_failures += 1;
                    tracing::error!(
                        "panel '{}' panicked ({}/{})",
                        name, health.consecutive_failures, MAX_CONSECUTIVE_FAILURES
                    );
                    if health.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        health.enabled = false;
                        health.disabled_reason = Some(format!(
                            "disabled after {} consecutive panics",
                            MAX_CONSECUTIVE_FAILURES
                        ));
                        tracing::error!("panel '{}' disabled due to repeated failures", name);
                    }
                    false
                }
            }
        } else {
            false
        }
    }

    /// Record total editor frame time and trigger safe mode if over budget
    pub fn end_frame(&mut self, total_editor_ms: f64) {
        self.last_frame_editor_ms = total_editor_ms;
        if total_editor_ms > self.total_editor_budget_ms * 3.0 {
            self.safe_mode_active = true;
            tracing::error!(
                "editor safe mode ACTIVATED: frame took {:.2}ms (budget: {:.2}ms)",
                total_editor_ms, self.total_editor_budget_ms
            );
        }
    }

    /// Re-enable a disabled panel (manual recovery)
    pub fn re_enable_panel(&mut self, name: &str) {
        if let Some(health) = self.panels.get_mut(name) {
            health.enabled = true;
            health.consecutive_failures = 0;
            health.disabled_reason = None;
        }
    }

    /// Exit safe mode (re-enables all panels)
    pub fn exit_safe_mode(&mut self) {
        self.safe_mode_active = false;
        for health in self.panels.values_mut() {
            health.enabled = true;
            health.consecutive_failures = 0;
            health.disabled_reason = None;
        }
    }

    pub fn is_safe_mode(&self) -> bool {
        self.safe_mode_active
    }

    /// Get failure counts for all panels (for persistence)
    pub fn panel_failure_counts(&self) -> HashMap<String, (u32, u32, bool)> {
        self.panels
            .iter()
            .map(|(name, health)| {
                (
                    name.clone(),
                    (
                        health.consecutive_failures,
                        health.total_failures,
                        !health.enabled,
                    ),
                )
            })
            .collect()
    }

    /// Get total failures for a specific panel
    pub fn panel_total_failures(&self, name: &str) -> u32 {
        self.panels
            .get(name)
            .map(|p| p.total_failures)
            .unwrap_or(0)
    }

    /// Check if panel was disabled
    pub fn is_panel_disabled(&self, name: &str) -> bool {
        self.panels
            .get(name)
            .map(|p| !p.enabled)
            .unwrap_or(false)
    }

    /// Get disabled reason for a panel
    pub fn panel_disabled_reason(&self, name: &str) -> Option<String> {
        self.panels
            .get(name)
            .and_then(|p| p.disabled_reason.clone())
    }

    pub fn panel_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Editor Panel Health Report ===\n");
        report.push_str(&format!("Safe mode: {}\n", if self.safe_mode_active { "ACTIVE" } else { "off" }));
        report.push_str(&format!("Last frame editor cost: {:.2}ms / {:.2}ms budget\n",
            self.last_frame_editor_ms, self.total_editor_budget_ms));

        let mut panels: Vec<_> = self.panels.values().collect();
        panels.sort_by(|a, b| a.name.cmp(&b.name));

        for p in panels {
            let status = if !p.enabled {
                format!("DISABLED ({})", p.disabled_reason.as_deref().unwrap_or("unknown"))
            } else {
                format!("ok ({:.2}ms / {:.2}ms)", p.last_draw_ms, p.budget_ms)
            };
            report.push_str(&format!("  [{}] {} — failures: {}\n",
                p.name, status, p.total_failures));
        }
        report
    }

    pub fn panel_budgets(&self) -> Vec<(&str, f64, f64)> {
        self.panels.values()
            .map(|p| (p.name.as_str(), p.last_draw_ms, p.budget_ms))
            .collect()
    }
}
