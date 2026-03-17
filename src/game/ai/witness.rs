use crate::game::ai::emotions;
use crate::game::ai::memory::*;
use crate::core::ecs::{Ecs, Entity};
use crate::core::persistent_id::PersistentEntityId;
use crate::world::components::*;

const WITNESS_RADIUS: f32 = 120.0;

/// Handle a kill event - record in memory and notify witnesses.
/// 
/// Uses helper methods instead of direct storage access.
pub fn on_kill(ecs: &mut Ecs, killer: Entity, victim: Entity, tick: u64) {
    let victim_kind = ecs.get_kind(victim).cloned();
    let killer_kind = ecs.get_kind(killer).cloned();
    let loc = ecs.get_transform(victim).map(|t| (t.cell_x, t.cell_y)).unwrap_or((0, 0));
    let victim_pid = ecs.identity.persistent_id_of(victim);
    let killer_pid = ecs.identity.persistent_id_of(killer);

    if let Some(mem) = ecs.get_memory_mut(killer) {
        mem.record_event(EventMemory {
            tick, kind: EventKind::KilledTarget, location: loc, other: victim_pid, emotional_impact: 0.3,
        });
        mem.mark_cell(loc.0, loc.1, CellTag::Food, 0.4);
        if let Some(vk) = &victim_kind {
            mem.record_lesson(Lesson {
                action: LessonAction::SoloHunt, context: context_for_kind(vk), attempts: 1, successes: 1,
            });
        }
    }
    if let Some(emo) = ecs.get_emotions_mut(killer) {
        emo.joy = (emo.joy + 0.3).min(1.0);
        emo.anger = (emo.anger - 0.2).max(0.0);
    }

    let witnesses = entities_near(ecs, victim, WITNESS_RADIUS);
    for &w in &witnesses {
        if w == killer { continue; }

        let w_kind = ecs.get_kind(w).cloned();
        let is_ally_of_victim = same_faction(&w_kind, &victim_kind);
        let is_ally_of_killer = same_faction(&w_kind, &killer_kind);

        if is_ally_of_victim {
            if let Some(mem) = ecs.get_memory_mut(w) {
                mem.record_event(EventMemory {
                    tick, kind: EventKind::AllyDied, location: loc, other: victim_pid, emotional_impact: 0.5,
                });
                if let Some(killer_pid) = killer_pid {
                mem.adjust_opinion(killer_pid, |op| {
                    op.hostility = (op.hostility + 0.4).min(1.0);
                    op.trust = (op.trust - 0.3).max(-1.0);
                });
                }
                mem.mark_cell(loc.0, loc.1, CellTag::Danger, 0.5);
            }
            let monster_traits = ecs.get_monster_traits(w).cloned();
            let npc_traits = ecs.get_npc_traits(w).cloned();
            if let Some(emo) = ecs.get_emotions_mut(w) {
                if let Some(traits) = monster_traits.as_ref() {
                    emotions::apply_monster_personality(emo, traits, 0.3, 0.1, 0.4, 0.0);
                } else if let Some(traits) = npc_traits.as_ref() {
                    emotions::apply_npc_personality(emo, traits, 0.2, 0.1, 0.5, 0.0);
                }
            }
        } else if is_ally_of_killer {
            if let Some(killer_pid) = killer_pid {
                if let Some(mem) = ecs.get_memory_mut(w) {
                    mem.adjust_opinion(killer_pid, |op| {
                        op.trust = (op.trust + 0.1).min(1.0);
                        op.familiarity = (op.familiarity + 0.05).min(1.0);
                    });
                }
            }
            if let Some(emo) = ecs.get_emotions_mut(w) {
                emo.joy = (emo.joy + 0.1).min(1.0);
            }
        }
    }
}

pub fn on_group_kill(ecs: &mut Ecs, group: &[Entity], victim: Entity, tick: u64) {
    let victim_kind = ecs.get_kind(victim).cloned();
    let loc = ecs.get_transform(victim).map(|t| (t.cell_x, t.cell_y)).unwrap_or((0, 0));

    let victim_pid = ecs.identity.persistent_id_of(victim);
    let ally_pids: Vec<(Entity, Option<PersistentEntityId>)> = group
        .iter()
        .copied()
        .map(|g| (g, ecs.identity.persistent_id_of(g)))
        .collect();
    for &a in group {
        if let Some(mem) = ecs.get_memory_mut(a) {
            mem.record_event(EventMemory {
                tick, kind: EventKind::GroupHuntWin, location: loc, other: victim_pid, emotional_impact: 0.4,
            });
            if let Some(vk) = &victim_kind {
                mem.record_lesson(Lesson {
                    action: LessonAction::GroupHunt, context: context_for_kind(vk), attempts: 1, successes: 1,
                });
            }
            for (ally, ally_pid) in &ally_pids {
                if *ally != a {
                    if let Some(ally_pid) = ally_pid {
                        mem.adjust_opinion(*ally_pid, |op| {
                            op.trust = (op.trust + 0.15).min(1.0);
                            op.familiarity = (op.familiarity + 0.1).min(1.0);
                            op.shared_kills += 1;
                        });
                    }
                }
            }
        }
        if let Some(emo) = ecs.get_emotions_mut(a) {
            emo.joy = (emo.joy + 0.4).min(1.0);
        }
    }

    let witnesses = entities_near(ecs, victim, WITNESS_RADIUS);
    for &w in &witnesses {
        if group.contains(&w) { continue; }
        let w_kind = ecs.get_kind(w).cloned();
        if same_faction(&w_kind, &victim_kind) {
            let attacker_pids: Vec<_> = group
                .iter()
                .copied()
                .map(|attacker| ecs.identity.persistent_id_of(attacker))
                .collect();
            if let Some(mem) = ecs.get_memory_mut(w) {
                mem.record_event(EventMemory {
                    tick, kind: EventKind::AllyDied, location: loc, other: victim_pid, emotional_impact: 0.5,
                });
                for attacker_pid in attacker_pids {
                    if let Some(attacker_pid) = attacker_pid {
                        mem.adjust_opinion(attacker_pid, |op| { op.hostility = (op.hostility + 0.3).min(1.0); });
                    }
                }
                mem.mark_cell(loc.0, loc.1, CellTag::Danger, 0.6);
            }
            if let Some(emo) = ecs.get_emotions_mut(w) {
                emo.grief = (emo.grief + 0.3).min(1.0);
                emo.fear = (emo.fear + 0.2).min(1.0);
            }
        }
    }
}

