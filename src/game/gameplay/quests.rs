// Phase 11: emergent quest generator.

use crate::core::persistent_id::PersistentEntityId;
use crate::world::cell::GRID_SIZE;
use crate::world::components::MonsterSpecies;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestType {
    FetchArtifact,
    KillMonsters,
    DeliverItem,
    ScoutArea,
    EscortNpc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Available,
    Active,
    Completed,
    Failed,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quest {
    pub id: u32,
    pub quest_type: QuestType,
    pub giver: PersistentEntityId,
    pub assignee: Option<PersistentEntityId>,
    pub target_cell: Option<(u32, u32)>,
    pub target_species: Option<MonsterSpecies>,
    pub target_count: u32,
    pub progress: u32,
    pub reward_money: f32,
    pub deadline_day: u32,
    pub status: QuestStatus,
}

impl Quest {
    pub fn is_available(&self) -> bool {
        self.status == QuestStatus::Available
    }

    pub fn is_active(&self) -> bool {
        self.status == QuestStatus::Active
    }
}

pub struct QuestRegistry {
    pub quests: Vec<Quest>,
    pub next_id: u32,
}

impl QuestRegistry {
    pub fn new() -> Self {
        Self {
            quests: Vec::new(),
            next_id: 1,
        }
    }

    pub fn generate_quest(
        &mut self,
        giver: PersistentEntityId,
        current_day: u32,
        rng: &mut impl Rng,
    ) -> &Quest {
        let id = self.next_id;
        self.next_id += 1;

        let quest_type = match rng.gen_range(0..5) {
            0 => QuestType::FetchArtifact,
            1 => QuestType::KillMonsters,
            2 => QuestType::DeliverItem,
            3 => QuestType::ScoutArea,
            _ => QuestType::EscortNpc,
        };

        let (target_cell, target_species, target_count) = match quest_type {
            QuestType::FetchArtifact | QuestType::ScoutArea => {
                let cx = rng.gen_range(0..GRID_SIZE);
                let cy = rng.gen_range(0..GRID_SIZE);
                (Some((cx, cy)), None, 1)
            }
            QuestType::KillMonsters => {
                let species = match rng.gen_range(0..3) {
                    0 => MonsterSpecies::Wolf,
                    1 => MonsterSpecies::Boar,
                    _ => MonsterSpecies::Bloodsucker,
                };
                let count = rng.gen_range(1..=5);
                (None, Some(species), count)
            }
            QuestType::DeliverItem | QuestType::EscortNpc => {
                let cx = rng.gen_range(0..GRID_SIZE);
                let cy = rng.gen_range(0..GRID_SIZE);
                (Some((cx, cy)), None, 1)
            }
        };

        let reward_money = rng.gen_range(20.0..150.0);
        let deadline_day = current_day + rng.gen_range(5..=30);

        let quest = Quest {
            id,
            quest_type,
            giver,
            assignee: None,
            target_cell,
            target_species,
            target_count,
            progress: 0,
            reward_money,
            deadline_day,
            status: QuestStatus::Available,
        };

        self.quests.push(quest);
        self.quests.last().unwrap()
    }

    pub fn assign_quest(
        &mut self,
        quest_id: u32,
        assignee: PersistentEntityId,
    ) -> bool {
        if let Some(q) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            if q.status == QuestStatus::Available {
                q.assignee = Some(assignee);
                q.status = QuestStatus::Active;
                return true;
            }
        }
        false
    }

    pub fn update_progress(&mut self, quest_id: u32, delta: u32) -> bool {
        if let Some(q) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            if q.status == QuestStatus::Active {
                q.progress = (q.progress + delta).min(q.target_count);
                return true;
            }
        }
        false
    }

    pub fn complete_quest(&mut self, quest_id: u32) -> bool {
        if let Some(q) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            if q.status == QuestStatus::Active {
                q.status = QuestStatus::Completed;
                return true;
            }
        }
        false
    }

    pub fn fail_quest(&mut self, quest_id: u32) -> bool {
        if let Some(q) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            if q.status == QuestStatus::Active {
                q.status = QuestStatus::Failed;
                return true;
            }
        }
        false
    }

    pub fn expire_quests(&mut self, current_day: u32) {
        for q in self.quests.iter_mut() {
            if q.status == QuestStatus::Active && current_day > q.deadline_day {
                q.status = QuestStatus::Expired;
            }
        }
    }

    pub fn expire_quest(&mut self, quest_id: u32) -> bool {
        if let Some(q) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            if q.status == QuestStatus::Available || q.status == QuestStatus::Active {
                q.status = QuestStatus::Expired;
                return true;
            }
        }
        false
    }

    pub fn available_quests(&self) -> impl Iterator<Item = &Quest> {
        self.quests
            .iter()
            .filter(|q| q.status == QuestStatus::Available)
    }

    pub fn active_quests_for(&self, assignee: PersistentEntityId) -> impl Iterator<Item = &Quest> {
        self.quests.iter().filter(move |q| {
            q.status == QuestStatus::Active && q.assignee == Some(assignee)
        })
    }

    pub fn get_quest(&self, quest_id: u32) -> Option<&Quest> {
        self.quests.iter().find(|q| q.id == quest_id)
    }

    pub fn get_quest_mut(&mut self, quest_id: u32) -> Option<&mut Quest> {
        self.quests.iter_mut().find(|q| q.id == quest_id)
    }
}

impl Default for QuestRegistry {
    fn default() -> Self {
        Self::new()
    }
}
