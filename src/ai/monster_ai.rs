use rand::Rng;

use crate::ai::{body, combat, desire, emotions, needs, perception, witness};
use crate::ai::body::BodyState;
use crate::ai::memory::CellTag;
use crate::ai::perception::PerceptionCache;
use crate::ai::plan::Plan;
use crate::core::ecs::{Ecs, Entity};
use crate::core::events::EventBus;
use crate::world::cell::CELL_SIZE;
use crate::world::components::*;
use crate::world::resources::ResourceGrid;

pub fn tick_monster(ecs: &mut Ecs, events: &mut EventBus, res: &mut ResourceGrid, entity: Entity, delta: f32, day_progress: f32) {
    if !ecs.is_alive(entity) { return; }

    let is_nocturnal = matches!(ecs.kinds.get(&entity), Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)));
    let pcache = PerceptionCache::build(ecs, entity);

    decay_and_sense(ecs, entity, delta, day_progress, is_nocturnal, &pcache);

    let new_goal = desire::desire_monster(entity, ecs, day_progress);
    let should_replan = should_replan(ecs, entity, new_goal);

    if should_replan {
        let target = find_target_for_goal(ecs, res, entity, new_goal, &pcache);
        ecs.plans.insert(entity, Plan::new(new_goal, target, ecs.tick));
    }

    if let Some(mut plan) = ecs.plans.remove(&entity) {
        plan.tick(delta);
        let stage = ecs.life_info.get(&entity).map_or(LifeStage::Adult, |li| li.life_stage());
        let body_st = ecs.personal_needs.get(&entity)
            .map(|pn| BodyState::compute_with_stage(pn, stage))
            .unwrap_or(BodyState { move_speed_mult: 1.0, perception_radius_mult: 1.0, combat_power_mult: 1.0, work_efficiency_mult: 1.0 });
        let tod = body::time_of_day_mult(day_progress, is_nocturnal);
        execute(ecs, events, res, entity, &plan, delta, &body_st, tod, &pcache);
        if ecs.is_alive(entity) {
            ecs.ai_states.insert(entity, AiState::Executing(plan.goal));
            if !plan.is_expired() { ecs.plans.insert(entity, plan); }
        }
    }
}

