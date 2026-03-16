use std::collections::HashSet;
use crate::core::ecs::{Ecs, Entity};
use crate::world::components::*;

pub const HUNT_RADIUS: f32 = 120.0;
pub const FEAR_RADIUS: f32 = 150.0;
pub const ALLY_RADIUS: f32 = 80.0;
pub const SOCIAL_RADIUS: f32 = 100.0;

pub struct PerceptionCache {
    pub predator: Option<Entity>,
    pub nearby_allies: Vec<Entity>,
    pub nearby_npcs: Vec<Entity>,
    pub prey: Option<Entity>,
    pub entities_nearby_count: usize,
}

impl PerceptionCache {
    pub fn build(ecs: &Ecs, entity: Entity) -> Self {
        Self {
            predator: find_predator(ecs, entity),
            nearby_allies: find_allies(ecs, entity),
            nearby_npcs: npcs_nearby(ecs, entity, SOCIAL_RADIUS),
            prey: find_prey(ecs, entity),
            entities_nearby_count: count_entities_nearby(ecs, entity, 80.0),
        }
    }
}

pub fn find_prey(ecs: &Ecs, hunter: Entity) -> Option<Entity> {
    let hunter_kind = ecs.kinds.get(&hunter)?;
    let ht = ecs.transforms.get(&hunter)?;
    let r2 = HUNT_RADIUS * HUNT_RADIUS;

    let mut best: Option<(Entity, f32)> = None;
    for e in ecs.spatial.candidates_in_radius(ht.x, ht.y, HUNT_RADIUS) {
        if e == hunter { continue; }
        let Some(ek) = ecs.kinds.get(&e) else { continue };
        if !is_prey_for(hunter_kind, ek) { continue; }
        let Some(et) = ecs.transforms.get(&e) else { continue };
        let d2 = (et.x - ht.x).powi(2) + (et.y - ht.y).powi(2);
        if d2 < r2 && best.map_or(true, |(_, bd)| d2 < bd) {
            best = Some((e, d2));
        }
    }
    best.map(|(e, _)| e)
}

pub fn find_prey_selective(ecs: &Ecs, hunter: Entity, prefer_weakest: f32) -> Option<Entity> {
    let hunter_kind = ecs.kinds.get(&hunter)?;
    let ht = ecs.transforms.get(&hunter)?;
    let r2 = HUNT_RADIUS * HUNT_RADIUS;

    let mut candidates: Vec<(Entity, f32, f32)> = Vec::new();
    for e in ecs.spatial.candidates_in_radius(ht.x, ht.y, HUNT_RADIUS) {
        if e == hunter { continue; }
        let Some(ek) = ecs.kinds.get(&e) else { continue };
        if !is_prey_for(hunter_kind, ek) { continue; }
        let Some(et) = ecs.transforms.get(&e) else { continue };
        let d2 = (et.x - ht.x).powi(2) + (et.y - ht.y).powi(2);
        if d2 < r2 {
            let power = base_power(ek);
            candidates.push((e, d2, power));
        }
    }
    if candidates.is_empty() { return None; }

    candidates.sort_by(|a, b| {
        let score_a = a.1 * (1.0 - prefer_weakest) + a.2 * prefer_weakest * 1000.0;
        let score_b = b.1 * (1.0 - prefer_weakest) + b.2 * prefer_weakest * 1000.0;
        score_a.partial_cmp(&score_b).unwrap()
    });
    Some(candidates[0].0)
}

pub fn find_predator(ecs: &Ecs, prey: Entity) -> Option<Entity> {
    let prey_kind = ecs.kinds.get(&prey)?;
    let pt = ecs.transforms.get(&prey)?;
    let r2 = FEAR_RADIUS * FEAR_RADIUS;

    let mut best: Option<(Entity, f32)> = None;
    for e in ecs.spatial.candidates_in_radius(pt.x, pt.y, FEAR_RADIUS) {
        if e == prey { continue; }
        let Some(ek) = ecs.kinds.get(&e) else { continue };
        if !is_prey_for(ek, prey_kind) { continue; }
        let Some(et) = ecs.transforms.get(&e) else { continue };
        let d2 = (et.x - pt.x).powi(2) + (et.y - pt.y).powi(2);
        if d2 < r2 && best.map_or(true, |(_, bd)| d2 < bd) {
            best = Some((e, d2));
        }
    }
    best.map(|(e, _)| e)
}

