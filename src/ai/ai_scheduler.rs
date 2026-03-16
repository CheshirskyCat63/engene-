//! AI Tick Staggering (Phase C.3)
//! 
//! Provides budget-aware AI processing with priority-based scheduling.
//! Guarantees each entity is ticked at least once every N frames.

use std::collections::HashMap;
use crate::core::ecs::Entity;
use crate::world::components::{AiState, Goal, PersonalNeeds, SimulationLevel};

/// Default AI budget as percentage of frame time (25% of 16.6ms = ~4ms)
pub const DEFAULT_AI_BUDGET_US: u64 = 4000;

/// Maximum frames an entity can be skipped before forced tick
pub const MAX_SKIP_FRAMES: u32 = 3;

/// Priority levels for AI scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiPriority {
    /// Idle, low-priority entities
    Idle = 0,
    /// Normal gameplay
    Normal = 1,
    /// Hungry, seeking resources
    Urgent = 2,
    /// In combat or fleeing
    Critical = 3,
}

/// Per-entity scheduling state
#[derive(Debug, Clone)]
pub struct EntitySchedule {
    pub last_tick_frame: u64,
    pub skip_count: u32,
    pub priority: AiPriority,
}

/// AI Scheduler that manages tick distribution across frames
pub struct AiScheduler {
    /// Per-entity scheduling state
    schedules: HashMap<Entity, EntitySchedule>,
    /// Current frame number
    frame: u64,
    /// Budget in microseconds
    budget_us: u64,
    /// Stats
    entities_ticked: u32,
    budget_misses: u32,
    forced_ticks: u32,
}

impl AiScheduler {
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            frame: 0,
            budget_us: DEFAULT_AI_BUDGET_US,
            entities_ticked: 0,
            budget_misses: 0,
            forced_ticks: 0,
        }
    }

    pub fn set_budget(&mut self, budget_us: u64) {
        self.budget_us = budget_us;
    }

    /// Begin a new frame, reset stats
    pub fn begin_frame(&mut self) {
        self.frame += 1;
        self.entities_ticked = 0;
        self.budget_misses = 0;
        self.forced_ticks = 0;
    }

    /// Compute priority for an entity based on its state
    pub fn compute_priority(
        ai_state: Option<&AiState>,
        needs: Option<&PersonalNeeds>,
        sim_level: SimulationLevel,
    ) -> AiPriority {
        // L0 entities are always higher priority
        if sim_level != SimulationLevel::L0 {
            return AiPriority::Idle;
        }

        // In combat or fleeing
        if let Some(state) = ai_state {
            match state {
                AiState::Executing(goal) => match goal {
                    Goal::Flee | Goal::Hunt | Goal::DefendTerritory => return AiPriority::Critical,
                    Goal::SeekFood | Goal::SeekWater | Goal::SeekShelter => return AiPriority::Urgent,
                    _ => {}
                },
                AiState::Idle => {}
            }
        }

        // Low health or hunger
        if let Some(needs) = needs {
            if needs.health < 0.3 || needs.hunger > 0.7 {
                return AiPriority::Urgent;
            }
        }

        AiPriority::Normal
    }

    /// Register or update an entity's scheduling state
    pub fn update_entity(
        &mut self,
        entity: Entity,
        ai_state: Option<&AiState>,
        needs: Option<&PersonalNeeds>,
        sim_level: SimulationLevel,
    ) {
        let priority = Self::compute_priority(ai_state, needs, sim_level);
        
        self.schedules
            .entry(entity)
            .and_modify(|s| s.priority = priority)
            .or_insert(EntitySchedule {
                last_tick_frame: 0,
                skip_count: 0,
                priority,
            });
    }

    /// Remove an entity from scheduling
    pub fn remove_entity(&mut self, entity: Entity) {
        self.schedules.remove(&entity);
    }

    /// Check if an entity should be ticked this frame
    /// Returns true if: (1) budget allows, (2) priority is high, or (3) max skip reached
    pub fn should_tick(&mut self, entity: Entity, elapsed_us: u64) -> bool {
        let schedule = self.schedules.get_mut(&entity);
        
        // Unknown entity - tick it
        let schedule = match schedule {
            Some(s) => s,
            None => return true,
        };

        // Check if forced tick needed (max skip exceeded)
        if schedule.skip_count >= MAX_SKIP_FRAMES {
            schedule.skip_count = 0;
            schedule.last_tick_frame = self.frame;
            self.forced_ticks += 1;
            return true;
        }

        // Check budget
        if elapsed_us >= self.budget_us {
            self.budget_misses += 1;
            // Still tick high priority entities
            if schedule.priority >= AiPriority::Critical {
                schedule.last_tick_frame = self.frame;
                return true;
            }
            return false;
        }

        // Priority-based decision
        match schedule.priority {
            AiPriority::Critical => true,
            AiPriority::Urgent => schedule.skip_count >= 1 || self.frame % 2 == 0,
            AiPriority::Normal => self.frame % 2 == schedule.skip_count as u64 % 2,
            AiPriority::Idle => schedule.skip_count >= 2,
        }
    }

    /// Mark an entity as ticked this frame
    pub fn mark_ticked(&mut self, entity: Entity) {
        if let Some(schedule) = self.schedules.get_mut(&entity) {
            schedule.last_tick_frame = self.frame;
            schedule.skip_count = 0;
        }
        self.entities_ticked += 1;
    }

    /// Mark an entity as skipped this frame
    pub fn mark_skipped(&mut self, entity: Entity) {
        if let Some(schedule) = self.schedules.get_mut(&entity) {
            schedule.skip_count += 1;
        }
    }

    /// Get statistics for this frame
    pub fn stats(&self) -> AiSchedulerStats {
        AiSchedulerStats {
            frame: self.frame,
            total_entities: self.schedules.len() as u32,
            entities_ticked: self.entities_ticked,
            budget_misses: self.budget_misses,
            forced_ticks: self.forced_ticks,
            budget_us: self.budget_us,
        }
    }

    /// Get priority distribution
    pub fn priority_distribution(&self) -> [u32; 4] {
        let mut dist = [0u32; 4];
        for schedule in self.schedules.values() {
            dist[schedule.priority as usize] += 1;
        }
        dist
    }
}