fn should_replan(ecs: &Ecs, entity: Entity, new_goal: Goal) -> bool {
    match ecs.plans.get(&entity) {
        None => true,
        Some(plan) => {
            if plan.is_expired() { return true; }
            if plan.goal != new_goal {
                let old_p = goal_priority(plan.goal);
                let new_p = goal_priority(new_goal);
                new_p > old_p + 1
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
        Goal::DefendTerritory => 6,
        Goal::Mate => 4,
        Goal::FollowPack => 4,
        Goal::Migrate => 3,
        Goal::Explore => 2,
        _ => 1,
    }
}

fn find_target_for_goal(ecs: &Ecs, res: &ResourceGrid, entity: Entity, goal: Goal, pcache: &PerceptionCache) -> Option<(f32, f32)> {
    let t = ecs.transforms.get(&entity)?;
    let mem = ecs.memories.get(&entity);

    match goal {
        Goal::SeekFood => {
            if let Some(pos) = res.find_nearest_carcass(t.x, t.y, 250.0) { return Some(pos); }
            if let Some(cell) = mem_food_cell(mem) { return Some(cell_center(cell.0, cell.1)); }
            res.best_food_cell(t.cell_x, t.cell_y, 5).map(|(cx, cy)| cell_center(cx, cy))
        }
        Goal::Hunt => {
            pcache.prey.and_then(|p| ecs.transforms.get(&p).map(|pt| (pt.x, pt.y)))
        }
        Goal::Flee => {
            let safe = mem.and_then(|m| {
                m.spatial.iter().filter(|(_, k)| k.danger < 0.1)
                    .min_by(|a, b| a.1.danger.partial_cmp(&b.1.danger).unwrap())
                    .map(|(&(cx, cy), _)| cell_center(cx, cy))
            });
            safe.or_else(|| {
                pcache.predator.and_then(|p| ecs.transforms.get(&p))
                    .map(|pt| (t.x + (t.x - pt.x).signum() * 120.0, t.y + (t.y - pt.y).signum() * 120.0))
            })
        }
        Goal::FollowPack => {
            let trusted = mem.and_then(|m| m.best_ally().and_then(|pid| ecs.identity.resolve(pid)));
            if let Some(a) = trusted {
                ecs.transforms.get(&a).map(|at| (at.x, at.y))
            } else {
                pcache.nearby_allies.first().and_then(|&a| ecs.transforms.get(&a).map(|at| (at.x, at.y)))
            }
        }
        Goal::Migrate => {
            let mut rng = rand::thread_rng();
            Some(cell_center(rng.gen_range(0..crate::world::cell::GRID_SIZE), rng.gen_range(0..crate::world::cell::GRID_SIZE)))
        }
        Goal::Explore => {
            let grid = crate::world::cell::GRID_SIZE;
            let mut rng = rand::thread_rng();
            for _ in 0..10 {
                let rx = rng.gen_range(0..grid);
                let ry = rng.gen_range(0..grid);
                if mem.map_or(true, |m| !m.spatial.contains_key(&(rx, ry))) {
                    return Some(cell_center(rx, ry));
                }
            }
            Some(cell_center(rng.gen_range(0..grid), rng.gen_range(0..grid)))
        }
        Goal::SeekShelter => {
            mem.and_then(|m| {
                m.spatial.iter()
                    .filter(|(_, k)| k.shelter > 0.2 && k.danger < 0.2)
                    .max_by(|a, b| a.1.shelter.partial_cmp(&b.1.shelter).unwrap())
                    .map(|(&(cx, cy), _)| cell_center(cx, cy))
            }).or_else(|| {
                pcache.nearby_allies.first().and_then(|&a| ecs.transforms.get(&a).map(|at| (at.x, at.y)))
            })
        }
        Goal::Mate => {
            perception::find_mate(ecs, entity)
                .and_then(|m| ecs.transforms.get(&m).map(|mt| (mt.x, mt.y)))
        }
        _ => None,
    }
}

fn decay_and_sense(ecs: &mut Ecs, entity: Entity, delta: f32, day_progress: f32, is_nocturnal: bool, pcache: &PerceptionCache) {
    let stress_tol = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.stress_tolerance);
    let energy_lvl = ecs.monster_traits.get(&entity).map_or(0.7, |t| t.energy_level);
    let bravery = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.bravery);
    let caution = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.caution);
    let pack_ment = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.pack_mentality);
    let adaptability = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.adaptability);
    let is_night = body::is_night(day_progress);
    let learning_mult = ecs.life_info.get(&entity).map_or(1.0, |li| li.life_stage().learning_mult());

    let night_sleep_mult = if is_nocturnal { if is_night { 0.5 } else { 2.0 } } else { if is_night { 2.5 } else { 1.0 } };

    if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
        needs::decay_personal_needs(pn, delta);
        pn.sleep = (pn.sleep + delta * 0.003 * night_sleep_mult).min(1.0);
        pn.discomfort = (pn.discomfort + delta * 0.002 * (1.0 - stress_tol)).min(1.0);
        pn.energy = (pn.energy + delta * 0.002 * energy_lvl).min(1.0);
    }
    if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) {
        eco.hunting = (eco.hunting + delta * 0.005).min(1.0);
        eco.territory_control = (eco.territory_control + delta * 0.003).min(1.0);
        eco.migration_urge = (eco.migration_urge + delta * 0.001).min(1.0);
        eco.pack_following = (eco.pack_following + delta * 0.002).min(1.0);
        eco.predator_avoidance = (eco.predator_avoidance - delta * 0.003).max(0.0);
    }
    if let Some(emo) = ecs.emotions.get_mut(&entity) {
        emo.decay(delta);
        if is_night && !is_nocturnal && bravery < 0.4 { emo.fear = (emo.fear + delta * 0.003).min(1.0); }
    }
    if let Some(mem) = ecs.memories.get_mut(&entity) { mem.decay_spatial(delta * 0.0005); }

    if let Some(predator) = pcache.predator {
        if let (Some(emo), Some(traits)) = (ecs.emotions.get_mut(&entity), ecs.monster_traits.get(&entity)) {
            emotions::apply_monster_personality(emo, traits, 0.0, delta * 0.15, 0.0, 0.0);
        }
        if let Some(pn) = ecs.personal_needs.get_mut(&entity) { pn.fear = (pn.fear + delta * 0.12 * caution * (1.0 - bravery)).min(1.0); }
        if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) { eco.predator_avoidance = (eco.predator_avoidance + delta * 0.05).min(1.0); }
        if let Some(loc) = ecs.transforms.get(&predator).map(|t| (t.cell_x, t.cell_y)) {
            if let Some(mem) = ecs.memories.get_mut(&entity) { mem.mark_cell(loc.0, loc.1, CellTag::Danger, 0.3); }
        }
    }

    let density = pcache.entities_nearby_count;
    if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) {
        eco.resource_competition = if density > 2 { (eco.resource_competition + delta * 0.008 * (1.0 - adaptability)).min(1.0) }
        else { (eco.resource_competition - delta * 0.005 * adaptability).max(0.0) };
    }

    if !pcache.nearby_allies.is_empty() {
        if let Some(pn) = ecs.personal_needs.get_mut(&entity) { pn.fear = (pn.fear - delta * 0.005 * pcache.nearby_allies.len() as f32).max(0.0); }
        if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.longing = (emo.longing - delta * 0.01).max(0.0); }
        for &a in &pcache.nearby_allies {
            if let Some(pid) = ecs.identity.persistent_id_of(a) {
                if let Some(mem) = ecs.memories.get_mut(&entity) {
                    let tick = ecs.tick;
                    mem.adjust_opinion(pid, |op| {
                        op.familiarity = (op.familiarity + delta * 0.002 * learning_mult).min(1.0);
                        op.last_seen_tick = tick;
                    });
                }
            }
        }
    } else if pack_ment > 0.5 {
        if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.longing = (emo.longing + delta * 0.005 * pack_ment).min(1.0); }
    }

    if let Some(loc) = ecs.transforms.get(&entity).map(|t| (t.cell_x, t.cell_y)) {
        if let Some(mem) = ecs.memories.get_mut(&entity) { mem.mark_cell(loc.0, loc.1, CellTag::Shelter, 0.02); }
    }
}

