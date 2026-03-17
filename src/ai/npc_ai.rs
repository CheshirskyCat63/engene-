use rand::Rng;

use crate::ai::{body, combat, desire, emotions, needs, perception, social, witness};
use crate::ai::body::BodyState;
use crate::ai::memory::CellTag;
use crate::ai::perception::PerceptionCache;
use crate::ai::plan::Plan;
use crate::core::ecs::{Ecs, Entity};
use crate::core::events::EventBus;
use crate::world::cell::CELL_SIZE;
use crate::world::components::*;
use crate::world::resources::ResourceGrid;

/// Tick a single NPC entity.
/// 
/// Uses query API methods instead of direct storage access.
pub fn tick_npc(ecs: &mut Ecs, events: &mut EventBus, res: &mut ResourceGrid, entity: Entity, delta: f32, day_progress: f32) {
    if !ecs.is_alive(entity) { return; }

    let pcache = PerceptionCache::build(ecs, entity);

    decay_and_sense(ecs, entity, delta, day_progress, &pcache);

    let new_goal = desire::desire_npc(entity, ecs, day_progress);
    let should_replan = should_replan(ecs, entity, new_goal);

    if should_replan {
        let target = find_target_for_goal(ecs, res, entity, new_goal, &pcache);
        ecs.plans.insert(entity, Plan::new(new_goal, target, ecs.tick));
    }

    if let Some(mut plan) = ecs.plans.remove(&entity) {
        plan.tick(delta);
        // Use helper methods
        let stage = ecs.get_life_info(entity).map_or(LifeStage::Adult, |li| li.life_stage());
        let body = ecs.get_needs(entity)
            .map(|pn| BodyState::compute_with_stage(pn, stage))
            .unwrap_or(BodyState { move_speed_mult: 1.0, perception_radius_mult: 1.0, combat_power_mult: 1.0, work_efficiency_mult: 1.0 });
        let tod = body::time_of_day_mult(day_progress, false);
        execute(ecs, events, res, entity, &plan, delta, &body, tod, &pcache);
        if ecs.is_alive(entity) {
            ecs.ai_states.insert(entity, AiState::Executing(plan.goal));
            if !plan.is_expired() {
                ecs.plans.insert(entity, plan);
            }
        }
    }
}

fn should_replan(ecs: &Ecs, entity: Entity, new_goal: Goal) -> bool {
    // Use helper method
    match ecs.get_plan(entity) {
        None => true,
        Some(plan) => {
            if plan.is_expired() { return true; }
            if plan.goal != new_goal {
                let old_priority = goal_priority(plan.goal);
                let new_priority = goal_priority(new_goal);
                new_priority > old_priority + 1
            } else { false }
        }
    }
}

fn goal_priority(g: Goal) -> u8 {
    match g {
        Goal::Flee => 10,
        Goal::SeekFood | Goal::SeekWater => 8,
        Goal::Sleep | Goal::SeekShelter => 7,
        Goal::Rest => 7,
        Goal::Hunt => 6,
        Goal::Work | Goal::StealOrRob => 5,
        Goal::Mate => 4,
        Goal::Socialize => 4,
        Goal::Trade => 3,
        Goal::Explore => 2,
        Goal::DefendTerritory => 6,
        Goal::FollowPack => 4,
        Goal::Migrate => 3,
        Goal::DoQuest => 5,
        Goal::StayAtPost => 4,
        Goal::RepairEquipment => 6,
        Goal::BuySupplies => 6,
    }
}

