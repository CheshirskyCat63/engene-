use std::collections::HashMap;

/// A rumor spreading through NPC networks
#[derive(Clone, Debug)]
pub struct Rumor {
    pub id: u64,
    pub event_type: RumorType,
    pub origin_cell: (u32, u32),
    pub origin_tick: u64,
    pub spread_count: u32,
    pub credibility: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RumorType {
    DangerSighting,
    ResourceFound,
    TradingOpportunity,
    DeathReported,
    TerritoryConflict,
    MonsterPackSeen,
}

/// Camp/settlement mood tracking
#[derive(Clone, Debug)]
pub struct CampMood {
    pub cell: (u32, u32),
    pub morale: f32,
    pub fear_level: f32,
    pub prosperity: f32,
    pub recent_deaths: u32,
    pub trade_activity: f32,
}

impl CampMood {
    pub fn new(cell: (u32, u32)) -> Self {
        Self {
            cell,
            morale: 0.5,
            fear_level: 0.1,
            prosperity: 0.5,
            recent_deaths: 0,
            trade_activity: 0.5,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let death_penalty = self.recent_deaths as f32 * 0.05;
        self.morale =
            (self.morale + self.prosperity * 0.01 * dt - death_penalty * dt).clamp(0.0, 1.0);
        self.fear_level = (self.fear_level - 0.01 * dt).clamp(0.0, 1.0);
        if self.recent_deaths > 0 {
            self.fear_level = (self.fear_level + 0.1).clamp(0.0, 1.0);
        }
    }
}

/// Local trust shift between entities
#[derive(Clone, Debug)]
pub struct TrustShift {
    pub from: u64,
    pub to: u64,
    pub delta: f32,
    pub reason: TrustReason,
}

#[derive(Clone, Copy, Debug)]
pub enum TrustReason {
    WitnessedHelp,
    WitnessedAttack,
    SharedResource,
    Betrayal,
    TradedFairly,
    HeardRumor,
}

/// Social propagation system state
pub struct SocialPropagation {
    pub rumors: Vec<Rumor>,
    pub camp_moods: HashMap<(u32, u32), CampMood>,
    pub pending_trust_shifts: Vec<TrustShift>,
    next_rumor_id: u64,
}

impl SocialPropagation {
    pub fn new() -> Self {
        Self {
            rumors: Vec::new(),
            camp_moods: HashMap::new(),
            pending_trust_shifts: Vec::new(),
            next_rumor_id: 1,
        }
    }

    pub fn create_rumor(&mut self, event_type: RumorType, cell: (u32, u32), tick: u64) {
        self.rumors.push(Rumor {
            id: self.next_rumor_id,
            event_type,
            origin_cell: cell,
            origin_tick: tick,
            spread_count: 0,
            credibility: 1.0,
        });
        self.next_rumor_id += 1;
    }

    pub fn update(&mut self, dt: f32, current_tick: u64) {
        for mood in self.camp_moods.values_mut() {
            mood.update(dt);
        }
        self.rumors
            .retain(|r| current_tick.saturating_sub(r.origin_tick) < 72_000 && r.credibility > 0.1);
        for rumor in &mut self.rumors {
            rumor.credibility *= 0.999;
        }
    }

    pub fn rumor_count(&self) -> usize {
        self.rumors.len()
    }
}

// ── D.2b: Regional System Pressure ──────────────────────────────────

/// Regional resource scarcity tracking
#[derive(Clone, Debug)]
pub struct RegionalPressure {
    pub region: (u32, u32),
    pub scarcity_index: f32,
    pub danger_level: f32,
    pub bounties: Vec<Bounty>,
}

#[derive(Clone, Debug)]
pub struct Bounty {
    pub target_description: String,
    pub reward: f32,
    pub region: (u32, u32),
    pub created_tick: u64,
    pub claimed: bool,
}

/// Regional pressure system
pub struct RegionalPressureSystem {
    pub regions: HashMap<(u32, u32), RegionalPressure>,
}

impl RegionalPressureSystem {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    pub fn update_scarcity(&mut self, region: (u32, u32), food_available: f32, population: u32) {
        let pressure = self.regions.entry(region).or_insert(RegionalPressure {
            region,
            scarcity_index: 0.0,
            danger_level: 0.0,
            bounties: Vec::new(),
        });
        let demand = population as f32 * 0.1;
        pressure.scarcity_index = if food_available > 0.01 {
            (demand / food_available).clamp(0.0, 2.0)
        } else {
            2.0
        };
    }

    pub fn update_danger(&mut self, region: (u32, u32), monster_count: u32, recent_deaths: u32) {
        let pressure = self.regions.entry(region).or_insert(RegionalPressure {
            region,
            scarcity_index: 0.0,
            danger_level: 0.0,
            bounties: Vec::new(),
        });
        pressure.danger_level =
            (monster_count as f32 * 0.1 + recent_deaths as f32 * 0.2).clamp(0.0, 1.0);
    }

    pub fn generate_bounty(
        &mut self,
        region: (u32, u32),
        description: String,
        reward: f32,
        tick: u64,
    ) {
        if let Some(pressure) = self.regions.get_mut(&region) {
            if pressure.bounties.len() < 5 {
                pressure.bounties.push(Bounty {
                    target_description: description,
                    reward,
                    region,
                    created_tick: tick,
                    claimed: false,
                });
            }
        }
    }

    pub fn total_bounties(&self) -> usize {
        self.regions.values().map(|r| r.bounties.len()).sum()
    }
}
