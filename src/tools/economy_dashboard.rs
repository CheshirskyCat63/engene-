//! Economy Dashboard — SDK panel showing wealth distribution, trade flow, and monthly payments.

use crate::core::ecs::Ecs;
use crate::world::components::EntityKind;

#[derive(Debug, Clone, Default)]
pub struct EconomySnapshot {
    pub total_money: f64,
    pub average_money: f64,
    pub min_money: f64,
    pub max_money: f64,
    pub npc_count: u32,
    pub bankrupt_count: u32,
    pub bandit_count: u32,
    pub monthly_payments_due: f64,
    pub month: u32,
}

pub struct EconomyDashboard {
    pub history: Vec<EconomySnapshot>,
    max_history: usize,
}

impl EconomyDashboard {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history,
        }
    }

    pub fn record_snapshot(&mut self, ecs: &Ecs, month: u32) {
        let mut snap = EconomySnapshot {
            month,
            min_money: f64::MAX,
            max_money: f64::MIN,
            ..Default::default()
        };

        for &e in ecs.alive() {
            if !matches!(ecs.get_kind(e), Some(EntityKind::Npc)) {
                continue;
            }
            snap.npc_count += 1;
            if let Some(econ) = ecs.get_npc_economy(e) {
                let m = econ.money as f64;
                snap.total_money += m;
                snap.min_money = snap.min_money.min(m);
                snap.max_money = snap.max_money.max(m);
                snap.monthly_payments_due += econ.monthly_required as f64;
                if econ.desperation > 0.9 {
                    snap.bankrupt_count += 1;
                }
            }
        }

        if snap.npc_count > 0 {
            snap.average_money = snap.total_money / snap.npc_count as f64;
        }
        if snap.min_money == f64::MAX {
            snap.min_money = 0.0;
        }
        if snap.max_money == f64::MIN {
            snap.max_money = 0.0;
        }

        self.history.push(snap);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn latest(&self) -> Option<&EconomySnapshot> {
        self.history.last()
    }

    pub fn wealth_trend(&self, last_n: usize) -> Vec<f64> {
        self.history
            .iter()
            .rev()
            .take(last_n)
            .rev()
            .map(|s| s.average_money)
            .collect()
    }
}

impl EconomyDashboard {
    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("Economy")
            .default_width(400.0)
            .show(ctx, |ui| {
                if let Some(s) = self.latest() {
                    ui.label(format!("Month {}", s.month));
                    ui.separator();
                    ui.label(format!(
                        "NPCs: {}  Bankrupt: {}  Bandits: {}",
                        s.npc_count, s.bankrupt_count, s.bandit_count
                    ));
                    ui.label(format!(
                        "Total money: {:.0}  Avg: {:.0}  Min: {:.0}  Max: {:.0}",
                        s.total_money, s.average_money, s.min_money, s.max_money
                    ));
                    ui.label(format!(
                        "Monthly payments due: {:.0}",
                        s.monthly_payments_due
                    ));
                } else {
                    ui.label("No economy data yet.");
                }
            });
    }
}

impl Default for EconomyDashboard {
    fn default() -> Self {
        Self::new(100)
    }
}