pub fn on_attacked(ecs: &mut Ecs, victim: Entity, attacker: Entity, tick: u64) {
    let loc = ecs.get_transform(victim).map(|t| (t.cell_x, t.cell_y)).unwrap_or((0, 0));
    let attacker_pid = ecs.identity.persistent_id_of(attacker);
    if let Some(mem) = ecs.get_memory_mut(victim) {
        mem.record_event(EventMemory {
            tick, kind: EventKind::WasAttacked, location: loc, other: attacker_pid, emotional_impact: 0.4,
        });
        if let Some(attacker_pid) = attacker_pid {
            mem.adjust_opinion(attacker_pid, |op| {
                op.hostility = (op.hostility + 0.3).min(1.0);
                op.trust = (op.trust - 0.2).max(-1.0);
                op.times_hurt_me += 1;
            });
        }
        mem.mark_cell(loc.0, loc.1, CellTag::Danger, 0.3);
    }
    if let Some(emo) = ecs.get_emotions_mut(victim) {
        emo.anger = (emo.anger + 0.2).min(1.0);
        emo.fear = (emo.fear + 0.15).min(1.0);
        emo.surprise = (emo.surprise + 0.3).min(1.0);
    }
}

pub fn on_hunt_failed(ecs: &mut Ecs, hunter: Entity, target_kind: Option<&EntityKind>, was_group: bool) {
    if let Some(mem) = ecs.get_memory_mut(hunter) {
        let action = if was_group { LessonAction::GroupHunt } else { LessonAction::SoloHunt };
        let ctx = target_kind.map_or(LessonContext::General, context_for_kind);
        mem.record_lesson(Lesson { action, context: ctx, attempts: 1, successes: 0 });
    }
}

pub fn communicate_danger(ecs: &mut Ecs, sender: Entity, radius: f32, danger_cell: (u32, u32)) {
    let witnesses = entities_near(ecs, sender, radius);
    let sender_kind = ecs.get_kind(sender).cloned();

    for &w in &witnesses {
        let w_kind = ecs.get_kind(w).cloned();
        if !same_faction(&w_kind, &sender_kind) { continue; }

        let trust = ecs.identity.persistent_id_of(sender)
            .and_then(|pid| ecs.get_memory(w).and_then(|m| m.entities.get(&pid)))
            .map_or(0.0, |op| op.trust);

        if trust > -0.3 {
            if let Some(mem) = ecs.get_memory_mut(w) {
                let strength = 0.3 * (0.5 + trust * 0.5);
                mem.mark_cell(danger_cell.0, danger_cell.1, CellTag::Danger, strength);
            }
        }
    }
}

pub fn communicate_rally(ecs: &mut Ecs, caller: Entity, radius: f32) {
    let loc = match ecs.get_transform(caller) {
        Some(t) => (t.cell_x, t.cell_y),
        None => return,
    };
    let witnesses = entities_near(ecs, caller, radius);
    let caller_kind = ecs.get_kind(caller).cloned();

    for &w in &witnesses {
        let w_kind = ecs.get_kind(w).cloned();
        if !same_faction(&w_kind, &caller_kind) { continue; }

        let trust = ecs.identity.persistent_id_of(caller)
            .and_then(|pid| ecs.get_memory(w).and_then(|m| m.entities.get(&pid)))
            .map_or(0.0, |op| op.trust);
        if trust > 0.0 {
            if let Some(mem) = ecs.get_memory_mut(w) {
                mem.mark_cell(loc.0, loc.1, CellTag::Ally, 0.3 + trust * 0.3);
            }
            if let Some(emo) = ecs.get_emotions_mut(w) {
                emo.longing = (emo.longing - 0.1).max(0.0);
            }
        }
    }
}

fn entities_near(ecs: &Ecs, origin: Entity, radius: f32) -> Vec<Entity> {
    let ot = match ecs.get_transform(origin) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let r2 = radius * radius;
    ecs.alive.iter().copied().filter(|&e| {
        e != origin && ecs.get_transform(e).map_or(false, |t| {
            (t.x - ot.x).powi(2) + (t.y - ot.y).powi(2) < r2
        })
    }).collect()
}

fn same_faction(a: &Option<EntityKind>, b: &Option<EntityKind>) -> bool {
    match (a, b) {
        (Some(EntityKind::Npc), Some(EntityKind::Npc)) => true,
        (Some(EntityKind::Monster(sa)), Some(EntityKind::Monster(sb))) => sa == sb,
        _ => false,
    }
}
