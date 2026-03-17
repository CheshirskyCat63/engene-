use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::core::persistent_id::PersistentEntityId;

const MAX_EVENTS: usize = 30;
const MAX_LESSONS: usize = 20;
const MAX_SPATIAL: usize = 200;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub events: VecDeque<EventMemory>,
    pub spatial: HashMap<(u32, u32), CellKnowledge>,
    pub entities: HashMap<PersistentEntityId, EntityOpinion>,
    pub lessons: VecDeque<Lesson>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            spatial: HashMap::new(),
            entities: HashMap::new(),
            lessons: VecDeque::new(),
        }
    }

    pub fn record_event(&mut self, event: EventMemory) {
        self.events.push_back(event);
        if self.events.len() > MAX_EVENTS {
            self.events.pop_front();
        }
    }

    pub fn mark_cell(&mut self, cx: u32, cy: u32, tag: CellTag, strength: f32) {
        let k = self.spatial.entry((cx, cy)).or_insert(CellKnowledge::default());
        match tag {
            CellTag::Food => k.food = (k.food + strength).min(1.0),
            CellTag::Danger => k.danger = (k.danger + strength).min(1.0),
            CellTag::Ally => k.ally_presence = (k.ally_presence + strength).min(1.0),
            CellTag::Shelter => k.shelter = (k.shelter + strength).min(1.0),
        }
    }

    pub fn cell_danger(&self, cx: u32, cy: u32) -> f32 {
        self.spatial.get(&(cx, cy)).map_or(0.0, |k| k.danger)
    }

    pub fn opinion_of(&self, other: PersistentEntityId) -> &EntityOpinion {
        static DEFAULT: EntityOpinion = EntityOpinion {
            trust: 0.0,
            hostility: 0.0,
            familiarity: 0.0,
            last_seen_tick: 0,
            shared_kills: 0,
            times_hurt_me: 0,
        };
        self.entities.get(&other).unwrap_or(&DEFAULT)
    }

    pub fn adjust_opinion(&mut self, other: PersistentEntityId, f: impl FnOnce(&mut EntityOpinion)) {
        let op = self.entities.entry(other).or_insert(EntityOpinion::default());
        f(op);
        op.trust = op.trust.clamp(-1.0, 1.0);
        op.hostility = op.hostility.clamp(0.0, 1.0);
        op.familiarity = op.familiarity.clamp(0.0, 1.0);
    }

    pub fn best_ally(&self) -> Option<PersistentEntityId> {
        self.entities.iter()
            .filter(|(_, op)| op.trust > 0.3)
            .max_by(|a, b| a.1.trust.total_cmp(&b.1.trust))
            .map(|(&e, _)| e)
    }

    pub fn record_lesson(&mut self, lesson: Lesson) {
        if let Some(existing) = self.lessons.iter_mut().find(|l| l.action == lesson.action && l.context == lesson.context) {
            existing.attempts += lesson.attempts;
            existing.successes += lesson.successes;
        } else {
            self.lessons.push_back(lesson);
            if self.lessons.len() > MAX_LESSONS {
                self.lessons.pop_front();
            }
        }
    }

    pub fn lesson_score(&self, action: LessonAction, context: LessonContext) -> f32 {
        self.lessons.iter()
            .find(|l| l.action == action && l.context == context)
            .map_or(0.5, |l| {
                if l.attempts == 0 { 0.5 } else { l.successes as f32 / l.attempts as f32 }
            })
    }

    pub fn decay_spatial(&mut self, rate: f32) {
        for k in self.spatial.values_mut() {
            k.food = (k.food - rate).max(0.0);
            k.danger = (k.danger - rate * 0.5).max(0.0);
            k.ally_presence = (k.ally_presence - rate).max(0.0);
            k.shelter = (k.shelter - rate * 0.3).max(0.0);
        }
        self.spatial.retain(|_, k| k.food > 0.01 || k.danger > 0.01 || k.ally_presence > 0.01 || k.shelter > 0.01);
        if self.spatial.len() > MAX_SPATIAL {
            let to_remove: Vec<_> = self.spatial.iter()
                .map(|(&k, v)| (k, v.food + v.danger + v.ally_presence + v.shelter))
                .collect();
            let mut sorted = to_remove;
            sorted.sort_by(|a, b| a.1.total_cmp(&b.1));
            let remove_count = self.spatial.len() - MAX_SPATIAL;
            for (k, _) in sorted.into_iter().take(remove_count) {
                self.spatial.remove(&k);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventMemory {
    pub tick: u64,
    pub kind: EventKind,
    pub location: (u32, u32),
    pub other: Option<PersistentEntityId>,
    pub emotional_impact: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    KilledTarget,
    WasAttacked,
    AllyDied,
    GroupHuntWin,
    GroupHuntFail,
    FledFromPredator,
    SawTheft,
    Traded,
    Socialized,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CellKnowledge {
    pub food: f32,
    pub danger: f32,
    pub ally_presence: f32,
    pub shelter: f32,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CellTag {
    Food,
    Danger,
    Ally,
    Shelter,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntityOpinion {
    pub trust: f32,
    pub hostility: f32,
    pub familiarity: f32,
    pub last_seen_tick: u64,
    pub shared_kills: u32,
    pub times_hurt_me: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lesson {
    pub action: LessonAction,
    pub context: LessonContext,
    pub attempts: u32,
    pub successes: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LessonAction {
    SoloHunt,
    GroupHunt,
    Flee,
    DefendTerritory,
    Steal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LessonContext {
    VsWolf,
    VsBoar,
    VsBloodsucker,
    VsNpc,
    InCell(u32, u32),
    General,
}

pub fn context_for_kind(kind: &crate::world::components::EntityKind) -> LessonContext {
    match kind {
        crate::world::components::EntityKind::Npc => LessonContext::VsNpc,
        crate::world::components::EntityKind::Monster(s) => match s {
            crate::world::components::MonsterSpecies::Wolf => LessonContext::VsWolf,
            crate::world::components::MonsterSpecies::Boar => LessonContext::VsBoar,
            crate::world::components::MonsterSpecies::Bloodsucker => LessonContext::VsBloodsucker,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pid(id: u64) -> PersistentEntityId {
        PersistentEntityId(id)
    }

    #[test]
    fn test_record_event() {
        let mut mem = Memory::new();
        
        mem.record_event(EventMemory {
            tick: 1,
            kind: EventKind::KilledTarget,
            location: (0, 0),
            other: None,
            emotional_impact: 0.5,
        });
        
        assert_eq!(mem.events.len(), 1);
    }

    #[test]
    fn test_event_capacity_limit() {
        let mut mem = Memory::new();
        
        for i in 0..50 {
            mem.record_event(EventMemory {
                tick: i,
                kind: EventKind::Traded,
                location: (0, 0),
                other: None,
                emotional_impact: 0.1,
            });
        }
        
        assert_eq!(mem.events.len(), MAX_EVENTS);
    }

    #[test]
    fn test_mark_cell() {
        let mut mem = Memory::new();
        
        mem.mark_cell(5, 5, CellTag::Danger, 0.8);
        mem.mark_cell(5, 5, CellTag::Food, 0.5);
        
        let danger = mem.cell_danger(5, 5);
        assert!((danger - 0.8).abs() < 0.01);
        
        let knowledge = mem.spatial.get(&(5, 5)).unwrap();
        assert!((knowledge.food - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_opinion_of() {
        let mut mem = Memory::new();
        let pid = make_pid(42);
        
        // Default opinion
        let op = mem.opinion_of(pid);
        assert_eq!(op.trust, 0.0);
        
        // Adjust opinion
        mem.adjust_opinion(pid, |o| {
            o.trust = 0.5;
            o.hostility = 0.2;
        });
        
        let op = mem.opinion_of(pid);
        assert!((op.trust - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_opinion_clamping() {
        let mut mem = Memory::new();
        let pid = make_pid(1);
        
        mem.adjust_opinion(pid, |o| o.trust = 5.0);
        let op = mem.opinion_of(pid);
        assert_eq!(op.trust, 1.0); // Clamped to max
        
        mem.adjust_opinion(pid, |o| o.trust = -5.0);
        let op = mem.opinion_of(pid);
        assert_eq!(op.trust, -1.0); // Clamped to min
    }

    #[test]
    fn test_lesson_recording() {
        let mut mem = Memory::new();
        
        mem.record_lesson(Lesson {
            action: LessonAction::SoloHunt,
            context: LessonContext::VsWolf,
            attempts: 1,
            successes: 1,
        });
        
        let score = mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf);
        assert!((score - 1.0).abs() < 0.01);
        
        // Record again - should aggregate
        mem.record_lesson(Lesson {
            action: LessonAction::SoloHunt,
            context: LessonContext::VsWolf,
            attempts: 1,
            successes: 0,
        });
        
        let score = mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf);
        assert!((score - 0.5).abs() < 0.01); // 1 success / 2 attempts
    }

    #[test]
    fn test_best_ally() {
        let mut mem = Memory::new();
        
        // No allies yet
        assert!(mem.best_ally().is_none());
        
        // Add some entities with different trust
        mem.adjust_opinion(make_pid(1), |o| o.trust = 0.1); // Below threshold
        mem.adjust_opinion(make_pid(2), |o| o.trust = 0.5);
        mem.adjust_opinion(make_pid(3), |o| o.trust = 0.8);
        
        let best = mem.best_ally();
        assert_eq!(best, Some(make_pid(3)));
    }

    #[test]
    fn test_decay_spatial() {
        let mut mem = Memory::new();
        
        mem.mark_cell(0, 0, CellTag::Danger, 0.5);
        mem.mark_cell(1, 1, CellTag::Food, 0.01); // Will decay to near-zero
        
        mem.decay_spatial(0.1);
        
        let danger = mem.cell_danger(0, 0);
        assert!(danger < 0.5); // Should have decayed
        assert!(danger > 0.01); // But still present
    }
}
