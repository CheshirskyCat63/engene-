use glam::Vec2;
use crate::core::ecs::{Ecs, Entity};
use crate::game::ai::combat_tactics::tactics::Tactic;

#[derive(Clone, Debug)]
pub struct GroupRole {
    pub entity: Entity,
    pub tactic: Tactic,
    pub target_position: Vec2,
    pub target_entity: Option<Entity>,
}

pub fn assign_group_roles(
    ecs: &Ecs,
    group: &[Entity],
    target: Entity,
    group_tactic: Tactic,
) -> Vec<GroupRole> {
    let target_pos = ecs.get_transform(target).map(|t| Vec2::new(t.x, t.y));
    let target_pos = match target_pos {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut roles = Vec::new();

    match group_tactic {
        Tactic::Surround => {
            let count = group.len();
            for (i, &entity) in group.iter().enumerate() {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                let offset = Vec2::new(angle.cos(), angle.sin()) * 30.0;
                roles.push(GroupRole {
                    entity,
                    tactic: Tactic::Surround,
                    target_position: target_pos + offset,
                    target_entity: Some(target),
                });
            }
        }
        Tactic::Flank => {
            let half = group.len() / 2;
            for (i, &entity) in group.iter().enumerate() {
                let side = if i < half { 1.0 } else { -1.0 };
                let forward = Vec2::new(0.0, 1.0);
                let lateral = Vec2::new(side * 40.0, -20.0);
                roles.push(GroupRole {
                    entity,
                    tactic: Tactic::Flank,
                    target_position: target_pos + forward * 10.0 + lateral,
                    target_entity: Some(target),
                });
            }
        }
        _ => {
            for &entity in group {
                roles.push(GroupRole {
                    entity,
                    tactic: group_tactic,
                    target_position: target_pos,
                    target_entity: Some(target),
                });
            }
        }
    }

    roles
}

pub fn find_group_members(ecs: &Ecs, leader: Entity, radius: f32) -> Vec<Entity> {
    let leader_pos = match ecs.get_transform(leader) {
        Some(t) => (t.x, t.y),
        None => return Vec::new(),
    };
    let leader_kind = ecs.get_kind(leader);

    let mut members = vec![leader];
    for &e in &ecs.alive {
        if e == leader { continue; }
        if ecs.get_kind(e) != leader_kind { continue; }
        if let Some(t) = ecs.get_transform(e) {
            let dx = t.x - leader_pos.0;
            let dy = t.y - leader_pos.1;
            if dx * dx + dy * dy < radius * radius {
                members.push(e);
            }
        }
    }

    members
}
