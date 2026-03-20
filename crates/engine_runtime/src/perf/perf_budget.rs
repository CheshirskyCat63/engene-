use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SystemPerfBudget {
    pub name: String,
    pub budget_us: u32,
    pub last_measured_us: u32,
    pub avg_measured_us: u32,
    pub peak_measured_us: u32,
    pub samples: u32,
    pub overrun_count: u32,
}

impl SystemPerfBudget {
    pub fn new(name: &str, budget_us: u32) -> Self {
        Self {
            name: name.to_string(),
            budget_us,
            last_measured_us: 0,
            avg_measured_us: 0,
            peak_measured_us: 0,
            samples: 0,
            overrun_count: 0,
        }
    }

    pub fn record(&mut self, measured_us: u32) {
        self.last_measured_us = measured_us;
        self.peak_measured_us = self.peak_measured_us.max(measured_us);
        self.samples += 1;
        let alpha = 0.1;
        self.avg_measured_us =
            (self.avg_measured_us as f32 * (1.0 - alpha) + measured_us as f32 * alpha) as u32;
        if measured_us > self.budget_us {
            self.overrun_count += 1;
        }
    }

    pub fn is_over_budget(&self) -> bool {
        self.avg_measured_us > self.budget_us
    }

    pub fn utilization(&self) -> f32 {
        if self.budget_us == 0 {
            return 0.0;
        }
        self.avg_measured_us as f32 / self.budget_us as f32
    }
}

pub struct PerfBudgetManager {
    budgets: HashMap<String, SystemPerfBudget>,
    total_frame_budget_us: u32,
}

impl PerfBudgetManager {
    pub fn new(target_fps: u32) -> Self {
        let frame_budget = 1_000_000 / target_fps;
        let mut mgr = Self {
            budgets: HashMap::new(),
            total_frame_budget_us: frame_budget,
        };
        mgr.register_defaults(frame_budget);
        mgr
    }

    fn register_defaults(&mut self, frame_budget: u32) {
        let allocations = [
            ("AI", 0.25),
            ("Physics", 0.15),
            ("Simulation", 0.10),
            ("Economy", 0.05),
            ("Animation", 0.05),
            ("Render", 0.25),
            ("Audio", 0.03),
            ("Navigation", 0.05),
            ("Body", 0.02),
            ("Streaming", 0.03),
            ("Overhead", 0.02),
        ];
        for (name, fraction) in allocations {
            let budget = (frame_budget as f32 * fraction) as u32;
            self.budgets
                .insert(name.to_string(), SystemPerfBudget::new(name, budget));
        }
    }

    pub fn record(&mut self, system_name: &str, measured_us: u32) {
        if let Some(budget) = self.budgets.get_mut(system_name) {
            budget.record(measured_us);
        }
    }

    pub fn get(&self, system_name: &str) -> Option<&SystemPerfBudget> {
        self.budgets.get(system_name)
    }

    pub fn overrun_systems(&self) -> Vec<&SystemPerfBudget> {
        self.budgets
            .values()
            .filter(|b| b.is_over_budget())
            .collect()
    }

    pub fn total_utilization(&self) -> f32 {
        let total_used: u32 = self.budgets.values().map(|b| b.avg_measured_us).sum();
        total_used as f32 / self.total_frame_budget_us as f32
    }

    pub fn summary(&self) -> String {
        let mut lines = vec![format!(
            "Frame budget: {}us ({:.0}% utilized)",
            self.total_frame_budget_us,
            self.total_utilization() * 100.0
        )];
        let mut entries: Vec<_> = self.budgets.values().collect();
        entries.sort_by(|a, b| b.avg_measured_us.cmp(&a.avg_measured_us));
        for b in entries {
            let marker = if b.is_over_budget() { "OVER" } else { "ok" };
            lines.push(format!(
                "  {} [{}]: {}/{}us ({:.0}%) peak={}us overruns={}",
                b.name,
                marker,
                b.avg_measured_us,
                b.budget_us,
                b.utilization() * 100.0,
                b.peak_measured_us,
                b.overrun_count
            ));
        }
        lines.join("\n")
    }
}

impl Default for PerfBudgetManager {
    fn default() -> Self {
        Self::new(60)
    }
}