fn find_target_for_goal(ecs: &Ecs, res: &ResourceGrid, entity: Entity, goal: Goal, pcache: &PerceptionCache) -> Option<(f32, f32)> {
    // Use helper methods
    let t = ecs.get_transform(entity)?;
    let mem = ecs.get_memory(entity);

    match goal {
        Goal::SeekFood => {
            if let Some(pos) = res.find_nearest_carcass(t.x, t.y, 200.0) { return Some(pos); }
            if let Some(cell) = mem_food_cell(mem) { return Some(cell_center(cell.0, cell.1)); }
            res.best_food_cell(t.cell_x, t.cell_y, 4).map(|(cx, cy)| cell_center(cx, cy))
        }
        Goal::SeekWater => {
            if let Some(cell) = mem_water_cell(mem) { return Some(cell_center(cell.0, cell.1)); }
            res.best_water_cell(t.cell_x, t.cell_y, 4).map(|(cx, cy)| cell_center(cx, cy))
        }
        Goal::Hunt => {
            pcache.prey.and_then(|prey| ecs.get_transform(prey).map(|pt| (pt.x, pt.y)))
        }
        Goal::Flee => {
            let safe = mem.and_then(|m| {
                m.spatial.iter()
                    .filter(|(_, k)| k.danger < 0.1 && k.shelter > 0.1)
                    .min_by(|a, b| a.1.danger.total_cmp(&b.1.danger))
                    .map(|(&(cx, cy), _)| cell_center(cx, cy))
            });
            safe.or_else(|| {
                pcache.predator.and_then(|p| ecs.get_transform(p))
                    .map(|pt| (t.x + (t.x - pt.x).signum() * 100.0, t.y + (t.y - pt.y).signum() * 100.0))
            })
        }
        Goal::Socialize => {
            pcache.nearby_npcs.first()
                .and_then(|&n| ecs.get_transform(n).map(|nt| (nt.x, nt.y)))
        }
        Goal::Explore => {
            let unknown_cell = find_unexplored_cell(mem, t.cell_x, t.cell_y);
            Some(cell_center(unknown_cell.0, unknown_cell.1))
        }
        Goal::SeekShelter => {
            mem.and_then(|m| {
                m.spatial.iter()
                    .filter(|(_, k)| k.shelter > 0.2 && k.danger < 0.2)
                    .max_by(|a, b| a.1.shelter.total_cmp(&b.1.shelter))
                    .map(|(&(cx, cy), _)| cell_center(cx, cy))
            }).or_else(|| Some(cell_center(10, 10)))
        }
        Goal::Mate => {
            perception::find_mate(ecs, entity)
                .and_then(|m| ecs.get_transform(m).map(|mt| (mt.x, mt.y)))
        }
        _ => None,
    }
}

fn decay_and_sense(ecs: &mut Ecs, entity: Entity, delta: f32, day_progress: f32, pcache: &PerceptionCache) {
    // Use helper methods
    let stress_res = ecs.get_npc_traits(entity).map_or(0.5, |t| t.stress_resistance);
    let sociality = ecs.get_npc_traits(entity).map_or(0.5, |t| t.sociality);
    let bravery = ecs.get_npc_traits(entity).map_or(0.5, |t| t.bravery);
    let is_night = body::is_night(day_progress);
    let learning_mult = ecs.get_life_info(entity).map_or(1.0, |li| li.life_stage().learning_mult());

    let night_fear_mult = if is_night { 1.5 } else { 1.0 };
    let night_sleep_mult = if is_night { 2.5 } else { 1.0 };

    if let Some(pn) = ecs.get_needs_mut(entity) {
        needs::decay_personal_needs(pn, delta);
        pn.sleep = (pn.sleep + delta * 0.003 * night_sleep_mult).min(1.0);
        pn.ambitions = (pn.ambitions + delta * 0.001).min(1.0);
        pn.discomfort = (pn.discomfort + delta * 0.002 * (1.0 - stress_res)).min(1.0);
        if pn.hunger > 0.7 || pn.thirst > 0.7 { pn.discomfort = (pn.discomfort + delta * 0.005).min(1.0); }
    }
    if let Some(sn) = ecs.get_social_needs_mut(entity) {
        sn.loneliness = (sn.loneliness + delta * 0.003 * sociality).min(1.0);
        sn.entertainment = (sn.entertainment + delta * 0.002).min(1.0);
        sn.family = (sn.family + delta * 0.002).min(1.0);
        sn.friendship = (sn.friendship - delta * 0.001).max(0.0);
        sn.reputation = (sn.reputation - delta * 0.0005).max(0.0);
    }

    let is_alone = pcache.nearby_npcs.is_empty();
    if let Some(emo) = ecs.get_emotions_mut(entity) {
        emo.decay(delta);
        if is_alone { emo.longing = (emo.longing + delta * 0.003 * sociality).min(1.0); }
        if is_night && bravery < 0.5 { emo.fear = (emo.fear + delta * 0.002 * night_fear_mult).min(1.0); }
    }
    if let Some(mem) = ecs.get_memory_mut(entity) { mem.decay_spatial(delta * 0.0005); }

    if let Some(predator) = pcache.predator {
        if let (Some(emo), Some(traits)) = (ecs.get_emotions_mut(entity), ecs.get_npc_traits(entity)) {
            emotions::apply_npc_personality(emo, traits, 0.0, delta * 0.15, 0.0, 0.0);
        }
        if let Some(pn) = ecs.get_needs_mut(entity) {
            pn.fear = (pn.fear + delta * 0.1 * (1.0 - bravery)).min(1.0);
        }
        if let Some(loc) = ecs.get_transform(predator).map(|t| (t.cell_x, t.cell_y)) {
            if let Some(mem) = ecs.get_memory_mut(entity) { mem.mark_cell(loc.0, loc.1, CellTag::Danger, 0.2); }
        }
    }

    if !pcache.nearby_npcs.is_empty() {
        if let Some(sn) = ecs.get_social_needs_mut(entity) {
            sn.loneliness = (sn.loneliness - delta * 0.01 * pcache.nearby_npcs.len() as f32).max(0.0);
        }
        for &npc in &pcache.nearby_npcs {
            if let Some(pid) = ecs.identity.persistent_id_of(npc) {
                if let Some(mem) = ecs.get_memory_mut(entity) {
                    let tick = ecs.tick;
                    mem.adjust_opinion(pid, |op| {
                        op.familiarity = (op.familiarity + delta * 0.002 * learning_mult).min(1.0);
                        op.last_seen_tick = tick;
                    });
                }
            }
        }
        if let Some(emo) = ecs.get_emotions_mut(entity) { emo.longing = (emo.longing - delta * 0.01).max(0.0); }
    }

    if let Some(loc) = ecs.get_transform(entity).map(|t| (t.cell_x, t.cell_y)) {
        if let Some(mem) = ecs.get_memory_mut(entity) { mem.mark_cell(loc.0, loc.1, CellTag::Shelter, 0.02); }
    }
}