fn execute(ecs: &mut Ecs, events: &mut EventBus, res: &mut ResourceGrid, entity: Entity, plan: &Plan, delta: f32, body: &BodyState, tod: f32, pcache: &PerceptionCache) {
    let speed = delta * body.move_speed_mult * tod;
    match plan.goal {
        Goal::Hunt => {
            let aggr = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.aggressiveness);
            let pack_ment = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.pack_mentality);
            let prey_sel = ecs.ecosystem_needs.get(&entity).map_or(0.5, |e| e.prey_selection);

            if let Some(prey) = perception::find_prey_selective(ecs, entity, prey_sel) {
                move_toward(ecs, entity, prey, speed * (4.0 + aggr * 3.0) * body.combat_power_mult);
                if perception::distance2(ecs, entity, prey) < 25.0 * 25.0 {
                    combat::resolve_combat(ecs, events, entity, prey);
                }
            } else if pack_ment > 0.4 {
                let allies = &pcache.nearby_allies;
                if !allies.is_empty() {
                    if let Some(target) = perception::find_group_target(ecs, allies, entity) {
                        witness::communicate_rally(ecs, entity, 200.0);
                        move_toward(ecs, entity, target, speed * 4.0);
                        if perception::distance2(ecs, entity, target) < 35.0 * 35.0 {
                            let mut group: Vec<Entity> = allies.iter().copied()
                                .filter(|&a| perception::distance2(ecs, a, target) < 50.0 * 50.0).collect();
                            group.push(entity);
                            combat::resolve_group_combat(ecs, events, &group, target);
                        }
                    } else { wander(ecs, entity, speed * 3.0); }
                } else {
                    let trusted = ecs.memories.get(&entity)
                        .and_then(|m| m.best_ally().and_then(|pid| ecs.identity.resolve(pid)))
                        .filter(|&a| ecs.is_alive(a));
                    if let Some(ally) = trusted { move_toward(ecs, entity, ally, speed * 3.0); }
                    else { wander(ecs, entity, speed * 3.0); }
                }
            } else { wander(ecs, entity, speed * 3.0); }
            if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) { eco.hunting = (eco.hunting - delta * 0.01).max(0.0); }
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) { pn.energy = (pn.energy - delta * 0.015).max(0.0); }
        }
        Goal::Flee => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 10.0); }
            else {
                flee_from(ecs, entity, pcache.predator, speed * 10.0);
            }
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) { pn.fear = (pn.fear - delta * 0.03).max(0.0); pn.energy = (pn.energy - delta * 0.015).max(0.0); }
            if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.fear = (emo.fear - delta * 0.005).max(0.0); }
        }
        Goal::Rest => {
            let energy_lvl = ecs.monster_traits.get(&entity).map_or(0.7, |t| t.energy_level);
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
                needs::satisfy_sleep(pn, delta * 0.03);
                pn.health = (pn.health + delta * 0.01 * energy_lvl).min(1.0);
                pn.discomfort = (pn.discomfort - delta * 0.01).max(0.0);
            }
            if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.anger = (emo.anger - delta * 0.003).max(0.0); }
        }
        Goal::Sleep => {
            let energy_lvl = ecs.monster_traits.get(&entity).map_or(0.7, |t| t.energy_level);
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
                needs::satisfy_sleep(pn, delta * 0.06);
                pn.health = (pn.health + delta * 0.02 * energy_lvl).min(1.0);
                pn.energy = (pn.energy + delta * 0.015).min(1.0);
                pn.discomfort = (pn.discomfort - delta * 0.02).max(0.0);
            }
            if let Some(emo) = ecs.emotions.get_mut(&entity) {
                emo.fear = (emo.fear - delta * 0.008).max(0.0);
                emo.anger = (emo.anger - delta * 0.005).max(0.0);
            }
        }
        Goal::SeekShelter => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 4.0); }
            else {
                if let Some(&leader) = pcache.nearby_allies.first() { move_toward(ecs, entity, leader, speed * 3.0); }
                else { wander(ecs, entity, speed * 3.0); }
            }
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
                needs::satisfy_sleep(pn, delta * 0.01);
            }
            if let Some(t) = ecs.transforms.get(&entity).map(|t| (t.cell_x, t.cell_y)) {
                if let Some(mem) = ecs.memories.get_mut(&entity) { mem.mark_cell(t.0, t.1, CellTag::Shelter, 0.1); }
            }
        }
        Goal::DefendTerritory => {
            let aggr = ecs.monster_traits.get(&entity).map_or(0.5, |t| t.aggressiveness);
            let intruders = entities_in_radius(ecs, entity, 60.0);
            if let Some(&intruder) = intruders.first() {
                move_toward(ecs, entity, intruder, speed * (3.0 + aggr * 2.0));
                if perception::distance2(ecs, entity, intruder) < 30.0 * 30.0 {
                    combat::resolve_combat(ecs, events, entity, intruder);
                }
            }
            if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) { eco.territory_control = (eco.territory_control - delta * 0.015).max(0.0); }
        }
        Goal::Migrate => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 6.0); }
            else { wander(ecs, entity, speed * 6.0); }
            if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) { eco.migration_urge = (eco.migration_urge - delta * 0.02).max(0.0); eco.resource_competition = (eco.resource_competition - delta * 0.01).max(0.0); }
        }
        Goal::FollowPack => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 3.0); }
            else {
                if let Some(&leader) = pcache.nearby_allies.first() { move_toward(ecs, entity, leader, speed * 3.0); }
                else { wander(ecs, entity, speed * 2.0); }
            }
            if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) { eco.pack_following = (eco.pack_following - delta * 0.01).max(0.0); }
            if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.longing = (emo.longing - delta * 0.01).max(0.0); }
        }
        Goal::Explore => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 5.0); }
            else { wander(ecs, entity, speed * 5.0); }
            if let Some(pn) = ecs.personal_needs.get_mut(&entity) { pn.curiosity = (pn.curiosity - delta * 0.008).max(0.0); }
            if let Some(t) = ecs.transforms.get(&entity).map(|t| (t.cell_x, t.cell_y)) {
                let food = res.food_at(t.0, t.1);
                if let Some(mem) = ecs.memories.get_mut(&entity) { if food > 0.2 { mem.mark_cell(t.0, t.1, CellTag::Food, food * 0.5); } }
            }
        }
        Goal::SeekFood => {
            if let Some(target) = plan.target_pos { move_to_pos(ecs, entity, target, speed * 4.0); }
            else { wander(ecs, entity, speed * 3.0); }
            let hoarding = ecs.monster_traits.get(&entity).map_or(0.3, |t| t.hoarding);
            if let Some(t) = ecs.transforms.get(&entity) {
                let from_carcass = res.consume_carcass_near(t.x, t.y, delta * 0.06);
                if from_carcass > 0.0 {
                    if let Some(pn) = ecs.personal_needs.get_mut(&entity) { needs::satisfy_hunger(pn, from_carcass * (1.0 + hoarding * 0.3)); }
                    if let Some(mem) = ecs.memories.get_mut(&entity) { mem.mark_cell(t.cell_x, t.cell_y, CellTag::Food, 0.4); }
                } else {
                    let taken = res.consume_food(t.cell_x, t.cell_y, delta * (0.02 + hoarding * 0.01));
                    if taken > 0.01 {
                        if let Some(pn) = ecs.personal_needs.get_mut(&entity) { needs::satisfy_hunger(pn, taken); }
                        if let Some(mem) = ecs.memories.get_mut(&entity) { mem.mark_cell(t.cell_x, t.cell_y, CellTag::Food, 0.2); }
                    }
                }
            }
        }
        Goal::Mate => {
            if let Some(mate) = perception::find_mate(ecs, entity) {
                move_toward(ecs, entity, mate, speed * 2.0);
                if perception::distance2(ecs, entity, mate) < 20.0 * 20.0 {
                    crate::ai::reproduction::try_reproduce(ecs, entity, mate);
                    if let Some(emo) = ecs.emotions.get_mut(&entity) { emo.joy = (emo.joy + 0.15).min(1.0); }
                }
            } else { wander(ecs, entity, speed * 2.0); }
        }
        _ => {}
    }
}

