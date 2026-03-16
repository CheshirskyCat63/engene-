use serde::{Deserialize, Serialize};
use crate::ai::combat_tactics::threat::ThreatEntry;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tactic {
    Flank,
    Charge,
    Ambush,
    Retreat,
    Surround,
    HitAndRun,
    HoldGround,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TacticProfile {
    pub flank_weight: f32,
    pub charge_weight: f32,
    pub ambush_weight: f32,
    pub retreat_weight: f32,
    pub surround_weight: f32,
    pub hit_and_run_weight: f32,
    pub hold_ground_weight: f32,
    pub retreat_health_threshold: f32,
    pub retreat_ally_loss_ratio: f32,
    pub preferred_group_size: u32,
}

impl Default for TacticProfile {
    fn default() -> Self {
        Self {
            flank_weight: 0.3,
            charge_weight: 0.3,
            ambush_weight: 0.1,
            retreat_weight: 0.1,
            surround_weight: 0.1,
            hit_and_run_weight: 0.05,
            hold_ground_weight: 0.05,
            retreat_health_threshold: 0.2,
            retreat_ally_loss_ratio: 0.5,
            preferred_group_size: 3,
        }
    }
}

pub fn select_tactic(
    profile: &TacticProfile,
    threats: &[ThreatEntry],
    my_health: f32,
    ally_count: u32,
    has_cover: bool,
) -> Tactic {
    if my_health < profile.retreat_health_threshold {
        return Tactic::Retreat;
    }

    if threats.is_empty() {
        return Tactic::HoldGround;
    }

    let outnumbered = threats.len() as u32 > ally_count;
    let strongest_threat = threats.first().map(|t| t.threat_score).unwrap_or(0.0);

    let mut scores = [
        (Tactic::Flank, profile.flank_weight),
        (Tactic::Charge, profile.charge_weight),
        (Tactic::Ambush, profile.ambush_weight),
        (Tactic::Retreat, profile.retreat_weight),
        (Tactic::Surround, profile.surround_weight),
        (Tactic::HitAndRun, profile.hit_and_run_weight),
        (Tactic::HoldGround, profile.hold_ground_weight),
    ];

    if ally_count >= profile.preferred_group_size {
        scores[4].1 *= 2.0; // Surround
        scores[0].1 *= 1.5; // Flank
    }

    if outnumbered {
        scores[3].1 *= 2.0; // Retreat
        scores[5].1 *= 1.5; // HitAndRun
    }

    if has_cover {
        scores[2].1 *= 2.0; // Ambush
    }

    if strongest_threat < 5.0 {
        scores[1].1 *= 2.0; // Charge
    }

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores[0].0
}