fn execute(ecs: &mut Ecs, events: &mut EventBus, res: &mut ResourceGrid, entity: Entity, plan: &Plan, delta: f32, body: &BodyState, tod: f32, pcache: &PerceptionCache) {
    let mut rng = rand::thread_rng();
    let speed = delta * body.move_speed_mult * tod;

    match plan.goal {
        Goal::SeekFood => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 3.0); }
            else { wander(ecs, entity, speed * 2.0); }
            // Use helper methods
            if let Some(t) = ecs.get_transform(entity) {
                let from_carcass = res.consume_carcass_near(t.x, t.y, delta * 0.05);
                if from_carcass > 0.0 {
                    if let Some(pn) = ecs.get_needs_mut(entity) { needs::satisfy_hunger(pn, from_carcass); }
                    if let Some(mem) = ecs.get_memory_mut(entity) { mem.mark_cell(t.cell_x, t.cell_y, CellTag::Food, 0.3); }
                } else {
                    let taken = res.consume_food(t.cell_x, t.cell_y, delta * 0.03);
                    if taken > 0.01 {
                        if let Some(pn) = ecs.get_needs_mut(entity) { needs::satisfy_hunger(pn, taken); }
                        if let Some(mem) = ecs.get_memory_mut(entity) { mem.mark_cell(t.cell_x, t.cell_y, CellTag::Food, 0.2); }
                    }
                }
            }
            if let Some(econ) = ecs.get_npc_economy_mut(entity) { econ.money = (econ.money - delta * 0.1).max(0.0); }
        }
        Goal::SeekWater => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 3.0); }
            else { wander(ecs, entity, speed * 2.0); }
            if let Some(t) = ecs.get_transform(entity) {
                let taken = res.consume_water(t.cell_x, t.cell_y, delta * 0.04);
                if taken > 0.01 {
                    if let Some(pn) = ecs.get_needs_mut(entity) { needs::satisfy_thirst(pn, taken); }
                }
            }
        }
        Goal::Rest => {
            if let Some(pn) = ecs.get_needs_mut(entity) {
                needs::satisfy_sleep(pn, delta * 0.03);
                pn.health = (pn.health + delta * 0.012).min(1.0);
                pn.discomfort = (pn.discomfort - delta * 0.015).max(0.0);
            }
            if let Some(emo) = ecs.get_emotions_mut(entity) { emo.fear = (emo.fear - delta * 0.005).max(0.0); }
        }
        Goal::Sleep => {
            if let Some(pn) = ecs.get_needs_mut(entity) {
                needs::satisfy_sleep(pn, delta * 0.06);
                pn.health = (pn.health + delta * 0.02).min(1.0);
                pn.energy = (pn.energy + delta * 0.015).min(1.0);
                pn.discomfort = (pn.discomfort - delta * 0.02).max(0.0);
            }
            if let Some(emo) = ecs.get_emotions_mut(entity) {
                emo.fear = (emo.fear - delta * 0.008).max(0.0);
                emo.anger = (emo.anger - delta * 0.005).max(0.0);
            }
        }
        Goal::SeekShelter => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 4.0); }
            else { wander(ecs, entity, speed * 3.0); }
            if let Some(pn) = ecs.get_needs_mut(entity) {
                needs::satisfy_sleep(pn, delta * 0.01);
            }
            if let Some(t) = ecs.get_transform(entity).map(|t| (t.cell_x, t.cell_y)) {
                if let Some(mem) = ecs.get_memory_mut(entity) { mem.mark_cell(t.0, t.1, CellTag::Shelter, 0.1); }
            }
        }
        Goal::Work => {
            let (income, work_ethic) = {
                let econ = match ecs.get_npc_economy(entity) { Some(e) => e, None => return };
                let we = ecs.get_npc_traits(entity).map_or(0.5, |t| t.work_ethic);
                let rate = match econ.job {
                    Job::Guard => 0.8, Job::Trader => 1.0, Job::ArtifactHunter => 1.5,
                    Job::Bandit => 1.2, Job::Hunter => 1.2, Job::Scavenger => 0.9,
                    Job::Courier => 0.7, Job::Resident => 0.3, Job::Unemployed => 0.3,
                };
                (delta * rate * we * body.work_efficiency_mult * rng.gen_range(0.8..1.2), we)
            };
            if let Some(econ) = ecs.get_npc_economy_mut(entity) { econ.money += income; }
            if let Some(pn) = ecs.get_needs_mut(entity) {
                pn.energy = (pn.energy - delta * 0.015 * (2.0 - work_ethic)).max(0.0);
            }
            if let Some(sn) = ecs.get_social_needs_mut(entity) { sn.money = (sn.money - delta * 0.005).max(0.0); sn.reputation = (sn.reputation + delta * 0.002).min(1.0); }
        }
        Goal::Hunt => {
            let autonomy = ecs.get_npc_traits(entity).map_or(0.5, |t| t.autonomy);
            let aggr = ecs.get_npc_traits(entity).map_or(0.5, |t| t.aggressiveness);
            if let Some(prey) = pcache.prey.filter(|&p| ecs.is_alive(p)) {
                move_toward(ecs, entity, prey, speed * (3.0 + aggr * 2.0) * body.combat_power_mult);
                if perception::distance2(ecs, entity, prey) < 30.0 * 30.0 {
                    combat::resolve_combat(ecs, events, entity, prey);
                }
            } else if autonomy < 0.6 {
                let allies = &pcache.nearby_allies;
                if !allies.is_empty() {
                    if let Some(target) = perception::find_group_target(ecs, allies, entity) {
                        witness::communicate_rally(ecs, entity, 150.0);
                        move_toward(ecs, entity, target, speed * 3.0);
                        if perception::distance2(ecs, entity, target) < 35.0 * 35.0 {
                            let mut group: Vec<Entity> = allies.iter().copied()
                                .filter(|&a| perception::distance2(ecs, a, target) < 50.0 * 50.0).collect();
                            group.push(entity);
                            combat::resolve_group_combat(ecs, events, &group, target);
                        }
                    } else { wander(ecs, entity, speed * 3.0); }
                } else {
                    let trusted = ecs.get_memory(entity)
                        .and_then(|m| m.best_ally().and_then(|pid| ecs.identity.resolve(pid)))
                        .filter(|&a| ecs.is_alive(a));
                    if let Some(ally) = trusted { move_toward(ecs, entity, ally, speed * 2.0); }
                    else { wander(ecs, entity, speed * 3.0); }
                }
            } else { wander(ecs, entity, speed * 3.0); }
            if let Some(pn) = ecs.get_needs_mut(entity) { pn.energy = (pn.energy - delta * 0.01).max(0.0); }
        }
        Goal::Trade => {
            if let Some(&partner) = pcache.nearby_npcs.first() {
                move_toward(ecs, entity, partner, speed * 2.0);
                if perception::distance2(ecs, entity, partner) < 25.0 * 25.0 {
                    if social::try_trade(ecs, entity, partner) {
                        if let Some(emo) = ecs.get_emotions_mut(entity) { emo.joy = (emo.joy + 0.1).min(1.0); }
                    }
                }
            } else { wander(ecs, entity, speed * 2.0); }
        }
        Goal::Explore => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 4.0); }
            else { wander(ecs, entity, speed * 4.0); }
            if let Some(pn) = ecs.get_needs_mut(entity) { pn.curiosity = (pn.curiosity - delta * 0.012).max(0.0); }
            if let Some(t) = ecs.get_transform(entity).map(|t| (t.cell_x, t.cell_y)) {
                let food = res.food_at(t.0, t.1);
                if let Some(mem) = ecs.get_memory_mut(entity) {
                    if food > 0.2 { mem.mark_cell(t.0, t.1, CellTag::Food, food * 0.5); }
                    mem.mark_cell(t.0, t.1, CellTag::Shelter, 0.05);
                }
            }
        }
        Goal::Socialize => {
            if let Some(&friend) = pcache.nearby_npcs.first() {
                move_toward(ecs, entity, friend, speed * 2.0);
                if perception::distance2(ecs, entity, friend) < 20.0 * 20.0 {
                    social::share_knowledge(ecs, entity, friend);
                    social::teach_skill(ecs, entity, friend);
                }
            }
            let sociality = ecs.get_npc_traits(entity).map_or(0.5, |t| t.sociality);
            if let Some(sn) = ecs.get_social_needs_mut(entity) {
                sn.loneliness = (sn.loneliness - delta * 0.06 * sociality).max(0.0);
                sn.entertainment = (sn.entertainment - delta * 0.04).max(0.0);
                sn.friendship = (sn.friendship + delta * 0.01).min(1.0);
                sn.faction_loyalty = (sn.faction_loyalty + delta * 0.005).min(1.0);
            }
            if let Some(emo) = ecs.get_emotions_mut(entity) { emo.joy = (emo.joy + delta * 0.005).min(1.0); emo.longing = (emo.longing - delta * 0.01).max(0.0); }
            for &n in &pcache.nearby_npcs {
                if let Some(pid) = ecs.identity.persistent_id_of(n) {
                    if let Some(mem) = ecs.get_memory_mut(entity) {
                        mem.adjust_opinion(pid, |op| { op.trust = (op.trust + delta * 0.003).min(1.0); op.familiarity = (op.familiarity + delta * 0.005).min(1.0); });
                    }
                }
            }
        }
        Goal::Flee => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 8.0); }
            else {
                flee_from(ecs, entity, pcache.predator, speed * 8.0);
            }
            if let Some(pn) = ecs.get_needs_mut(entity) { pn.fear = (pn.fear - delta * 0.02).max(0.0); pn.energy = (pn.energy - delta * 0.02).max(0.0); }
            if let Some(emo) = ecs.get_emotions_mut(entity) { emo.fear = (emo.fear - delta * 0.005).max(0.0); }
        }
        Goal::StealOrRob => {
            let honesty = ecs.get_npc_traits(entity).map_or(0.5, |t| t.honesty);
            let risk_tol = ecs.get_npc_traits(entity).map_or(0.5, |t| t.risk_tolerance);
            let stolen = delta * rng.gen_range(1.0..3.0) * (1.0 + risk_tol * 0.5);
            if let Some(econ) = ecs.get_npc_economy_mut(entity) { econ.money += stolen; }
            if let Some(sn) = ecs.get_social_needs_mut(entity) { sn.reputation = (sn.reputation - delta * 0.02).max(0.0); sn.fear_of_punishment = (sn.fear_of_punishment + delta * 0.03).min(1.0); }
            if let Some(emo) = ecs.get_emotions_mut(entity) { emo.disgust = (emo.disgust + delta * 0.01 * honesty).min(1.0); }
            for &w in &pcache.nearby_npcs {
                let tick = ecs.tick;
                let loc = ecs.get_transform(entity).map(|t| (t.cell_x, t.cell_y)).unwrap_or((0,0));
                if let Some(entity_pid) = ecs.identity.persistent_id_of(entity) {
                    if let Some(mem) = ecs.get_memory_mut(w) {
                        mem.record_event(crate::ai::memory::EventMemory { tick, kind: crate::ai::memory::EventKind::SawTheft, location: loc, other: Some(entity_pid), emotional_impact: 0.2 });
                        mem.adjust_opinion(entity_pid, |op| { op.trust = (op.trust - 0.15).max(-1.0); op.hostility = (op.hostility + 0.1).min(1.0); });
                    }
                }
                if let Some(emo) = ecs.get_emotions_mut(w) { emo.disgust = (emo.disgust + 0.15).min(1.0); }
            }
        }
        Goal::Mate => {
            if let Some(mate) = perception::find_mate(ecs, entity) {
                move_toward(ecs, entity, mate, speed * 2.0);
                if perception::distance2(ecs, entity, mate) < 20.0 * 20.0 {
                    crate::ai::reproduction::try_reproduce(ecs, entity, mate);
                    if let Some(emo) = ecs.get_emotions_mut(entity) { emo.joy = (emo.joy + 0.2).min(1.0); }
                }
            } else { wander(ecs, entity, speed * 2.0); }
        }
        _ => {}
    }
}