fn mem_food_cell(mem: Option<&crate::ai::memory::Memory>) -> Option<(u32, u32)> {
    mem?.spatial.iter().filter(|(_, k)| k.food > 0.15)
        .max_by(|a, b| a.1.food.partial_cmp(&b.1.food).unwrap())
        .map(|(&pos, _)| pos)
}

fn cell_center(cx: u32, cy: u32) -> (f32, f32) {
    (cx as f32 * CELL_SIZE + CELL_SIZE * 0.5, cy as f32 * CELL_SIZE + CELL_SIZE * 0.5)
}

fn entities_in_radius(ecs: &Ecs, origin: Entity, radius: f32) -> Vec<Entity> {
    let ot = match ecs.transforms.get(&origin) { Some(t) => t, None => return Vec::new() };
    let r2 = radius * radius;
    let kind = ecs.kinds.get(&origin);
    ecs.spatial.candidates_in_radius(ot.x, ot.y, radius)
        .into_iter()
        .filter(|&e| {
            if e == origin { return false; }
            let same = match (kind, ecs.kinds.get(&e)) {
                (Some(EntityKind::Monster(a)), Some(EntityKind::Monster(b))) => a == b,
                _ => false,
            };
            if same { return false; }
            ecs.transforms.get(&e).map_or(false, |t| (t.x - ot.x).powi(2) + (t.y - ot.y).powi(2) < r2)
        })
        .collect()
}

