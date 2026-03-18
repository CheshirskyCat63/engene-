use crate::core::ecs::{Ecs, Entity};
use crate::game::ai::memory::{CellTag, EventKind, EventMemory};
use crate::game::economy::trading;

/// Attempt trade between buyer and seller.
///
/// Uses helper methods instead of direct storage access.
pub fn try_trade(ecs: &mut Ecs, buyer: Entity, seller: Entity) -> bool {
    let buyer_money = ecs.get_npc_economy(buyer).map_or(0.0, |e| e.money);
    let seller_food = ecs.get_needs(seller).map_or(0.0, |p| 1.0 - p.hunger);

    if buyer_money < 5.0 || seller_food < 0.3 {
        return false;
    }

    let buyer_trust = ecs
        .identity
        .persistent_id_of(seller)
        .and_then(|pid| ecs.get_memory(buyer).and_then(|m| m.entities.get(&pid)))
        .map_or(0.0, |op| op.trust);
    if buyer_trust < -0.5 {
        return false;
    }

    let price = 5.0 + (1.0 - buyer_trust) * 3.0;

    if !trading::attempt_trade(ecs, buyer, seller, price) {
        return false;
    }
    if let Some(pn) = ecs.get_needs_mut(buyer) {
        pn.hunger = (pn.hunger - 0.15).max(0.0);
    }
    if let Some(pn) = ecs.get_needs_mut(seller) {
        pn.hunger = (pn.hunger + 0.05).min(1.0);
    }

    let tick = ecs.tick;
    let loc = ecs
        .get_transform(buyer)
        .map(|t| (t.cell_x, t.cell_y))
        .unwrap_or((0, 0));
    for &e in &[buyer, seller] {
        let other_entity = if e == buyer { seller } else { buyer };
        if let Some(other_pid) = ecs.identity.persistent_id_of(other_entity) {
            if let Some(mem) = ecs.get_memory_mut(e) {
                mem.adjust_opinion(other_pid, |op| {
                    op.trust = (op.trust + 0.05).min(1.0);
                    op.familiarity = (op.familiarity + 0.03).min(1.0);
                });
                mem.record_event(EventMemory {
                    tick,
                    kind: EventKind::Traded,
                    location: loc,
                    other: Some(other_pid),
                    emotional_impact: 0.1,
                });
            }
        }
    }

    true
}

pub fn share_knowledge(ecs: &mut Ecs, speaker: Entity, listener: Entity) {
    let trust = ecs
        .identity
        .persistent_id_of(speaker)
        .and_then(|pid| ecs.get_memory(listener).and_then(|m| m.entities.get(&pid)))
        .map_or(0.0, |op| op.trust);
    if trust < 0.1 {
        return;
    }

    let cells_to_share: Vec<((u32, u32), f32, f32)> = {
        let mem = match ecs.get_memory(speaker) {
            Some(m) => m,
            None => return,
        };
        mem.spatial
            .iter()
            .filter(|(_, k)| k.food > 0.2 || k.danger > 0.2)
            .map(|(&pos, k)| (pos, k.food, k.danger))
            .take(5)
            .collect()
    };

    let transfer_strength = trust * 0.5;
    if let Some(mem) = ecs.get_memory_mut(listener) {
        for ((cx, cy), food, danger) in cells_to_share {
            if food > 0.2 {
                mem.mark_cell(cx, cy, CellTag::Food, food * transfer_strength);
            }
            if danger > 0.2 {
                mem.mark_cell(cx, cy, CellTag::Danger, danger * transfer_strength);
            }
        }
    }

    let tick = ecs.tick;
    let loc = ecs
        .get_transform(speaker)
        .map(|t| (t.cell_x, t.cell_y))
        .unwrap_or((0, 0));
    for &e in &[speaker, listener] {
        let other_entity = if e == speaker { listener } else { speaker };
        if let Some(other_pid) = ecs.identity.persistent_id_of(other_entity) {
            if let Some(mem) = ecs.get_memory_mut(e) {
                mem.adjust_opinion(other_pid, |op| {
                    op.trust = (op.trust + 0.03).min(1.0);
                    op.familiarity = (op.familiarity + 0.02).min(1.0);
                });
                mem.record_event(EventMemory {
                    tick,
                    kind: EventKind::Socialized,
                    location: loc,
                    other: Some(other_pid),
                    emotional_impact: 0.05,
                });
            }
        }
    }
}

pub fn teach_skill(ecs: &mut Ecs, teacher: Entity, student: Entity) {
    let trust = ecs
        .identity
        .persistent_id_of(teacher)
        .and_then(|pid| ecs.get_memory(student).and_then(|m| m.entities.get(&pid)))
        .map_or(0.0, |op| op.trust);
    if trust < 0.2 {
        return;
    }

    let lessons_to_share: Vec<crate::game::ai::memory::Lesson> = {
        let mem = match ecs.get_memory(teacher) {
            Some(m) => m,
            None => return,
        };
        mem.lessons
            .iter()
            .filter(|l| l.attempts >= 3 && l.successes as f32 / l.attempts as f32 > 0.5)
            .take(3)
            .cloned()
            .collect()
    };

    if let Some(mem) = ecs.get_memory_mut(student) {
        for lesson in lessons_to_share {
            let boost = crate::game::ai::memory::Lesson {
                action: lesson.action,
                context: lesson.context,
                attempts: (lesson.attempts as f32 * trust * 0.3) as u32,
                successes: (lesson.successes as f32 * trust * 0.3) as u32,
            };
            mem.record_lesson(boost);
        }
    }
}