fn mem_food_cell(mem: Option<&crate::ai::memory::Memory>) -> Option<(u32, u32)> {
    mem?.spatial.iter()
        .filter(|(_, k)| k.food > 0.15)
        .max_by(|a, b| a.1.food.total_cmp(&b.1.food))
        .map(|(&pos, _)| pos)
}

fn mem_water_cell(mem: Option<&crate::ai::memory::Memory>) -> Option<(u32, u32)> {
    mem?.spatial.iter()
        .filter(|(_, k)| k.shelter > 0.05)
        .max_by(|a, b| a.1.shelter.total_cmp(&b.1.shelter))
        .map(|(&pos, _)| pos)
}

fn find_unexplored_cell(mem: Option<&crate::ai::memory::Memory>, _cx: u32, _cy: u32) -> (u32, u32) {
    let mut rng = rand::thread_rng();
    let grid = crate::world::cell::GRID_SIZE;
    for _ in 0..10 {
        let rx = rng.gen_range(0..grid);
        let ry = rng.gen_range(0..grid);
        if mem.map_or(true, |m| !m.spatial.contains_key(&(rx, ry))) {
            return (rx, ry);
        }
    }
    (rng.gen_range(0..grid), rng.gen_range(0..grid))
}

fn cell_center(cx: u32, cy: u32) -> (f32, f32) {
    (cx as f32 * CELL_SIZE + CELL_SIZE * 0.5, cy as f32 * CELL_SIZE + CELL_SIZE * 0.5)
}