fn move_to_pos(ecs: &mut Ecs, entity: Entity, target: (f32, f32), speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    if let Some(t) = ecs.transforms.get_mut(&entity) {
        let dx = target.0 - t.x; let dy = target.1 - t.y;
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        t.x = (t.x + dx / len * speed).clamp(0.0, ws);
        t.y = (t.y + dy / len * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}

fn move_toward(ecs: &mut Ecs, mover: Entity, target: Entity, speed: f32) {
    let (tx, ty) = match ecs.transforms.get(&target) { Some(t) => (t.x, t.y), None => return };
    move_to_pos(ecs, mover, (tx, ty), speed);
}

fn flee_from(ecs: &mut Ecs, entity: Entity, threat: Option<Entity>, speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    let mut rng = rand::thread_rng();
    let (dx, dy) = if let Some(t) = threat {
        let (tx, ty) = match ecs.transforms.get(&t) { Some(tr) => (tr.x, tr.y), None => (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) };
        let (mx, my) = match ecs.transforms.get(&entity) { Some(tr) => (tr.x, tr.y), None => return };
        (mx - tx, my - ty)
    } else { (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) };
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    if let Some(t) = ecs.transforms.get_mut(&entity) {
        t.x = (t.x + dx / len * speed).clamp(0.0, ws);
        t.y = (t.y + dy / len * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}

fn wander(ecs: &mut Ecs, entity: Entity, speed: f32) {
    let ws = crate::world::cell::WORLD_SIZE;
    let mut rng = rand::thread_rng();
    if let Some(t) = ecs.transforms.get_mut(&entity) {
        t.x = (t.x + rng.gen_range(-1.0..1.0) * speed).clamp(0.0, ws);
        t.y = (t.y + rng.gen_range(-1.0..1.0) * speed).clamp(0.0, ws);
        let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx; t.cell_y = cy;
    }
}
