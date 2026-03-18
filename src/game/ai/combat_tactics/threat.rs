use crate::core::ecs::{Ecs, Entity};
use crate::world::components::*;

#[derive(Clone, Debug)]
pub struct ThreatEntry {
    pub entity: Entity,
    pub threat_score: f32,
    pub distance: f32,
    pub health_ratio: f32,
    pub power: f32,
    pub group_size: u32,
}

/// Assess threats for an entity within a given radius.
///
/// Uses helper methods instead of direct storage access.
pub fn assess_threats(ecs: &Ecs, entity: Entity, radius: f32) -> Vec<ThreatEntry> {
    let pos = match ecs.get_transform(entity) {
        Some(t) => (t.x, t.y),
        None => return Vec::new(),
    };

    let my_kind = ecs.get_kind(entity);
    let mut threats = Vec::new();

    for &other in &ecs.alive {
        if other == entity {
            continue;
        }
        let other_pos = match ecs.get_transform(other) {
            Some(t) => (t.x, t.y),
            None => continue,
        };

        let dx = other_pos.0 - pos.0;
        let dy = other_pos.1 - pos.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > radius {
            continue;
        }

        let is_threat = match (my_kind, ecs.get_kind(other)) {
            (Some(EntityKind::Monster(_)), Some(EntityKind::Npc)) => true,
            (Some(EntityKind::Npc), Some(EntityKind::Monster(MonsterSpecies::Bloodsucker))) => true,
            (Some(EntityKind::Npc), Some(EntityKind::Monster(MonsterSpecies::Wolf))) => true,
            (
                Some(EntityKind::Monster(MonsterSpecies::Boar)),
                Some(EntityKind::Monster(MonsterSpecies::Wolf)),
            ) => true,
            (
                Some(EntityKind::Monster(MonsterSpecies::Wolf)),
                Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)),
            ) => true,
            _ => false,
        };
        if !is_threat {
            continue;
        }

        let health = ecs.get_needs(other).map_or(1.0, |pn| pn.health);
        let power = ecs.get_kind(other).map_or(5.0, |k| base_power(k));

        let distance_factor = 1.0 - (dist / radius).min(1.0);
        let threat_score = power * health * distance_factor;

        threats.push(ThreatEntry {
            entity: other,
            threat_score,
            distance: dist,
            health_ratio: health,
            power,
            group_size: 1,
        });
    }

    threats.sort_by(|a, b| {
        b.threat_score
            .partial_cmp(&a.threat_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    threats
}