impl Default for AiScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AiSchedulerStats {
    pub frame: u64,
    pub total_entities: u32,
    pub entities_ticked: u32,
    pub budget_misses: u32,
    pub forced_ticks: u32,
    pub budget_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_critical_flee() {
        let priority = AiScheduler::compute_priority(
            Some(&AiState::Executing(Goal::Flee)),
            None,
            SimulationLevel::L0,
        );
        assert_eq!(priority, AiPriority::Critical);
    }

    #[test]
    fn test_priority_urgent_hunger() {
        let mut needs = PersonalNeeds::default_npc();
        needs.hunger = 0.8;

        let priority = AiScheduler::compute_priority(
            None,
            Some(&needs),
            SimulationLevel::L0,
        );
        assert_eq!(priority, AiPriority::Urgent);
    }

    #[test]
    fn test_priority_idle_l2() {
        let priority = AiScheduler::compute_priority(
            None,
            None,
            SimulationLevel::L2,
        );
        assert_eq!(priority, AiPriority::Idle);
    }

    #[test]
    fn test_forced_tick_after_max_skip() {
        let mut scheduler = AiScheduler::new();
        let entity: Entity = 1;
        
        scheduler.update_entity(entity, None, None, SimulationLevel::L0);
        
        // Skip MAX_SKIP_FRAMES times
        for _ in 0..MAX_SKIP_FRAMES {
            scheduler.mark_skipped(entity);
        }
        
        // Should force tick
        assert!(scheduler.should_tick(entity, 0));
        
        let stats = scheduler.stats();
        assert_eq!(stats.forced_ticks, 1);
    }

    #[test]
    fn test_budget_miss() {
        let mut scheduler = AiScheduler::new();
        scheduler.set_budget(100); // Very small budget
        
        let entity: Entity = 1;
        scheduler.update_entity(entity, None, None, SimulationLevel::L0);
        
        // Budget exceeded
        assert!(!scheduler.should_tick(entity, 200));
        
        let stats = scheduler.stats();
        assert_eq!(stats.budget_misses, 1);
    }
}
