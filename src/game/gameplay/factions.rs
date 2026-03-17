// Phase 11: faction reputation system.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Loners,
    Duty,
    Freedom,
    Bandits,
    Military,
    Scientists,
    Traders,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionStance {
    Allied,
    Friendly,
    Neutral,
    Suspicious,
    Hostile,
}

impl FactionStance {
    pub fn is_hostile(&self) -> bool {
        matches!(self, FactionStance::Hostile)
    }

    pub fn can_trade(&self) -> bool {
        matches!(self, FactionStance::Allied | FactionStance::Friendly | FactionStance::Neutral)
    }
}

pub struct FactionRelations {
    relations: HashMap<(Faction, Faction), FactionStance>,
}

impl FactionRelations {
    pub fn new() -> Self {
        let mut rel = Self {
            relations: HashMap::new(),
        };
        rel.default_relations();
        rel
    }

    pub fn default_relations(&mut self) {
        let factions = [
            Faction::Loners,
            Faction::Duty,
            Faction::Freedom,
            Faction::Bandits,
            Faction::Military,
            Faction::Scientists,
            Faction::Traders,
        ];

        for &a in &factions {
            for &b in &factions {
                if a == b {
                    self.relations.insert((a, b), FactionStance::Allied);
                } else {
                    let stance = match (a, b) {
                        (Faction::Duty, Faction::Freedom) | (Faction::Freedom, Faction::Duty) => {
                            FactionStance::Hostile
                        }
                        (Faction::Bandits, _) | (_, Faction::Bandits) => {
                            if a == Faction::Bandits || b == Faction::Bandits {
                                FactionStance::Suspicious
                            } else {
                                FactionStance::Hostile
                            }
                        }
                        (Faction::Military, Faction::Freedom) | (Faction::Freedom, Faction::Military) => {
                            FactionStance::Hostile
                        }
                        (Faction::Traders, _) | (_, Faction::Traders) => FactionStance::Friendly,
                        (Faction::Scientists, _) | (_, Faction::Scientists) => FactionStance::Neutral,
                        _ => FactionStance::Neutral,
                    };
                    self.relations.insert((a, b), stance);
                }
            }
        }
    }

    pub fn stance_between(&self, a: Faction, b: Faction) -> FactionStance {
        self.relations
            .get(&(a, b))
            .copied()
            .unwrap_or(FactionStance::Neutral)
    }

    pub fn modify_relation(&mut self, a: Faction, b: Faction, delta: i32) {
        if a == b {
            return;
        }
        let current = self.stance_between(a, b);
        let ord = match current {
            FactionStance::Hostile => 0,
            FactionStance::Suspicious => 1,
            FactionStance::Neutral => 2,
            FactionStance::Friendly => 3,
            FactionStance::Allied => 4,
        };
        let new_ord = (ord as i32 + delta).clamp(0, 4);
        let new_stance = match new_ord {
            0 => FactionStance::Hostile,
            1 => FactionStance::Suspicious,
            2 => FactionStance::Neutral,
            3 => FactionStance::Friendly,
            _ => FactionStance::Allied,
        };
        self.relations.insert((a, b), new_stance);
    }

    pub fn is_hostile(&self, a: Faction, b: Faction) -> bool {
        self.stance_between(a, b).is_hostile()
    }

    pub fn can_trade(&self, a: Faction, b: Faction) -> bool {
        self.stance_between(a, b).can_trade()
    }
}

impl Default for FactionRelations {
    fn default() -> Self {
        Self::new()
    }
}
