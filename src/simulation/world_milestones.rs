use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldMilestoneTracker {
    pub bankruptcies: u32,
    pub banditizations: u32,
    pub quest_completions: u32,
    pub quest_failures: u32,
    pub trade_events: u32,
    pub monster_territory_shifts: u32,
    pub npc_deaths: u32,
    pub npc_births: u32,
    pub resource_scarcity_events: u32,
    pub months_tracked: u32,
    pub milestones_achieved: Vec<WorldMilestone>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldMilestone {
    pub name: String,
    pub description: String,
    pub month_achieved: u32,
}

impl WorldMilestoneTracker {
    pub fn new() -> Self {
        Self {
            bankruptcies: 0,
            banditizations: 0,
            quest_completions: 0,
            quest_failures: 0,
            trade_events: 0,
            monster_territory_shifts: 0,
            npc_deaths: 0,
            npc_births: 0,
            resource_scarcity_events: 0,
            months_tracked: 0,
            milestones_achieved: Vec::new(),
        }
    }

    pub fn record_bankruptcy(&mut self) {
        self.bankruptcies += 1;
    }
    pub fn record_banditization(&mut self) {
        self.banditizations += 1;
    }
    pub fn record_quest_completion(&mut self) {
        self.quest_completions += 1;
    }
    pub fn record_quest_failure(&mut self) {
        self.quest_failures += 1;
    }
    pub fn record_trade(&mut self) {
        self.trade_events += 1;
    }
    pub fn record_territory_shift(&mut self) {
        self.monster_territory_shifts += 1;
    }
    pub fn record_npc_death(&mut self) {
        self.npc_deaths += 1;
    }
    pub fn record_npc_birth(&mut self) {
        self.npc_births += 1;
    }

    pub fn check_milestones(&mut self, current_month: u32) {
        self.months_tracked = current_month;

        if self.bankruptcies >= 3 && !self.has_milestone("First Economic Crisis") {
            self.milestones_achieved.push(WorldMilestone {
                name: "First Economic Crisis".into(),
                description: "3+ NPCs went bankrupt".into(),
                month_achieved: current_month,
            });
        }
        if self.banditizations >= 2 && !self.has_milestone("Bandit Rise") {
            self.milestones_achieved.push(WorldMilestone {
                name: "Bandit Rise".into(),
                description: "2+ NPCs turned to banditry".into(),
                month_achieved: current_month,
            });
        }
        if self.quest_completions >= 10 && !self.has_milestone("Quest Economy Active") {
            self.milestones_achieved.push(WorldMilestone {
                name: "Quest Economy Active".into(),
                description: "10+ quests completed".into(),
                month_achieved: current_month,
            });
        }
        if self.trade_events >= 20 && !self.has_milestone("Trading Hub") {
            self.milestones_achieved.push(WorldMilestone {
                name: "Trading Hub".into(),
                description: "20+ trades executed".into(),
                month_achieved: current_month,
            });
        }
    }

    fn has_milestone(&self, name: &str) -> bool {
        self.milestones_achieved.iter().any(|m| m.name == name)
    }

    pub fn summary(&self) -> String {
        format!(
            "Months: {} | Deaths: {} | Births: {} | Bankruptcies: {} | Bandits: {} | Quests: {}/{} | Trades: {} | Milestones: {}",
            self.months_tracked,
            self.npc_deaths,
            self.npc_births,
            self.bankruptcies,
            self.banditizations,
            self.quest_completions,
            self.quest_failures,
            self.trade_events,
            self.milestones_achieved.len()
        )
    }
}

impl Default for WorldMilestoneTracker {
    fn default() -> Self {
        Self::new()
    }
}