pub fn find_allies(ecs: &Ecs, entity: Entity) -> Vec<Entity> {
    let kind = match ecs.kinds.get(&entity) {
        Some(k) => k.clone(),
        None => return Vec::new(),
    };
    let et = match ecs.transforms.get(&entity) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let r2 = ALLY_RADIUS * ALLY_RADIUS;

    ecs.spatial.candidates_in_radius(et.x, et.y, ALLY_RADIUS)
        .into_iter()
        .filter(|&e| {
            if e == entity { return false; }
            let same_kind = match (&kind, ecs.kinds.get(&e)) {
                (EntityKind::Npc, Some(EntityKind::Npc)) => true,
                (EntityKind::Monster(a), Some(EntityKind::Monster(b))) => a == b,
                _ => false,
            };
            if !same_kind { return false; }
            ecs.transforms.get(&e).map_or(false, |t| {
                (t.x - et.x).powi(2) + (t.y - et.y).powi(2) < r2
            })
        })
        .collect()
}

pub fn find_group_target(ecs: &Ecs, group: &[Entity], leader: Entity) -> Option<Entity> {
    let group_set: HashSet<Entity> = group.iter().copied().collect();
    let group_power: f32 = group.iter()
        .filter_map(|&e| ecs.kinds.get(&e))
        .map(base_power)
        .sum::<f32>()
        + ecs.kinds.get(&leader).map_or(0.0, base_power);

    let lt = ecs.transforms.get(&leader)?;
    let r2 = HUNT_RADIUS * HUNT_RADIUS;

    let mut best: Option<(Entity, f32)> = None;
    for e in ecs.spatial.candidates_in_radius(lt.x, lt.y, HUNT_RADIUS) {
        if e == leader || group_set.contains(&e) { continue; }
        let Some(ek) = ecs.kinds.get(&e) else { continue };
        let target_power = base_power(ek);
        if target_power >= group_power { continue; }
        let Some(et) = ecs.transforms.get(&e) else { continue };
        let d2 = (et.x - lt.x).powi(2) + (et.y - lt.y).powi(2);
        if d2 < r2 && best.map_or(true, |(_, bd)| d2 < bd) {
            best = Some((e, d2));
        }
    }
    best.map(|(e, _)| e)
}

pub fn count_entities_nearby(ecs: &Ecs, entity: Entity, radius: f32) -> usize {
    let et = match ecs.transforms.get(&entity) {
        Some(t) => t,
        None => return 0,
    };
    let r2 = radius * radius;
    ecs.spatial.candidates_in_radius(et.x, et.y, radius)
        .iter()
        .filter(|&&e| {
            e != entity && ecs.transforms.get(&e).map_or(false, |t| {
                (t.x - et.x).powi(2) + (t.y - et.y).powi(2) < r2
            })
        })
        .count()
}

pub fn npcs_nearby(ecs: &Ecs, entity: Entity, radius: f32) -> Vec<Entity> {
    let et = match ecs.transforms.get(&entity) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let r2 = radius * radius;
    ecs.spatial.candidates_in_radius(et.x, et.y, radius)
        .into_iter()
        .filter(|&e| {
            e != entity
                && matches!(ecs.kinds.get(&e), Some(EntityKind::Npc))
                && ecs.transforms.get(&e).map_or(false, |t| {
                    (t.x - et.x).powi(2) + (t.y - et.y).powi(2) < r2
                })
        })
        .collect()
}

pub fn find_mate(ecs: &Ecs, entity: Entity) -> Option<Entity> {
    let kind = ecs.kinds.get(&entity)?;
    let et = ecs.transforms.get(&entity)?;
    let r2 = SOCIAL_RADIUS * SOCIAL_RADIUS;

    let mut best: Option<(Entity, f32)> = None;
    for e in ecs.spatial.candidates_in_radius(et.x, et.y, SOCIAL_RADIUS) {
        if e == entity { continue; }
        let same = match (kind, ecs.kinds.get(&e)) {
            (EntityKind::Npc, Some(EntityKind::Npc)) => true,
            (EntityKind::Monster(a), Some(EntityKind::Monster(b))) => a == b,
            _ => false,
        };
        if !same { continue; }
        let pn = match ecs.personal_needs.get(&e) { Some(p) => p, None => continue };
        if pn.health < 0.5 || pn.hunger > 0.5 { continue; }
        let trust = ecs.identity.persistent_id_of(e)
            .and_then(|pid| ecs.memories.get(&entity).and_then(|m| m.entities.get(&pid)))
            .map_or(0.0, |op| op.trust);
        if trust < 0.2 { continue; }
        let Some(t) = ecs.transforms.get(&e) else { continue };
        let d2 = (t.x - et.x).powi(2) + (t.y - et.y).powi(2);
        if d2 < r2 && best.map_or(true, |(_, bd)| d2 < bd) {
            best = Some((e, d2));
        }
    }
    best.map(|(e, _)| e)
}

pub fn distance2(ecs: &Ecs, a: Entity, b: Entity) -> f32 {
    let (ax, ay) = match ecs.transforms.get(&a) {
        Some(t) => (t.x, t.y),
        None => return f32::MAX,
    };
    let (bx, by) = match ecs.transforms.get(&b) {
        Some(t) => (t.x, t.y),
        None => return f32::MAX,
    };
    (ax - bx).powi(2) + (ay - by).powi(2)
}
