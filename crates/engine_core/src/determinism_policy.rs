//! Determinism Policy Matrix — Contract 7 (Parallel Ownership Model).
//!
//! Classifies every engine system by its determinism requirement.
//! Used by replay validation and parallel execution planning.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeterminismLevel {
    /// Must produce identical results given identical inputs. Replay-critical.
    Required,
    /// Should produce identical results when practical. Same-cost tiebreaking preferred deterministic.
    Preferred,
    /// Non-deterministic acceptable. Cosmetic or UI-only systems.
    Acceptable,
}

impl std::fmt::Display for DeterminismLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "DETERMINISTIC_REQUIRED"),
            Self::Preferred => write!(f, "DETERMINISTIC_PREFERRED"),
            Self::Acceptable => write!(f, "NON_DETERMINISTIC_OK"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeterminismEntry {
    pub system_name: String,
    pub level: DeterminismLevel,
    pub notes: String,
}

pub struct DeterminismPolicyMatrix {
    entries: Vec<DeterminismEntry>,
    by_name: HashMap<String, DeterminismLevel>,
}

impl DeterminismPolicyMatrix {
    pub fn build_default() -> Self {
        let entries = vec![
            // REQUIRED: these must produce bit-identical results for replay
            DeterminismEntry {
                system_name: "ECS tick order".into(),
                level: DeterminismLevel::Required,
                notes: "Entity processing order fixed by PID sort".into(),
            },
            DeterminismEntry {
                system_name: "Entity spawn order".into(),
                level: DeterminismLevel::Required,
                notes: "spawn_new() PID assignment is sequential".into(),
            },
            DeterminismEntry {
                system_name: "AI decision evaluation".into(),
                level: DeterminismLevel::Required,
                notes: "Same inputs -> same goal selection".into(),
            },
            DeterminismEntry {
                system_name: "Quest generation".into(),
                level: DeterminismLevel::Required,
                notes: "Seeded RNG for quest type/reward".into(),
            },
            DeterminismEntry {
                system_name: "Economy calculations".into(),
                level: DeterminismLevel::Required,
                notes: "Monthly payments, trade prices".into(),
            },
            DeterminismEntry {
                system_name: "Combat resolution".into(),
                level: DeterminismLevel::Required,
                notes: "Damage, hit zones, death".into(),
            },
            DeterminismEntry {
                system_name: "Reproduction timing".into(),
                level: DeterminismLevel::Required,
                notes: "Population balance depends on this".into(),
            },
            DeterminismEntry {
                system_name: "SimulationSystem".into(),
                level: DeterminismLevel::Required,
                notes: "L0-L3 transitions".into(),
            },
            DeterminismEntry {
                system_name: "WorldTickSystem".into(),
                level: DeterminismLevel::Required,
                notes: "Food regen, danger, ecosystem".into(),
            },
            // PREFERRED: should be deterministic but not replay-critical
            DeterminismEntry {
                system_name: "Pathfinding".into(),
                level: DeterminismLevel::Preferred,
                notes: "Same cost -> same path preferred".into(),
            },
            DeterminismEntry {
                system_name: "Event ordering".into(),
                level: DeterminismLevel::Preferred,
                notes: "Sort by (tick, source_pid, event_type) after merge".into(),
            },
            DeterminismEntry {
                system_name: "PhysicsSystem".into(),
                level: DeterminismLevel::Preferred,
                notes: "Rapier is mostly deterministic with fixed timestep".into(),
            },
            // ACCEPTABLE: cosmetic, non-determinism is fine
            DeterminismEntry {
                system_name: "Render order".into(),
                level: DeterminismLevel::Acceptable,
                notes: "Visual only".into(),
            },
            DeterminismEntry {
                system_name: "Audio scheduling".into(),
                level: DeterminismLevel::Acceptable,
                notes: "Playback timing cosmetic".into(),
            },
            DeterminismEntry {
                system_name: "Editor panel layout".into(),
                level: DeterminismLevel::Acceptable,
                notes: "SDK UI state".into(),
            },
            DeterminismEntry {
                system_name: "Particle visuals".into(),
                level: DeterminismLevel::Acceptable,
                notes: "Cosmetic effects".into(),
            },
            DeterminismEntry {
                system_name: "Surface state decay".into(),
                level: DeterminismLevel::Acceptable,
                notes: "Blood/burn mark fading is visual".into(),
            },
        ];

        let by_name: HashMap<String, DeterminismLevel> = entries
            .iter()
            .map(|e| (e.system_name.clone(), e.level))
            .collect();

        Self { entries, by_name }
    }

    pub fn level_of(&self, system_name: &str) -> DeterminismLevel {
        self.by_name
            .get(system_name)
            .copied()
            .unwrap_or(DeterminismLevel::Preferred)
    }

    pub fn required_systems(&self) -> Vec<&DeterminismEntry> {
        self.entries
            .iter()
            .filter(|e| e.level == DeterminismLevel::Required)
            .collect()
    }

    pub fn all_entries(&self) -> &[DeterminismEntry] {
        &self.entries
    }
}

impl Default for DeterminismPolicyMatrix {
    fn default() -> Self {
        Self::build_default()
    }
}
