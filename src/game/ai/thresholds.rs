use crate::core::ai_memory::Memory;
use crate::world::components::{MonsterTraits, NpcTraits};

#[derive(Clone, Debug)]
pub struct Thresholds {
    pub health_panic: f32,
    pub fear_flee: f32,
    pub hunger_critical: f32,
    pub hunger_proactive: f32,
    pub thirst_critical: f32,
    pub sleep_critical: f32,
    pub energy_low: f32,
    pub loneliness_seek: f32,
    pub desperation_crime: f32,
}

impl Thresholds {
    pub fn from_npc(t: &NpcTraits, mem: Option<&Memory>) -> Self {
        let near_death_exp = mem.map_or(0.0, |m| {
            m.events
                .iter()
                .filter(|e| e.kind == crate::game::ai::memory::EventKind::WasAttacked)
                .count() as f32
                * 0.02
        });

        Self {
            health_panic: (0.15 + (1.0 - t.bravery) * 0.2 + near_death_exp).min(0.5),
            fear_flee: 0.5 + (1.0 - t.bravery) * 0.3,
            hunger_critical: 0.6 + (1.0 - t.stress_resistance) * 0.15,
            hunger_proactive: 0.3 + t.risk_tolerance * 0.15,
            thirst_critical: 0.7 + (1.0 - t.stress_resistance) * 0.1,
            sleep_critical: 0.65 + (1.0 - t.work_ethic) * 0.15,
            energy_low: 0.1 + (1.0 - t.work_ethic) * 0.1,
            loneliness_seek: 0.4 + (1.0 - t.sociality) * 0.2,
            desperation_crime: 0.4 + t.honesty * 0.3,
        }
    }

    pub fn from_monster(t: &MonsterTraits, mem: Option<&Memory>) -> Self {
        let near_death_exp = mem.map_or(0.0, |m| {
            m.events
                .iter()
                .filter(|e| e.kind == crate::game::ai::memory::EventKind::WasAttacked)
                .count() as f32
                * 0.03
        });

        Self {
            health_panic: (0.15 + (1.0 - t.bravery) * 0.25 + near_death_exp).min(0.5),
            fear_flee: 0.4 + t.caution * 0.4,
            hunger_critical: 0.55 + (1.0 - t.adaptability) * 0.2,
            hunger_proactive: 0.25 + t.aggressiveness * 0.2,
            thirst_critical: 0.7,
            sleep_critical: 0.6 + (1.0 - t.energy_level) * 0.2,
            energy_low: 0.1 + (1.0 - t.energy_level) * 0.15,
            loneliness_seek: 0.5,
            desperation_crime: 1.0,
        }
    }
}