fn move_to_pos(ecs: &mut Ecs, entity: Entity, target: (f32, f32), speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    // Use helper method
    if let Some(t) = ecs.get_transform_mut(entity) {
        let dx = target.0 - t.x; let dy = target.1 - t.y;
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        t.x = (t.x + dx / len * speed).clamp(0.0, ws);
        t.y = (t.y + dy / len * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}

fn move_toward(ecs: &mut Ecs, mover: Entity, target: Entity, speed: f32) {
    // Use helper method
    let (tx, ty) = match ecs.get_transform(target) { Some(t) => (t.x, t.y), None => return };
    move_to_pos(ecs, mover, (tx, ty), speed);
}

fn flee_from(ecs: &mut Ecs, entity: Entity, threat: Option<Entity>, speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    let mut rng = rand::thread_rng();
    // Use helper methods
    let (dx, dy) = if let Some(t) = threat {
        let (tx, ty) = match ecs.get_transform(t) { Some(tr) => (tr.x, tr.y), None => (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) };
        let (mx, my) = match ecs.get_transform(entity) { Some(tr) => (tr.x, tr.y), None => return };
        (mx - tx, my - ty)
    } else { (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) };
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    if let Some(t) = ecs.get_transform_mut(entity) {
        t.x = (t.x + dx / len * speed).clamp(0.0, ws);
        t.y = (t.y + dy / len * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}

fn wander(ecs: &mut Ecs, entity: Entity, speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    let mut rng = rand::thread_rng();
    // Use helper method
    if let Some(t) = ecs.get_transform_mut(entity) {
        t.x = (t.x + rng.gen_range(-1.0..1.0) * speed).clamp(0.0, ws);
        t.y = (t.y + rng.gen_range(-1.0..1.0) * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}
