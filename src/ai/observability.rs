/// Per-entity AI introspection snapshot
#[derive(Clone, Debug)]
pub struct AiIntrospection {
    pub entity: u64,
    pub current_goal: String,
    pub confidence: f32,
    pub chosen_plan: String,
    pub plan_reason: String,
    pub rejected_alternatives: Vec<(String, String)>,
    pub group_role: Option<String>,
    pub memory_driver_count: usize,
    pub social_influence_count: usize,
    pub stuck_ticks: u32,
    pub goal_oscillation_count: u32,
}

/// Tiered simulation test configuration
#[derive(Clone, Debug)]
pub struct SimTestConfig {
    pub tier: SimTestTier,
    pub simulated_ticks: u64,
    pub seed: u64,
    pub entity_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimTestTier {
    /// 10 minutes simulated, every build
    Quick,
    /// 1 hour simulated, CI nightly
    Medium,
    /// 6-24h simulated, periodic soak test
    Soak,
}

impl SimTestConfig {
    pub fn quick(seed: u64) -> Self {
        Self {
            tier: SimTestTier::Quick,
            simulated_ticks: 12_000,
            seed,
            entity_count: 200,
        }
    }
    pub fn medium(seed: u64) -> Self {
        Self {
            tier: SimTestTier::Medium,
            simulated_ticks: 72_000,
            seed,
            entity_count: 400,
        }
    }
    pub fn soak(seed: u64) -> Self {
        Self {
            tier: SimTestTier::Soak,
            simulated_ticks: 864_000,
            seed,
            entity_count: 500,
        }
    }
}

/// Stability metrics from a simulation test run
#[derive(Clone, Debug, Default)]
pub struct StabilityMetrics {
    pub population_start: usize,
    pub population_end: usize,
    pub total_births: u32,
    pub total_deaths: u32,
    pub money_total_start: f64,
    pub money_total_end: f64,
    pub stuck_entities: u32,
    pub path_failures: u32,
    pub event_storms: u32,
    pub goal_oscillations: u32,
    pub nan_detected: bool,
    pub crash_detected: bool,
}

impl StabilityMetrics {
    pub fn is_stable(&self) -> bool {
        !self.nan_detected
            && !self.crash_detected
            && self.stuck_entities < 10
            && self.event_storms < 5
            && self.population_end > 0
    }

    pub fn population_drift(&self) -> f64 {
        if self.population_start == 0 {
            return 0.0;
        }
        (self.population_end as f64 - self.population_start as f64) / self.population_start as f64
    }

    pub fn economy_drift(&self) -> f64 {
        if self.money_total_start < 1.0 {
            return 0.0;
        }
        (self.money_total_end - self.money_total_start) / self.money_total_start
    }
}
