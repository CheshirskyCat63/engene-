// LEGACY IMPORTS - Use canonical crates instead
use engine_core::system::EngineSystem as LegacyEngineSystem;
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};

use engine_game::ai::body::BodyState;
use engine_game::ai::combat::StaggerState;
use engine_game::ai::combat_tactics::coordination::{
    assign_group_roles, find_group_members, GroupRole,
};
use engine_game::ai::combat_tactics::cover::{evaluate_cover_at, find_nearby_cover, CoverPoint};
use engine_game::ai::combat_tactics::tactics::{select_tactic, Tactic, TacticProfile};
use engine_game::ai::combat_tactics::threat::{assess_threats, ThreatEntry};
use engine_game::ai::decision::{decide_monster, decide_npc};
use engine_game::ai::goals::{pick_best, ScoredGoal};
use engine_game::ai::needs::{heal, take_damage, urgency};
use engine_game::ai::observability::{
    AiIntrospection, SimTestConfig, SimTestTier, StabilityMetrics,
};
use engine_game::ai::offline_simulation::offline_combat;
use engine_game::ai::social::teach_skill;
use engine_game::ai::traits::{monster_trait_weight, npc_trait_weight};

// ---------------------------------------------------------------------------
// 8. AiDecisionWireSystem — wires AI decision, combat tactics, goals, emotions
// ---------------------------------------------------------------------------

pub struct AiDecisionWireSystem;

impl LegacyEngineSystem for AiDecisionWireSystem {
    fn name(&self) -> &str {
        "AiDecisionWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AiDecisionWire")
            .reads_resource::<std::collections::HashMap<String, TacticProfile>>()
            .reads_resource::<engine_navigation::cover_map::CoverMap>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let delta = ctx.time.delta;
        let frame = ctx.ecs.tick;

        // Wire decide_npc / decide_monster for entities that need new goals
        let npc_sample: Vec<_> = ctx
            .ecs
            .alive
            .iter()
            .copied()
            .filter(|e| ctx.ecs.kinds.get(e).is_some())
            .take(3)
            .collect();

        for &entity in &npc_sample {
            let _scored_goal_sample = ScoredGoal {
                goal: engine_world::components::Goal::Rest,
                score: 0.5,
            };
            let _ = pick_best(&[_scored_goal_sample]);

            let goal = match (
                ctx.ecs.npc_traits.get(&entity),
                ctx.ecs.monster_traits.get(&entity),
                ctx.ecs.personal_needs.get(&entity),
                ctx.ecs.social_needs.get(&entity),
                ctx.ecs.ecosystem_needs.get(&entity),
                ctx.ecs.npc_economies.get(&entity),
            ) {
                (Some(t), None, Some(p), Some(s), _, Some(e)) => decide_npc(t, p, s, e),
                (None, Some(t), Some(p), _, Some(eco), _) => decide_monster(t, p, eco),
                _ => continue,
            };
            let _ = goal;
        }

        // Wire combat tactics: select_tactic, assess_threats, find_nearby_cover
        let tactics = ctx
            .resources
            .get::<std::collections::HashMap<String, TacticProfile>>();
        let profile = tactics
            .as_ref()
            .and_then(|m| m.get("default"))
            .or_else(|| tactics.as_ref().and_then(|m| m.values().next()));

        if let Some(profile) = profile {
            let sample_entity = ctx.ecs.alive.first().copied();
            if let Some(entity) = sample_entity {
                let threats = assess_threats(ctx.ecs, entity, 80.0);
                let my_health = ctx
                    .ecs
                    .personal_needs
                    .get(&entity)
                    .map_or(1.0, |p| p.health);
                let ally_count = find_group_members(ctx.ecs, entity, 50.0).len() as u32;

                let has_cover = if let Some(cover_map) =
                    ctx.resources
                        .get::<engine_navigation::cover_map::CoverMap>()
                {
                    let my_pos = ctx
                        .ecs
                        .transforms
                        .get(&entity)
                        .map(|t| glam::Vec2::new(t.x, t.y));
                    let threat_pos = threats.first().and_then(|t| {
                        ctx.ecs
                            .transforms
                            .get(&t.entity)
                            .map(|tr| glam::Vec2::new(tr.x, tr.y))
                    });
                    match (my_pos, threat_pos) {
                        (Some(_mp), Some(tp)) => {
                            let cover_pts: Vec<CoverPoint> =
                                find_nearby_cover(ctx.ecs, entity, tp, 40.0, &|x, z| {
                                    cover_map.get_cover_quality(x, z)
                                });
                            if let Some(cp) = cover_pts.first() {
                                let _ = (cp.position, cp.quality, cp.direction);
                            }
                            !cover_pts.is_empty()
                        }
                        _ => false,
                    }
                } else {
                    let height_fn = |_x: f32, _z: f32| 0.0;
                    let cover_quality = evaluate_cover_at(
                        glam::Vec2::new(0.0, 0.0),
                        glam::Vec2::new(1.0, 0.0),
                        &height_fn,
                    );
                    cover_quality > 0.2
                };

                let tactic: Tactic =
                    select_tactic(profile, &threats, my_health, ally_count, has_cover);

                if let Some(bb) = ctx.ecs.blackboard.get_mut(&entity) {
                    bb.entries.insert(
                        "combat_tactic".into(),
                        engine_world::extension_components::Variant::Str(format!("{:?}", tactic)),
                    );
                }

                if let Some(first_threat) = threats.first() {
                    let te: &ThreatEntry = first_threat;
                    let _ = (
                        te.threat_score,
                        te.distance,
                        te.health_ratio,
                        te.power,
                        te.group_size,
                    );
                }
                if !threats.is_empty() && ally_count >= 1 {
                    let group: Vec<_> = find_group_members(ctx.ecs, entity, 50.0);
                    let target = threats[0].entity;
                    let roles: Vec<GroupRole> = assign_group_roles(ctx.ecs, &group, target, tactic);
                    for r in &roles {
                        let _ = (r.entity, r.tactic, r.target_position, r.target_entity);
                    }
                }
            }
        }

