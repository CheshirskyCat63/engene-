//! Simulation Metrics Dashboard — tracks population, economy, quests, and ecology health.
//!
//! Without this, balancing A-life/economy/factions is guesswork.

use crate::core::ecs::Ecs;
use crate::world::components::{EntityKind, MonsterSpecies};

#[derive(Debug, Clone, Default)]
pub struct SimMetricsSnapshot {
    pub stalker_count: u32,
    pub wolf_count: u32,
    pub boar_count: u32,
    pub bloodsucker_count: u32,
    pub dead_count: u32,
    pub total_alive: u32,

    pub total_npc_money: f64,
    pub average_wealth: f64,
    pub min_wealth: f64,
    pub max_wealth: f64,
    pub bankruptcies: u32,

    pub quests_active: u32,
    pub quests_completed_total: u32,
    pub quests_failed_total: u32,

    pub average_desperation: f32,
    pub average_hunger: f32,
    pub average_health: f32,

    pub month: u32,
    pub day: u32,
}

pub struct SimMetricsDashboard {
    pub history: Vec<SimMetricsSnapshot>,
    max_history: usize,
}

impl SimMetricsDashboard {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history,
        }
    }

    pub fn record_snapshot(&mut self, ecs: &Ecs, month: u32, day: u32) {
        let mut snap = SimMetricsSnapshot {
            month,
            day,
            total_alive: ecs.alive.len() as u32,
            ..Default::default()
        };

        let mut npc_count = 0u32;
        let mut total_money = 0.0f64;
        let mut min_money = f64::MAX;
        let mut max_money = f64::MIN;
        let total_desperation = 0.0f32;
        let mut total_hunger = 0.0f32;
        let mut total_health = 0.0f32;

        for &e in &ecs.alive {
            match ecs.kinds.get(&e) {
                Some(EntityKind::Npc) => {
                    snap.stalker_count += 1;
                    npc_count += 1;
                    if let Some(econ) = ecs.npc_economies.get(&e) {
                        total_money += econ.money as f64;
                        min_money = min_money.min(econ.money as f64);
                        max_money = max_money.max(econ.money as f64);
                    }
                    if let Some(needs) = ecs.personal_needs.get(&e) {
                        total_health += needs.health;
                        total_hunger += needs.hunger;
                    }
                }
                Some(EntityKind::Monster(species)) => match species {
                    MonsterSpecies::Wolf => snap.wolf_count += 1,
                    MonsterSpecies::Boar => snap.boar_count += 1,
                    MonsterSpecies::Bloodsucker => snap.bloodsucker_count += 1,
                },
                None => {}
            }
        }

        if npc_count > 0 {
            let n = npc_count as f64;
            snap.total_npc_money = total_money;
            snap.average_wealth = total_money / n;
            snap.min_wealth = if min_money == f64::MAX { 0.0 } else { min_money };
            snap.max_wealth = if max_money == f64::MIN { 0.0 } else { max_money };
            snap.average_health = total_health / npc_count as f32;
            snap.average_hunger = total_hunger / npc_count as f32;
            snap.average_desperation = total_desperation / npc_count as f32;
        }

        self.history.push(snap);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn latest(&self) -> Option<&SimMetricsSnapshot> {
        self.history.last()
    }

    pub fn population_trend(&self, last_n: usize) -> Vec<(u32, u32)> {
        self.history
            .iter()
            .rev()
            .take(last_n)
            .rev()
            .map(|s| (s.stalker_count, s.wolf_count + s.boar_count + s.bloodsucker_count))
            .collect()
    }
}

impl SimMetricsDashboard {
    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("Sim Metrics").default_width(420.0).show(ctx, |ui| {
            if let Some(s) = self.latest() {
                ui.heading(format!("Month {} Day {}", s.month, s.day));
                ui.separator();
                ui.label(format!("Stalkers: {}  Wolves: {}  Boars: {}  Bloodsuckers: {}",
                    s.stalker_count, s.wolf_count, s.boar_count, s.bloodsucker_count));
                ui.label(format!("Total alive: {}  Dead: {}", s.total_alive, s.dead_count));
                ui.separator();
                ui.label(format!("Avg wealth: {:.0}  Min: {:.0}  Max: {:.0}",
                    s.average_wealth, s.min_wealth, s.max_wealth));
                ui.label(format!("Quests active: {}  completed: {}  failed: {}",
                    s.quests_active, s.quests_completed_total, s.quests_failed_total));
                ui.label(format!("Avg health: {:.2}  hunger: {:.2}  desperation: {:.2}",
                    s.average_health, s.average_hunger, s.average_desperation));
            } else {
                ui.label("No snapshots recorded yet.");
            }
        });
    }
}

impl Default for SimMetricsDashboard {
    fn default() -> Self {
        Self::new(1000)
    }
}
