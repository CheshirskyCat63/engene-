//! Simulation-level telemetry counters for the Sim Metrics Dashboard.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SimTelemetry {
    pub entity_births: u64,
    pub entity_deaths: u64,
    pub combat_events: u64,
    pub quests_generated: u64,
    pub quests_completed: u64,
    pub quests_failed: u64,
    pub bankruptcies: u64,
    pub banditizations: u64,
    pub migrations: u64,
    pub reproduction_events: u64,
    pub monthly_payments_processed: u64,

    per_species_births: HashMap<String, u64>,
    per_species_deaths: HashMap<String, u64>,
}

impl SimTelemetry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_birth(&mut self, species: &str) {
        self.entity_births += 1;
        *self
            .per_species_births
            .entry(species.to_string())
            .or_default() += 1;
    }

    pub fn record_death(&mut self, species: &str) {
        self.entity_deaths += 1;
        *self
            .per_species_deaths
            .entry(species.to_string())
            .or_default() += 1;
    }

    pub fn record_combat(&mut self) {
        self.combat_events += 1;
    }

    pub fn record_quest_generated(&mut self) {
        self.quests_generated += 1;
    }

    pub fn record_quest_completed(&mut self) {
        self.quests_completed += 1;
    }

    pub fn record_quest_failed(&mut self) {
        self.quests_failed += 1;
    }

    pub fn record_bankruptcy(&mut self) {
        self.bankruptcies += 1;
    }

    pub fn record_banditization(&mut self) {
        self.banditizations += 1;
    }

    pub fn species_births(&self, species: &str) -> u64 {
        self.per_species_births.get(species).copied().unwrap_or(0)
    }

    pub fn species_deaths(&self, species: &str) -> u64 {
        self.per_species_deaths.get(species).copied().unwrap_or(0)
    }
}