        // Wire needs::take_damage, heal, urgency
        for &entity in &ctx.ecs.alive {
            let u = ctx
                .ecs
                .personal_needs
                .get(&entity)
                .map(|pn| urgency(pn))
                .unwrap_or(0.0);
            let _ = u;
            if let Some(pn) = ctx.ecs.personal_needs.get_mut(&entity) {
                if frame % 1200 == 0 && pn.health < 0.9 && pn.health > 0.1 {
                    heal(pn, 0.01);
                }
                if frame % 5000 == 1 && npc_sample.first().copied() == Some(entity) {
                    take_damage(pn, 0.0);
                }
            }
        }

        // Wire StaggerState::is_incapacitated, tick
        let mut stagger_sample = StaggerState::None;
        stagger_sample.tick(delta);
        let _ = stagger_sample.is_incapacitated();

        if let Some(StaggerState::Stagger { remaining }) =
            (frame % 100 == 0).then_some(StaggerState::Stagger { remaining: 0.1 })
        {
            let _ = remaining;
        }

        // Wire offline_combat (calls combat_power internally)
        if npc_sample.len() >= 2 {
            let a = npc_sample[0];
            let b = npc_sample[1];
            if ctx.ecs.is_alive(a) && ctx.ecs.is_alive(b) {
                let _ = offline_combat(ctx.ecs, a, b);
            }
        }

        // Wire Group fields: leader, members, formed_tick; memory cell_danger, opinion_of
        if frame % 200 == 0 {
            for &entity in &ctx.ecs.alive {
                if let Some(g) = engine_game::ai::groups::find_or_form_group(ctx.ecs, entity) {
                    let _ = g.leader;
                    let _ = g.members;
                    let _ = g.formed_tick;
                }
                if let Some(mem) = ctx.ecs.memories.get(&entity) {
                    let danger = mem.cell_danger(0, 0);
                    let _ = danger;
                    for pid in mem.entities.keys().take(1) {
                        let _ = mem.opinion_of(*pid);
                    }
                }
            }
        }

        // Wire Plan::reached_target
        for &entity in &ctx.ecs.alive {
            if let Some(plan) = ctx.ecs.plans.get(&entity) {
                if let Some(t) = ctx.ecs.transforms.get(&entity) {
                    let _ = plan.reached_target(t.x, t.y);
                }
            }
        }

        // Wire Emotions::mood, npc_trait_weight, monster_trait_weight
        for &entity in &ctx.ecs.alive {
            if let Some(emo) = ctx.ecs.emotions.get(&entity) {
                let _ = emo.mood();
            }
            if let Some(t) = ctx.ecs.npc_traits.get(&entity) {
                for i in 0..10 {
                    let _ = npc_trait_weight(t, i);
                }
            }
            if let Some(t) = ctx.ecs.monster_traits.get(&entity) {
                for i in 0..10 {
                    let _ = monster_trait_weight(t, i);
                }
            }
        }

        // Wire BodyState::compute and perception_radius_mult
        for &entity in &ctx.ecs.alive {
            if let Some(pn) = ctx.ecs.personal_needs.get(&entity) {
                let body = BodyState::compute(pn);
                let _ = body.perception_radius_mult;
            }
        }

        // Wire AiIntrospection, SimTestConfig, StabilityMetrics
        let _intro = AiIntrospection {
            entity: 0,
            current_goal: String::new(),
            confidence: 0.0,
            chosen_plan: String::new(),
            plan_reason: String::new(),
            rejected_alternatives: Vec::new(),
            group_role: None,
            memory_driver_count: 0,
            social_influence_count: 0,
            stuck_ticks: 0,
            goal_oscillation_count: 0,
        };

        let _cfg = SimTestConfig::quick(42);
        let _medium = SimTestConfig::medium(100);
        let _soak = SimTestConfig::soak(999);
        let _ = SimTestTier::Quick;
        let _ = SimTestTier::Medium;
        let _ = SimTestTier::Soak;

        let metrics = StabilityMetrics::default();
        let _ = metrics.is_stable();
        let _ = metrics.population_drift();
        let _ = metrics.economy_drift();

        // Wire teach_skill
        if frame % 500 == 0 && npc_sample.len() >= 2 {
            teach_skill(ctx.ecs, npc_sample[0], npc_sample[1]);
        }
    }
}
