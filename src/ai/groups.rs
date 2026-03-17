use crate::core::ecs::{Ecs, Entity};
use crate::core::persistent_id::PersistentEntityId;
use crate::world::components::EntityKind;

#[derive(Clone, Debug)]
pub struct Group {
    pub leader: PersistentEntityId,
    pub members: Vec<PersistentEntityId>,
    pub formed_tick: u64,
}

/// Find or form a group for the given entity.
/// 
/// Uses helper methods instead of direct storage access.
pub fn find_or_form_group(ecs: &Ecs, entity: Entity) -> Option<Group> {
    let kind = ecs.get_kind(entity)?;
    let et = ecs.get_transform(entity)?;
    let mem = ecs.get_memory(entity);
    let r2 = 100.0 * 100.0;

    let mut candidates: Vec<(Entity, f32)> = Vec::new();
    for e in ecs.spatial.candidates_in_radius(et.x, et.y, 100.0) {
        if e == entity { continue; }
        if !same_kind(kind, ecs.get_kind(e)) { continue; }
        let Some(t) = ecs.get_transform(e) else { continue };
        let d2 = (t.x - et.x).powi(2) + (t.y - et.y).powi(2);
        if d2 > r2 { continue; }

        let trust = ecs.identity.persistent_id_of(e)
            .and_then(|pid| mem.and_then(|m| m.entities.get(&pid)))
            .map_or(0.0, |op| op.trust);
        if trust > -0.2 {
            candidates.push((e, trust));
        }
    }

    if candidates.is_empty() { return None; }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let member_entities: Vec<Entity> = candidates.into_iter().take(4).map(|(e, _)| e).collect();
    let members: Vec<PersistentEntityId> = member_entities.iter()
        .filter_map(|&e| ecs.identity.persistent_id_of(e))
        .collect();
    if members.is_empty() { return None; }

    let leader_entity = pick_leader(ecs, entity, &member_entities);
    let leader = ecs.identity.persistent_id_of(leader_entity)?;

    Some(Group { leader, members, formed_tick: ecs.tick })
}

fn pick_leader(ecs: &Ecs, self_entity: Entity, members: &[Entity]) -> Entity {
    let mut best = self_entity;
    let mut best_rep = reputation_of(ecs, self_entity);

    for &m in members {
        let rep = reputation_of(ecs, m);
        if rep > best_rep {
            best = m;
            best_rep = rep;
        }
    }
    best
}

fn reputation_of(ecs: &Ecs, entity: Entity) -> f32 {
    ecs.get_social_needs(entity).map_or(0.0, |s| s.reputation)
        + ecs.get_ecosystem_needs(entity).map_or(0.0, |e| e.food_chain_position * 0.5)
        + ecs.get_needs(entity).map_or(0.0, |p| p.health * 0.3)
}

fn same_kind(a: &EntityKind, b: Option<&EntityKind>) -> bool {
    match (a, b) {
        (EntityKind::Npc, Some(EntityKind::Npc)) => true,
        (EntityKind::Monster(sa), Some(EntityKind::Monster(sb))) => sa == sb,
        _ => false,
    }
}
