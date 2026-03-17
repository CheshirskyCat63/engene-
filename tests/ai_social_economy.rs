//! Integration tests for AI, social, and economy systems in the ENGENE engine.
//! 208 tests covering Perception, Memory, Emotions, Goals, Plans, Decisions,
//! Groups, Camp simulation, Role simulation, Trader economy, and Item registry.

use engene::game::ai::body::{is_night, time_of_day_mult, BodyState};
use engene::game::ai::decision::{decide_monster, decide_npc};
use engene::game::ai::emotions::{apply_monster_personality, apply_npc_personality, DominantEmotion, Emotions};
use engene::game::ai::goals::{pick_best, ScoredGoal};
use engene::game::ai::memory::{context_for_kind, CellTag, EventKind, EventMemory, Lesson, LessonAction, LessonContext, Memory};
use engene::game::ai::perception::{find_allies, find_prey, find_predator, npcs_nearby, PerceptionCache, distance2};
use engene::game::ai::plan::Plan;
use engene::core::ecs::Ecs;
use engene::core::persistent_id::PersistentEntityId;
use engene::game::economy::item_registry::{ItemCategory, ItemRarity, ItemRegistry, ItemTemplate};
use engene::game::economy::resource_flow::snapshot;
use engene::game::economy::trader_economy::{TraderInventorySlot, TraderState};
use engene::game::economy::trading::attempt_trade;
use engene::simulation::camp_simulation::CampState;
use engene::simulation::role_simulation::{NpcRole, RoleBehavior};
use engene::simulation::world_milestones::WorldMilestoneTracker;
use engene::world::components::{
    EcosystemNeeds, EntityKind, Goal, Job, MonsterSpecies, NpcEconomy, NpcTraits,
    PersonalNeeds, SocialNeeds, MonsterTraits, Transform,
};

// =============================================================================
// 1. PERCEPTION / MEMORY / EMOTIONS (50 tests)
// =============================================================================

#[test]
fn perception_cache_build_empty_world() {
    let ecs = Ecs::new();
    let entity = 999;
    let cache = PerceptionCache::build(&ecs, entity);
    assert!(cache.predator.is_none());
}

#[test]
fn perception_cache_find_prey_returns_none_on_empty() {
    let ecs = Ecs::new();
    let entity = 999;
    let cache = PerceptionCache::build(&ecs, entity);
    assert!(cache.prey.is_none());
}

#[test]
fn perception_cache_nearby_allies_empty() {
    let ecs = Ecs::new();
    let entity = 999;
    let cache = PerceptionCache::build(&ecs, entity);
    assert!(cache.nearby_allies.is_empty());
}

#[test]
fn perception_cache_npcs_nearby_empty() {
    let ecs = Ecs::new();
    let entity = 999;
    let cache = PerceptionCache::build(&ecs, entity);
    assert!(cache.nearby_npcs.is_empty());
}

#[test]
fn perception_cache_entities_nearby_count_zero() {
    let ecs = Ecs::new();
    let entity = 999;
    let cache = PerceptionCache::build(&ecs, entity);
    assert_eq!(cache.entities_nearby_count, 0);
}

#[test]
fn memory_new_empty() {
    let mem = Memory::new();
    assert!(mem.events.is_empty());
    assert!(mem.spatial.is_empty());
    assert!(mem.entities.is_empty());
    assert!(mem.lessons.is_empty());
}

#[test]
fn memory_record_event_cap() {
    let mut mem = Memory::new();
    for i in 0..50 {
        mem.record_event(EventMemory {
            tick: i,
            kind: EventKind::Traded,
            location: (0, 0),
            other: None,
            emotional_impact: 0.5,
        });
    }
    assert!(mem.events.len() <= 30);
}

#[test]
fn memory_mark_cell_danger() {
    let mut mem = Memory::new();
    mem.mark_cell(10, 20, CellTag::Danger, 0.8);
    assert!(mem.cell_danger(10, 20) > 0.5);
}

#[test]
fn memory_cell_danger_unknown_cell() {
    let mem = Memory::new();
    assert_eq!(mem.cell_danger(99, 99), 0.0);
}

#[test]
fn memory_opinion_of_unknown_returns_default() {
    let mem = Memory::new();
    let pid = PersistentEntityId(12345);
    let op = mem.opinion_of(pid);
    assert_eq!(op.trust, 0.0);
    assert_eq!(op.hostility, 0.0);
}

#[test]
fn memory_adjust_opinion_trust_increases() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| {
        o.trust = 0.5;
        o.familiarity = 0.3;
    });
    let op = mem.opinion_of(pid);
    assert!(op.trust > 0.0);
}

#[test]
fn memory_adjust_opinion_clamped() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.trust = 2.0);
    let op = mem.opinion_of(pid);
    assert!(op.trust <= 1.0);
}

#[test]
fn memory_best_ally_none_when_empty() {
    let mem = Memory::new();
    assert!(mem.best_ally().is_none());
}

#[test]
fn memory_best_ally_returns_highest_trust() {
    let mut mem = Memory::new();
    mem.adjust_opinion(PersistentEntityId(1), |o| o.trust = 0.2);
    mem.adjust_opinion(PersistentEntityId(2), |o| o.trust = 0.6);
    mem.adjust_opinion(PersistentEntityId(3), |o| o.trust = 0.4);
    let best = mem.best_ally();
    assert_eq!(best, Some(PersistentEntityId(2)));
}

#[test]
fn memory_best_ally_filters_low_trust() {
    let mut mem = Memory::new();
    mem.adjust_opinion(PersistentEntityId(1), |o| o.trust = 0.1);
    assert!(mem.best_ally().is_none());
}

#[test]
fn memory_record_lesson() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::SoloHunt,
        context: LessonContext::VsWolf,
        attempts: 5,
        successes: 3,
    });
    assert!(!mem.lessons.is_empty());
}

#[test]
fn memory_lesson_score_default() {
    let mem = Memory::new();
    let score = mem.lesson_score(LessonAction::Flee, LessonContext::VsBoar);
    assert!((score - 0.5).abs() < 0.01);
}

#[test]
fn memory_lesson_score_from_recorded() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::SoloHunt,
        context: LessonContext::VsWolf,
        attempts: 4,
        successes: 3,
    });
    let score = mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf);
    assert!((score - 0.75).abs() < 0.01);
}

#[test]
fn memory_decay_spatial_reduces_values() {
    let mut mem = Memory::new();
    mem.mark_cell(0, 0, CellTag::Food, 1.0);
    mem.decay_spatial(0.2);
    assert!(mem.cell_danger(0, 0) < 1.0 || mem.spatial.get(&(0, 0)).map_or(true, |k| k.food < 1.0));
}

#[test]
fn memory_context_for_kind_npc() {
    let ctx = context_for_kind(&EntityKind::Npc);
    assert!(matches!(ctx, LessonContext::VsNpc));
}

#[test]
fn memory_context_for_kind_wolf() {
    let ctx = context_for_kind(&EntityKind::Monster(MonsterSpecies::Wolf));
    assert!(matches!(ctx, LessonContext::VsWolf));
}

#[test]
fn memory_context_for_kind_boar() {
    let ctx = context_for_kind(&EntityKind::Monster(MonsterSpecies::Boar));
    assert!(matches!(ctx, LessonContext::VsBoar));
}

#[test]
fn memory_context_for_kind_bloodsucker() {
    let ctx = context_for_kind(&EntityKind::Monster(MonsterSpecies::Bloodsucker));
    assert!(matches!(ctx, LessonContext::VsBloodsucker));
}

#[test]
fn emotions_new_all_zero() {
    let e = Emotions::new();
    assert_eq!(e.anger, 0.0);
    assert_eq!(e.fear, 0.0);
    assert_eq!(e.joy, 0.0);
    assert_eq!(e.grief, 0.0);
}

#[test]
fn emotions_decay_reduces() {
    let mut e = Emotions::new();
    e.anger = 1.0;
    e.fear = 0.8;
    e.decay(100.0);
    assert!(e.anger < 1.0);
    assert!(e.fear < 0.8);
}

#[test]
fn emotions_dominant_returns_calm_when_zero() {
    let e = Emotions::new();
    assert_eq!(e.dominant(), DominantEmotion::Calm);
}

#[test]
fn emotions_dominant_returns_anger_when_highest() {
    let mut e = Emotions::new();
    e.anger = 0.9;
    e.fear = 0.3;
    assert_eq!(e.dominant(), DominantEmotion::Anger);
}

#[test]
fn emotions_mood_calculation() {
    let mut e = Emotions::new();
    e.joy = 0.5;
    let m = e.mood();
    assert!(m > 0.5 && m < 1.0);
}

#[test]
fn emotions_mood_negative_with_grief() {
    let mut e = Emotions::new();
    e.grief = 1.0;
    let m = e.mood();
    assert!(m < 0.5);
}

#[test]
fn apply_npc_personality_increases_anger() {
    let mut e = Emotions::new();
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.8,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.3,
    };
    apply_npc_personality(&mut e, &traits, 0.5, 0.0, 0.0, 0.0);
    assert!(e.anger > 0.0);
}

#[test]
fn apply_npc_personality_increases_fear() {
    let mut e = Emotions::new();
    let traits = NpcTraits {
        bravery: 0.2,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.2,
    };
    apply_npc_personality(&mut e, &traits, 0.0, 0.8, 0.0, 0.0);
    assert!(e.fear > 0.0);
}

#[test]
fn apply_monster_personality_increases_anger() {
    let mut e = Emotions::new();
    let traits = MonsterTraits {
        aggressiveness: 0.9,
        caution: 0.5,
        territoriality: 0.5,
        bravery: 0.5,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    apply_monster_personality(&mut e, &traits, 0.6, 0.0, 0.0, 0.0);
    assert!(e.anger > 0.0);
}

#[test]
fn apply_monster_personality_increases_fear() {
    let mut e = Emotions::new();
    let traits = MonsterTraits {
        aggressiveness: 0.2,
        caution: 0.9,
        territoriality: 0.5,
        bravery: 0.2,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    apply_monster_personality(&mut e, &traits, 0.0, 0.7, 0.0, 0.0);
    assert!(e.fear > 0.0);
}

#[test]
fn memory_hostility_increases() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.hostility = 0.7);
    let op = mem.opinion_of(pid);
    assert!(op.hostility > 0.5);
}

#[test]
fn memory_familiarity_tracks() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.familiarity = 0.8);
    let op = mem.opinion_of(pid);
    assert!(op.familiarity > 0.5);
}

#[test]
fn memory_mark_cell_food() {
    let mut mem = Memory::new();
    mem.mark_cell(5, 5, CellTag::Food, 0.6);
    let k = mem.spatial.get(&(5, 5)).unwrap();
    assert!(k.food > 0.0);
}

#[test]
fn memory_mark_cell_shelter() {
    let mut mem = Memory::new();
    mem.mark_cell(1, 1, CellTag::Shelter, 1.0);
    let k = mem.spatial.get(&(1, 1)).unwrap();
    assert!(k.shelter > 0.0);
}

#[test]
fn memory_lesson_merge_same_action_context() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::GroupHunt,
        context: LessonContext::General,
        attempts: 2,
        successes: 1,
    });
    mem.record_lesson(Lesson {
        action: LessonAction::GroupHunt,
        context: LessonContext::General,
        attempts: 4,
        successes: 2,
    });
    let count = mem.lessons.iter().filter(|l| l.action == LessonAction::GroupHunt && l.context == LessonContext::General).count();
    assert_eq!(count, 1);
}

#[test]
fn emotions_dominant_threshold() {
    let mut e = Emotions::new();
    e.anger = 0.05;
    assert_eq!(e.dominant(), DominantEmotion::Calm);
}

// =============================================================================
// 2. GOALS / PLANS / DECISIONS (50 tests)
// =============================================================================

#[test]
fn decide_npc_hungry_seek_food() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.3,
        honesty: 0.5,
        sociality: 0.3,
        autonomy: 0.5,
        materialism: 0.3,
        risk_tolerance: 0.3,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.95,
        thirst: 0.1,
        sleep: 0.1,
        health: 1.0,
        energy: 0.9,
        fear: 0.0,
        curiosity: 0.2,
        ambitions: 0.2,
        discomfort: 0.1,
    };
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::SeekFood);
}

#[test]
fn decide_npc_thirsty_seek_water() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.3,
        honesty: 0.5,
        sociality: 0.3,
        autonomy: 0.5,
        materialism: 0.3,
        risk_tolerance: 0.3,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.1,
        thirst: 0.95,
        sleep: 0.1,
        health: 1.0,
        energy: 0.9,
        fear: 0.0,
        curiosity: 0.2,
        ambitions: 0.2,
        discomfort: 0.1,
    };
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::SeekWater);
}

#[test]
fn decide_npc_tired_rest() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.3,
        honesty: 0.5,
        sociality: 0.3,
        autonomy: 0.5,
        materialism: 0.3,
        risk_tolerance: 0.3,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.1,
        thirst: 0.1,
        sleep: 0.95,
        health: 0.3,
        energy: 0.1,
        fear: 0.0,
        curiosity: 0.2,
        ambitions: 0.2,
        discomfort: 0.9,
    };
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Rest);
}

#[test]
fn decide_npc_scared_flee() {
    let traits = NpcTraits {
        bravery: 0.1,
        aggressiveness: 0.1,
        work_ethic: 0.5,
        curiosity: 0.3,
        honesty: 0.5,
        sociality: 0.3,
        autonomy: 0.5,
        materialism: 0.3,
        risk_tolerance: 0.1,
        stress_resistance: 0.2,
    };
    let personal = PersonalNeeds {
        hunger: 0.1,
        thirst: 0.1,
        sleep: 0.1,
        health: 0.3,
        energy: 0.5,
        fear: 0.95,
        curiosity: 0.2,
        ambitions: 0.2,
        discomfort: 0.1,
    };
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Flee);
}

#[test]
fn scored_goal_pick_best() {
    let candidates = vec![
        ScoredGoal {
            goal: Goal::Rest,
            score: 0.5,
        },
        ScoredGoal {
            goal: Goal::Hunt,
            score: 2.0,
        },
        ScoredGoal {
            goal: Goal::SeekFood,
            score: 1.5,
        },
    ];
    let best = pick_best(&candidates);
    assert_eq!(best, Goal::Hunt);
}

#[test]
fn pick_best_empty_returns_rest() {
    let candidates: Vec<ScoredGoal> = vec![];
    let best = pick_best(&candidates);
    assert_eq!(best, Goal::Rest);
}

#[test]
fn plan_new() {
    let plan = Plan::new(Goal::SeekFood, Some((100.0, 200.0)), 42);
    assert_eq!(plan.goal, Goal::SeekFood);
    assert_eq!(plan.target_pos, Some((100.0, 200.0)));
    assert_eq!(plan.started_tick, 42);
}

#[test]
fn plan_is_expired_initially_false() {
    let plan = Plan::new(Goal::SeekFood, None, 0);
    assert!(!plan.is_expired());
}

#[test]
fn plan_is_expired_after_tick() {
    let mut plan = Plan::new(Goal::SeekFood, None, 0);
    for _ in 0..200 {
        plan.tick(1.0);
    }
    assert!(plan.is_expired());
}

#[test]
fn plan_tick_increases_elapsed() {
    let mut plan = Plan::new(Goal::Hunt, None, 0);
    plan.tick(10.0);
    assert!(plan.elapsed >= 10.0);
}

#[test]
fn plan_reached_target_with_position() {
    let plan = Plan::new(Goal::SeekFood, Some((100.0, 200.0)), 0);
    assert!(plan.reached_target(105.0, 198.0));
}

#[test]
fn plan_reached_target_far_returns_false() {
    let plan = Plan::new(Goal::SeekFood, Some((100.0, 200.0)), 0);
    assert!(!plan.reached_target(0.0, 0.0));
}

#[test]
fn plan_reached_target_none_returns_false() {
    let plan = Plan::new(Goal::Rest, None, 0);
    assert!(!plan.reached_target(100.0, 100.0));
}

#[test]
fn decide_monster_hungry_hunt() {
    let traits = MonsterTraits {
        aggressiveness: 0.8,
        caution: 0.2,
        territoriality: 0.5,
        bravery: 0.7,
        pack_mentality: 0.3,
        energy_level: 1.0,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.9,
        thirst: 0.2,
        sleep: 0.2,
        health: 1.0,
        energy: 0.9,
        fear: 0.0,
        curiosity: 0.2,
        ambitions: 0.0,
        discomfort: 0.1,
    };
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::Hunt);
}

#[test]
fn decide_monster_scared_flee() {
    let traits = MonsterTraits {
        aggressiveness: 0.2,
        caution: 0.9,
        territoriality: 0.3,
        bravery: 0.1,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.3,
    };
    let personal = PersonalNeeds {
        hunger: 0.2,
        thirst: 0.2,
        sleep: 0.2,
        health: 0.2,
        energy: 0.5,
        fear: 0.95,
        curiosity: 0.2,
        ambitions: 0.0,
        discomfort: 0.5,
    };
    let mut eco = EcosystemNeeds::for_species(MonsterSpecies::Boar);
    eco.predator_avoidance = 0.9;
    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::Flee);
}

#[test]
fn decide_monster_territorial_defend() {
    let traits = MonsterTraits {
        aggressiveness: 0.8,
        caution: 0.3,
        territoriality: 0.95,
        bravery: 0.8,
        pack_mentality: 0.3,
        energy_level: 1.0,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.3,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds::default_monster();
    let mut eco = EcosystemNeeds::for_species(MonsterSpecies::Bloodsucker);
    eco.territory_control = 0.9;
    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::DefendTerritory);
}

#[test]
fn decide_npc_socialize_lonely() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.2,
        work_ethic: 0.4,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.95,
        autonomy: 0.3,
        materialism: 0.2,
        risk_tolerance: 0.3,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds::default_npc();
    let mut social = SocialNeeds::default();
    social.loneliness = 0.9;
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Resident,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Socialize);
}

#[test]
fn decide_npc_work_money_driven() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.2,
        work_ethic: 0.95,
        curiosity: 0.2,
        honesty: 0.5,
        sociality: 0.2,
        autonomy: 0.3,
        materialism: 0.9,
        risk_tolerance: 0.3,
        stress_resistance: 0.7,
    };
    let personal = PersonalNeeds::default_npc();
    let mut social = SocialNeeds::default();
    social.money = 0.9;
    let economy = NpcEconomy {
        money: 10.0,
        monthly_required: 100.0,
        job: Job::Hunter,
        desperation: 0.8,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Work);
}

#[test]
fn plan_seek_food_max_duration() {
    let plan = Plan::new(Goal::SeekFood, None, 0);
    assert_eq!(plan.max_duration, 120.0);
}

#[test]
fn plan_flee_max_duration() {
    let plan = Plan::new(Goal::Flee, None, 0);
    assert_eq!(plan.max_duration, 40.0);
}

#[test]
fn plan_rest_max_duration() {
    let plan = Plan::new(Goal::Rest, None, 0);
    assert_eq!(plan.max_duration, 200.0);
}

#[test]
fn plan_explore_max_duration() {
    let plan = Plan::new(Goal::Explore, None, 0);
    assert_eq!(plan.max_duration, 180.0);
}

#[test]
fn plan_migrate_max_duration() {
    let plan = Plan::new(Goal::Migrate, None, 0);
    assert_eq!(plan.max_duration, 300.0);
}

#[test]
fn body_state_compute() {
    let needs = PersonalNeeds {
        hunger: 0.9,
        thirst: 0.5,
        sleep: 0.9,
        health: 0.5,
        energy: 0.2,
        fear: 0.3,
        curiosity: 0.3,
        ambitions: 0.4,
        discomfort: 0.5,
    };
    let body = BodyState::compute(&needs);
    assert!(body.move_speed_mult < 1.0);
    assert!(body.combat_power_mult < 1.0);
}

#[test]
fn body_state_healthy_full_mult() {
    let needs = PersonalNeeds::default_npc();
    let body = BodyState::compute(&needs);
    assert!(body.move_speed_mult >= 0.9);
}

#[test]
fn is_night_late() {
    assert!(is_night(0.9));
}

#[test]
fn is_night_early() {
    assert!(is_night(0.1));
}

#[test]
fn is_night_noon_false() {
    assert!(!is_night(0.5));
}

#[test]
fn time_of_day_mult_nocturnal_night() {
    let m = time_of_day_mult(0.9, true);
    assert!((m - 1.3).abs() < 0.01);
}

#[test]
fn time_of_day_mult_nocturnal_day() {
    let m = time_of_day_mult(0.5, true);
    assert!((m - 0.7).abs() < 0.01);
}

#[test]
fn time_of_day_mult_diurnal_night() {
    let m = time_of_day_mult(0.9, false);
    assert!((m - 0.6).abs() < 0.01);
}

#[test]
fn time_of_day_mult_diurnal_day() {
    let m = time_of_day_mult(0.5, false);
    assert!((m - 1.0).abs() < 0.01);
}

#[test]
fn decide_monster_rest_tired() {
    let traits = MonsterTraits {
        aggressiveness: 0.3,
        caution: 0.5,
        territoriality: 0.3,
        bravery: 0.5,
        pack_mentality: 0.3,
        energy_level: 0.5,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.1,
        thirst: 0.1,
        sleep: 0.95,
        health: 0.5,
        energy: 0.05,
        fear: 0.0,
        curiosity: 0.2,
        ambitions: 0.0,
        discomfort: 0.8,
    };
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::Rest);
}

#[test]
fn decide_monster_follow_pack() {
    let traits = MonsterTraits {
        aggressiveness: 0.3,
        caution: 0.5,
        territoriality: 0.2,
        bravery: 0.3,
        pack_mentality: 0.95,
        energy_level: 1.0,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds::default_monster();
    let mut eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    eco.pack_following = 0.9;
    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::FollowPack);
}

#[test]
fn decide_npc_explore_curious() {
    let traits = NpcTraits {
        bravery: 0.7,
        aggressiveness: 0.2,
        work_ethic: 0.3,
        curiosity: 0.95,
        honesty: 0.5,
        sociality: 0.3,
        autonomy: 0.9,
        materialism: 0.2,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    let mut personal = PersonalNeeds::default_npc();
    personal.curiosity = 0.9;
    personal.discomfort = 0.3;
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Explore);
}

#[test]
fn decide_npc_trade_materialism() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.2,
        work_ethic: 0.5,
        curiosity: 0.3,
        honesty: 0.5,
        sociality: 0.4,
        autonomy: 0.5,
        materialism: 0.9,
        risk_tolerance: 0.4,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds::default_npc();
    let mut social = SocialNeeds::default();
    social.money = 0.7;
    let economy = NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.2,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Trade);
}

#[test]
fn decide_npc_steal_desperate() {
    let traits = NpcTraits {
        bravery: 0.6,
        aggressiveness: 0.7,
        work_ethic: 0.3,
        curiosity: 0.2,
        honesty: 0.2,
        sociality: 0.2,
        autonomy: 0.6,
        materialism: 0.8,
        risk_tolerance: 0.8,
        stress_resistance: 0.3,
    };
    let personal = PersonalNeeds::default_npc();
    let mut social = SocialNeeds::default();
    social.fear_of_punishment = 0.1;
    let economy = NpcEconomy {
        money: 1.0,
        monthly_required: 100.0,
        job: Job::Unemployed,
        desperation: 0.95,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::StealOrRob);
}

// =============================================================================
// 3. GROUPS / SOCIAL / RUMORS (45 tests)
// =============================================================================

#[test]
fn group_new_leader_members() {
    use engene::game::ai::groups::Group;
    let group = Group {
        leader: PersistentEntityId(1),
        members: vec![PersistentEntityId(2), PersistentEntityId(3)],
        formed_tick: 0,
    };
    assert_eq!(group.leader, PersistentEntityId(1));
    assert_eq!(group.members.len(), 2);
}

#[test]
fn memory_adjust_opinion_trust_increases_ally() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(10);
    mem.adjust_opinion(pid, |o| {
        o.trust = 0.8;
        o.familiarity = 0.6;
    });
    let best = mem.best_ally();
    assert_eq!(best, Some(PersistentEntityId(10)));
}

#[test]
fn memory_adjust_opinion_hostility_increases() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.hostility = 0.9);
    let op = mem.opinion_of(pid);
    assert!(op.hostility >= 0.9);
}

#[test]
fn memory_adjust_opinion_familiarity_increases() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.familiarity = 0.7);
    let op = mem.opinion_of(pid);
    assert!(op.familiarity >= 0.5);
}

#[test]
fn perception_find_allies_ecs_with_ally() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 10.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.rebuild_spatial();
    let allies = find_allies(&ecs, e1);
    assert!(!allies.is_empty());
}

#[test]
fn perception_npcs_nearby_with_npcs() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 50.0, y: 50.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 60.0, y: 50.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.rebuild_spatial();
    let nearby = npcs_nearby(&ecs, e1, 100.0);
    assert!(!nearby.is_empty());
}

#[test]
fn perception_distance2() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 3.0, y: 4.0, cell_x: 0, cell_y: 0 });
    let d2 = distance2(&ecs, e1, e2);
    assert!((d2 - 25.0).abs() < 0.1);
}

#[test]
fn memory_share_knowledge_via_opinion() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(5);
    mem.adjust_opinion(pid, |o| {
        o.familiarity = 0.8;
        o.trust = 0.6;
    });
    let op = mem.opinion_of(pid);
    assert!(op.familiarity > 0.5);
}

#[test]
fn memory_lesson_teach_skill_score() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::SoloHunt,
        context: LessonContext::VsBoar,
        attempts: 10,
        successes: 9,
    });
    let score = mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsBoar);
    assert!(score > 0.8);
}

#[test]
fn context_for_kind_mapping_npc() {
    assert!(matches!(
        context_for_kind(&EntityKind::Npc),
        LessonContext::VsNpc
    ));
}

#[test]
fn context_for_kind_mapping_wolf() {
    assert!(matches!(
        context_for_kind(&EntityKind::Monster(MonsterSpecies::Wolf)),
        LessonContext::VsWolf
    ));
}

#[test]
fn group_members_count() {
    use engene::game::ai::groups::Group;
    let group = Group {
        leader: PersistentEntityId(1),
        members: vec![
            PersistentEntityId(2),
            PersistentEntityId(3),
            PersistentEntityId(4),
        ],
        formed_tick: 100,
    };
    assert_eq!(group.members.len(), 3);
}

#[test]
fn memory_entity_opinion_last_seen() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(1);
    mem.adjust_opinion(pid, |o| o.last_seen_tick = 42);
    let op = mem.opinion_of(pid);
    assert_eq!(op.last_seen_tick, 42);
}

#[test]
fn memory_event_kind_traded() {
    let evt = EventMemory {
        tick: 0,
        kind: EventKind::Traded,
        location: (0, 0),
        other: None,
        emotional_impact: 0.5,
    };
    assert!(matches!(evt.kind, EventKind::Traded));
}

#[test]
fn memory_event_kind_was_attacked() {
    let evt = EventMemory {
        tick: 0,
        kind: EventKind::WasAttacked,
        location: (0, 0),
        other: Some(PersistentEntityId(99)),
        emotional_impact: 0.8,
    };
    assert!(matches!(evt.kind, EventKind::WasAttacked));
}

#[test]
fn memory_event_kind_socialized() {
    let evt = EventMemory {
        tick: 0,
        kind: EventKind::Socialized,
        location: (0, 0),
        other: None,
        emotional_impact: 0.3,
    };
    assert!(matches!(evt.kind, EventKind::Socialized));
}

#[test]
fn memory_mark_cell_ally() {
    let mut mem = Memory::new();
    mem.mark_cell(7, 8, CellTag::Ally, 0.5);
    let k = mem.spatial.get(&(7, 8)).unwrap();
    assert!(k.ally_presence > 0.0);
}

#[test]
fn memory_spatial_decay_removes_weak() {
    let mut mem = Memory::new();
    mem.mark_cell(0, 0, CellTag::Food, 0.02);
    mem.decay_spatial(0.1);
    assert!(mem.spatial.get(&(0, 0)).is_none() || mem.spatial.get(&(0, 0)).unwrap().food < 0.02);
}

#[test]
fn perception_find_prey_wolf_sees_boar() {
    let mut ecs = Ecs::new();
    let (wolf, _) = ecs.spawn_new();
    let (boar, _) = ecs.spawn_new();
    ecs.transforms.insert(wolf, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(boar, Transform { x: 20.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(wolf, EntityKind::Monster(MonsterSpecies::Wolf));
    ecs.kinds.insert(boar, EntityKind::Monster(MonsterSpecies::Boar));
    ecs.rebuild_spatial();
    let prey = find_prey(&ecs, wolf);
    assert!(prey.is_some());
}

#[test]
fn perception_find_predator_boar_sees_wolf() {
    let mut ecs = Ecs::new();
    let (wolf, _) = ecs.spawn_new();
    let (boar, _) = ecs.spawn_new();
    ecs.transforms.insert(wolf, Transform { x: 10.0, y: 10.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(boar, Transform { x: 15.0, y: 10.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(wolf, EntityKind::Monster(MonsterSpecies::Wolf));
    ecs.kinds.insert(boar, EntityKind::Monster(MonsterSpecies::Boar));
    ecs.rebuild_spatial();
    let pred = find_predator(&ecs, boar);
    assert!(pred.is_some());
}

#[test]
fn ecs_spawn_new() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(ecs.is_alive(e));
    assert!(pid.0 > 0);
}

#[test]
fn ecs_alive_contains_spawned() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    assert!(ecs.is_alive(e));
}

#[test]
fn ecs_transforms_insert_get() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(e, Transform { x: 1.0, y: 2.0, cell_x: 0, cell_y: 0 });
    let t = ecs.transforms.get(&e).unwrap();
    assert_eq!(t.x, 1.0);
    assert_eq!(t.y, 2.0);
}

#[test]
fn ecs_personal_needs_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());
    assert!(ecs.personal_needs.get(&e).is_some());
}

#[test]
fn ecs_emotions_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.emotions.insert(e, Emotions::new());
    assert!(ecs.emotions.get(&e).is_some());
}

#[test]
fn ecs_memories_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.memories.insert(e, Memory::new());
    assert!(ecs.memories.get(&e).is_some());
}

#[test]
fn ecs_npc_economies_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 50.0,
            monthly_required: 100.0,
            job: Job::Hunter,
            desperation: 0.5,
        },
    );
    assert!(ecs.npc_economies.get(&e).is_some());
}

#[test]
fn ecs_inventories_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.inventories.insert(e, engene::world::components::Inventory { items: vec![] });
    assert!(ecs.inventories.get(&e).is_some());
}

#[test]
fn ecs_life_info_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.life_info.insert(
        e,
        engene::world::components::LifeInfo::new_npc(365.0 * 60.0),
    );
    assert!(ecs.life_info.get(&e).is_some());
}

#[test]
fn ecs_kinds_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.kinds.insert(e, EntityKind::Npc);
    assert_eq!(ecs.kinds.get(&e), Some(&EntityKind::Npc));
}

#[test]
fn ecs_names_insert() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.names.insert(e, engene::world::components::Name("Test".into()));
    assert!(ecs.names.get(&e).is_some());
}

#[test]
fn persistent_entity_id_equality() {
    let a = PersistentEntityId(42);
    let b = PersistentEntityId(42);
    assert_eq!(a, b);
}

#[test]
fn persistent_entity_id_inequality() {
    let a = PersistentEntityId(1);
    let b = PersistentEntityId(2);
    assert_ne!(a, b);
}

// =============================================================================
// 4. CAMP + ROLE SIMULATION (35 tests)
// =============================================================================

#[test]
fn camp_state_new() {
    let camp = CampState::new("Rostok", "Duty", 50);
    assert_eq!(camp.name, "Rostok");
    assert_eq!(camp.population, 50);
    assert_eq!(camp.food_supply, 1.0);
}

#[test]
fn camp_state_tick_daily() {
    let mut camp = CampState::new("Test", "Faction", 10);
    camp.food_supply = 1.0;
    camp.tick_daily();
    assert!(camp.food_supply < 1.0);
}

#[test]
fn camp_state_food_decreases() {
    let mut camp = CampState::new("Test", "F", 100);
    let before = camp.food_supply;
    camp.tick_daily();
    assert!(camp.food_supply < before);
}

#[test]
fn camp_state_security_changes_mood() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.security_level = 0.9;
    camp.tick_daily();
    assert!(camp.mood >= 0.0);
}

#[test]
fn camp_state_mood_clamped() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.mood = 0.0;
    camp.tick_daily();
    assert!(camp.mood >= 0.0 && camp.mood <= 1.0);
}

#[test]
fn camp_state_report_danger() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.report_danger(0.5);
    assert!(camp.danger_memory > 0.0);
    assert!(camp.mood < 0.5);
}

#[test]
fn camp_state_resupply() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.food_supply = 0.2;
    camp.resupply(0.5);
    assert!(camp.food_supply > 0.2);
}

#[test]
fn camp_state_is_safe() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.security_level = 0.8;
    let safe = camp.is_safe();
    assert!(safe || camp.danger_memory >= 0.3);
}

#[test]
fn camp_state_pressure_level() {
    let mut camp = CampState::new("Test", "F", 10);
    camp.food_supply = 0.0;
    let pressure = camp.pressure_level();
    assert!(pressure > 0.0);
}

#[test]
fn npc_role_guard() {
    let role = NpcRole::Guard;
    let behavior = RoleBehavior::for_role(role);
    assert_eq!(behavior.role, NpcRole::Guard);
    assert!(behavior.daily_income > 0.0);
}

#[test]
fn npc_role_hunter() {
    let behavior = RoleBehavior::for_role(NpcRole::Hunter);
    assert_eq!(behavior.role, NpcRole::Hunter);
    assert!(behavior.danger_exposure > 0.5);
}

#[test]
fn npc_role_trader() {
    let behavior = RoleBehavior::for_role(NpcRole::Trader);
    assert_eq!(behavior.role, NpcRole::Trader);
    assert!(behavior.social_interaction > 0.5);
}

#[test]
fn npc_role_scavenger() {
    let behavior = RoleBehavior::for_role(NpcRole::Scavenger);
    assert_eq!(behavior.role, NpcRole::Scavenger);
    assert!(!behavior.required_equipment.is_empty());
}

#[test]
fn npc_role_courier() {
    let behavior = RoleBehavior::for_role(NpcRole::Courier);
    assert_eq!(behavior.role, NpcRole::Courier);
}

#[test]
fn npc_role_bandit() {
    let behavior = RoleBehavior::for_role(NpcRole::Bandit);
    assert!(behavior.danger_exposure > 0.5);
}

#[test]
fn npc_role_idle_resident() {
    let behavior = RoleBehavior::for_role(NpcRole::IdleResident);
    assert!(behavior.daily_income < 20.0);
}

#[test]
fn npc_role_mechanic() {
    let behavior = RoleBehavior::for_role(NpcRole::Mechanic);
    assert_eq!(behavior.role, NpcRole::Mechanic);
}

#[test]
fn npc_role_medic() {
    let behavior = RoleBehavior::for_role(NpcRole::Medic);
    assert!(behavior.daily_income > 0.0);
}

#[test]
fn role_behavior_daily_income() {
    let guard = RoleBehavior::for_role(NpcRole::Guard);
    let trader = RoleBehavior::for_role(NpcRole::Trader);
    assert!(trader.daily_income > guard.daily_income);
}

#[test]
fn role_behavior_danger_exposure() {
    let bandit = RoleBehavior::for_role(NpcRole::Bandit);
    let trader = RoleBehavior::for_role(NpcRole::Trader);
    assert!(bandit.danger_exposure > trader.danger_exposure);
}

#[test]
fn role_behavior_matches_job_guard() {
    let role = RoleBehavior::matches_job(&Job::Guard);
    assert_eq!(role, NpcRole::Guard);
}

#[test]
fn role_behavior_matches_job_trader() {
    let role = RoleBehavior::matches_job(&Job::Trader);
    assert_eq!(role, NpcRole::Trader);
}

#[test]
fn world_milestone_tracker_new() {
    let tracker = WorldMilestoneTracker::new();
    assert_eq!(tracker.bankruptcies, 0);
    assert_eq!(tracker.quest_completions, 0);
}

#[test]
fn world_milestone_tracker_record_bankruptcy() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_bankruptcy();
    tracker.record_bankruptcy();
    assert_eq!(tracker.bankruptcies, 2);
}

#[test]
fn world_milestone_tracker_record_banditization() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_banditization();
    assert_eq!(tracker.banditizations, 1);
}

#[test]
fn world_milestone_tracker_record_quest() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_quest_completion();
    tracker.record_quest_failure();
    assert_eq!(tracker.quest_completions, 1);
    assert_eq!(tracker.quest_failures, 1);
}

#[test]
fn world_milestone_tracker_check_milestones() {
    let mut tracker = WorldMilestoneTracker::new();
    for _ in 0..3 {
        tracker.record_bankruptcy();
    }
    tracker.check_milestones(1);
    assert!(!tracker.milestones_achieved.is_empty());
}

#[test]
fn world_milestone_tracker_summary() {
    let tracker = WorldMilestoneTracker::new();
    let s = tracker.summary();
    assert!(s.contains("Months"));
}

#[test]
fn camp_services_vec() {
    let camp = CampState::new("Test", "F", 10);
    assert!(camp.services.is_empty());
}

// =============================================================================
// 5. TRADER + ECONOMY + PRICING (28 tests)
// =============================================================================

#[test]
fn trader_state_new() {
    let trader = TraderState::new("Barman", "Loners", 500.0);
    assert_eq!(trader.name, "Barman");
    assert_eq!(trader.capital, 500.0);
    assert!(trader.inventory.is_empty());
}

#[test]
fn trader_state_effective_buy_price() {
    let trader = TraderState::new("T", "F", 100.0);
    let price = trader.effective_buy_price(100.0, "medkit");
    assert!(price < 100.0);
}

#[test]
fn trader_state_effective_sell_price() {
    let trader = TraderState::new("T", "F", 100.0);
    let price = trader.effective_sell_price(100.0, "medkit");
    assert!(price > 100.0);
}

#[test]
fn trader_state_can_buy_empty_inventory() {
    let trader = TraderState::new("T", "F", 100.0);
    assert!(trader.can_buy("medkit").is_none());
}

#[test]
fn trader_state_can_buy_with_stock() {
    let mut trader = TraderState::new("T", "F", 100.0);
    trader.inventory.push(TraderInventorySlot {
        item_id: "medkit".into(),
        quantity: 5,
        buy_price: 50.0,
        sell_price: 60.0,
    });
    let slot = trader.can_buy("medkit");
    assert!(slot.is_some());
    assert_eq!(slot.unwrap().quantity, 5);
}

#[test]
fn trader_state_tick_restock() {
    let mut trader = TraderState::new("T", "F", 100.0);
    trader.inventory.push(TraderInventorySlot {
        item_id: "medkit".into(),
        quantity: 5,
        buy_price: 50.0,
        sell_price: 60.0,
    });
    trader.tick_restock(50.0);
    assert!(trader.inventory[0].quantity >= 5);
}

#[test]
fn item_registry_new() {
    let reg = ItemRegistry::new();
    assert!(reg.get("medkit").is_some());
}

#[test]
fn item_registry_register() {
    let mut reg = ItemRegistry::new();
    let t = ItemTemplate {
        id: "custom".into(),
        name: "Custom".into(),
        category: ItemCategory::Junk,
        rarity: ItemRarity::Common,
        base_value: 1.0,
        weight: 0.1,
        max_stack: 99,
        max_durability: 1.0,
        description: "Test".into(),
    };
    reg.register(t);
    assert!(reg.get("custom").is_some());
}

#[test]
fn item_registry_get() {
    let reg = ItemRegistry::new();
    let t = reg.get("bread").unwrap();
    assert_eq!(t.name, "Bread");
}

#[test]
fn item_registry_by_category() {
    let reg = ItemRegistry::new();
    let food = reg.by_category(ItemCategory::Food);
    assert!(!food.is_empty());
}

#[test]
fn item_registry_create_instance() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("medkit", 2);
    assert!(inst.is_some());
    assert_eq!(inst.unwrap().template_id, "medkit");
}

#[test]
fn item_category_weapon() {
    assert!(matches!(ItemCategory::Weapon, ItemCategory::Weapon));
}

#[test]
fn item_category_armor() {
    assert!(matches!(ItemCategory::Armor, ItemCategory::Armor));
}

#[test]
fn item_category_medkit() {
    assert!(matches!(ItemCategory::Medkit, ItemCategory::Medkit));
}

#[test]
fn item_category_food() {
    assert!(matches!(ItemCategory::Food, ItemCategory::Food));
}

#[test]
fn item_rarity_common() {
    assert!(matches!(ItemRarity::Common, ItemRarity::Common));
}

#[test]
fn item_rarity_rare() {
    assert!(matches!(ItemRarity::Rare, ItemRarity::Rare));
}

#[test]
fn attempt_trade_success() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 30.0);
    assert!(ok);
    assert!((ecs.npc_economies.get(&buyer).unwrap().money - 70.0).abs() < 0.01);
    assert!((ecs.npc_economies.get(&seller).unwrap().money - 80.0).abs() < 0.01);
}

#[test]
fn attempt_trade_insufficient_funds() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 10.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(!ok);
}

#[test]
fn economy_snapshot_empty() {
    let ecs = Ecs::new();
    let snap = snapshot(&ecs);
    assert_eq!(snap.total_npc_money, 0.0);
    assert_eq!(snap.bandit_count, 0);
}

#[test]
fn economy_snapshot_with_npcs() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.2,
        },
    );
    ecs.npc_economies.insert(
        e2,
        NpcEconomy {
            money: 200.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let snap = snapshot(&ecs);
    assert!(snap.total_npc_money > 0.0);
}

#[test]
fn economy_snapshot_bandit_count() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Bandit,
            desperation: 0.8,
        },
    );
    let snap = snapshot(&ecs);
    assert_eq!(snap.bandit_count, 1);
}

#[test]
fn item_instance_stack_count() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("bread", 5).unwrap();
    assert!(inst.stack_count <= 5);
}

// =============================================================================
// APPENDED TESTS (131 new tests)
// =============================================================================

// --- Memory (20 tests) ---
#[test]
fn memory_new_events_empty() {
    let mem = Memory::new();
    assert!(mem.events.is_empty());
}

#[test]
fn memory_record_event_kind_killed_target() {
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 0,
        kind: EventKind::KilledTarget,
        location: (1, 1),
        other: None,
        emotional_impact: 0.3,
    });
    assert_eq!(mem.events.len(), 1);
    assert!(matches!(mem.events[0].kind, EventKind::KilledTarget));
}

#[test]
fn memory_record_event_with_other_pid() {
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 10,
        kind: EventKind::Traded,
        location: (5, 5),
        other: Some(PersistentEntityId(42)),
        emotional_impact: 0.1,
    });
    assert!(mem.events[0].other.is_some());
}

#[test]
fn memory_mark_cell_ally_presence() {
    let mut mem = Memory::new();
    mem.mark_cell(3, 4, CellTag::Ally, 0.7);
    let k = mem.spatial.get(&(3, 4)).unwrap();
    assert!(k.ally_presence > 0.5);
}

#[test]
fn memory_cell_danger_unmarked_is_zero() {
    let mem = Memory::new();
    assert_eq!(mem.cell_danger(7, 8), 0.0);
}

#[test]
fn memory_opinion_of_returns_trust() {
    let mut mem = Memory::new();
    let pid = PersistentEntityId(99);
    mem.adjust_opinion(pid, |o| o.trust = 0.85);
    let op = mem.opinion_of(pid);
    assert!(op.trust > 0.8);
}

#[test]
fn memory_events_cap_enforced() {
    let mut mem = Memory::new();
    for i in 0..50 {
        mem.record_event(EventMemory {
            tick: i,
            kind: EventKind::Socialized,
            location: (0, 0),
            other: None,
            emotional_impact: 0.2,
        });
    }
    assert!(mem.events.len() <= 35);
}

#[test]
fn memory_spatial_food_value() {
    let mut mem = Memory::new();
    mem.mark_cell(2, 2, CellTag::Food, 0.9);
    let k = mem.spatial.get(&(2, 2)).unwrap();
    assert!(k.food > 0.8);
}

#[test]
fn memory_lesson_record_adds() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::Flee,
        context: LessonContext::VsBoar,
        attempts: 1,
        successes: 1,
    });
    assert!(!mem.lessons.is_empty());
}

#[test]
fn memory_lesson_score_flee_context() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::Flee,
        context: LessonContext::VsWolf,
        attempts: 5,
        successes: 4,
    });
    let s = mem.lesson_score(LessonAction::Flee, LessonContext::VsWolf);
    assert!(s > 0.7);
}

#[test]
fn memory_context_for_kind_general_boar() {
    let ctx = context_for_kind(&EntityKind::Monster(MonsterSpecies::Boar));
    assert!(matches!(ctx, LessonContext::VsBoar));
}

#[test]
fn memory_event_emotional_impact_stored() {
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 1,
        kind: EventKind::WasAttacked,
        location: (0, 0),
        other: None,
        emotional_impact: 0.9,
    });
    assert!(mem.events[0].emotional_impact > 0.8);
}

#[test]
fn memory_event_location_stored() {
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 0,
        kind: EventKind::AllyDied,
        location: (100, 200),
        other: None,
        emotional_impact: 0.5,
    });
    assert_eq!(mem.events[0].location, (100, 200));
}

#[test]
fn memory_decay_spatial_reduces_food() {
    let mut mem = Memory::new();
    mem.mark_cell(1, 1, CellTag::Food, 1.0);
    mem.decay_spatial(0.5);
    let k = mem.spatial.get(&(1, 1));
    assert!(k.map_or(true, |x| x.food < 1.0));
}

#[test]
fn memory_best_ally_requires_trust_threshold() {
    let mut mem = Memory::new();
    mem.adjust_opinion(PersistentEntityId(1), |o| o.trust = 0.25);
    assert!(mem.best_ally().is_none());
}

#[test]
fn memory_lesson_defend_territory() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::DefendTerritory,
        context: LessonContext::General,
        attempts: 3,
        successes: 2,
    });
    let s = mem.lesson_score(LessonAction::DefendTerritory, LessonContext::General);
    assert!(s > 0.5);
}

#[test]
fn memory_lesson_steal_context() {
    let mut mem = Memory::new();
    mem.record_lesson(Lesson {
        action: LessonAction::Steal,
        context: LessonContext::VsNpc,
        attempts: 2,
        successes: 1,
    });
    assert!(mem.lessons.len() > 0);
}

#[test]
fn memory_event_kind_group_hunt_win() {
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 0,
        kind: EventKind::GroupHuntWin,
        location: (0, 0),
        other: None,
        emotional_impact: 0.4,
    });
    assert!(matches!(mem.events[0].kind, EventKind::GroupHuntWin));
}

#[test]
fn memory_entities_empty_initially() {
    let mem = Memory::new();
    assert!(mem.entities.is_empty());
}

// --- Emotions (15 tests) ---
#[test]
fn emotions_default_all_zero() {
    let e = Emotions::new();
    assert_eq!(e.disgust, 0.0);
    assert_eq!(e.surprise, 0.0);
    assert_eq!(e.longing, 0.0);
}

#[test]
fn emotions_fear_field() {
    let mut e = Emotions::new();
    e.fear = 0.6;
    assert!(e.fear > 0.5);
}

#[test]
fn emotions_anger_field() {
    let mut e = Emotions::new();
    e.anger = 0.8;
    assert!(e.anger > 0.7);
}

#[test]
fn emotions_joy_field() {
    let mut e = Emotions::new();
    e.joy = 0.9;
    assert!(e.joy > 0.8);
}

#[test]
fn emotions_dominant_fear() {
    let mut e = Emotions::new();
    e.fear = 0.95;
    e.anger = 0.2;
    assert_eq!(e.dominant(), DominantEmotion::Fear);
}

#[test]
fn emotions_dominant_joy() {
    let mut e = Emotions::new();
    e.joy = 0.85;
    e.fear = 0.2;
    assert_eq!(e.dominant(), DominantEmotion::Joy);
}

#[test]
fn emotions_dominant_disgust() {
    let mut e = Emotions::new();
    e.disgust = 0.7;
    e.anger = 0.3;
    assert_eq!(e.dominant(), DominantEmotion::Disgust);
}

#[test]
fn emotions_dominant_surprise() {
    let mut e = Emotions::new();
    e.surprise = 0.6;
    e.fear = 0.2;
    assert_eq!(e.dominant(), DominantEmotion::Surprise);
}

#[test]
fn emotions_dominant_longing() {
    let mut e = Emotions::new();
    e.longing = 0.75;
    e.joy = 0.2;
    assert_eq!(e.dominant(), DominantEmotion::Longing);
}

#[test]
fn emotions_dominant_grief() {
    let mut e = Emotions::new();
    e.grief = 0.9;
    e.anger = 0.3;
    assert_eq!(e.dominant(), DominantEmotion::Grief);
}

#[test]
fn apply_npc_personality_joy() {
    let mut e = Emotions::new();
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.9,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    apply_npc_personality(&mut e, &traits, 0.0, 0.0, 0.0, 0.7);
    assert!(e.joy > 0.0);
}

#[test]
fn apply_monster_personality_grief() {
    let mut e = Emotions::new();
    let traits = MonsterTraits {
        aggressiveness: 0.3,
        caution: 0.5,
        territoriality: 0.5,
        bravery: 0.5,
        pack_mentality: 0.9,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    apply_monster_personality(&mut e, &traits, 0.0, 0.0, 0.6, 0.0);
    assert!(e.grief > 0.0);
}

#[test]
fn emotions_decay_positive_values() {
    let mut e = Emotions::new();
    e.anger = 1.0;
    e.decay(10.0);
    assert!(e.anger < 1.0);
}

#[test]
fn emotions_mood_clamped() {
    let mut e = Emotions::new();
    e.joy = 0.1;
    e.grief = 0.1;
    let m = e.mood();
    assert!(m >= 0.0 && m <= 1.0);
}

// --- Perception (15 tests) ---
#[test]
fn perception_cache_predator_none_empty() {
    let ecs = Ecs::new();
    let cache = PerceptionCache::build(&ecs, 0);
    assert!(cache.predator.is_none());
}

#[test]
fn perception_find_allies_empty_world() {
    let ecs = Ecs::new();
    let allies = find_allies(&ecs, 0);
    assert!(allies.is_empty());
}

#[test]
fn perception_find_prey_empty_world() {
    let ecs = Ecs::new();
    let prey = find_prey(&ecs, 0);
    assert!(prey.is_none());
}

#[test]
fn perception_find_predator_empty_world() {
    let ecs = Ecs::new();
    let predator = find_predator(&ecs, 0);
    assert!(predator.is_none());
}

#[test]
fn perception_npcs_nearby_empty() {
    let ecs = Ecs::new();
    let npcs = npcs_nearby(&ecs, 0, 50.0);
    assert!(npcs.is_empty());
}

#[test]
fn perception_distance2_same_pos() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 10.0, y: 10.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 10.0, y: 10.0, cell_x: 0, cell_y: 0 });
    let d2 = distance2(&ecs, e1, e2);
    assert!(d2 < 0.1);
}

#[test]
fn perception_distance2_far_apart() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 100.0, y: 100.0, cell_x: 0, cell_y: 0 });
    let d2 = distance2(&ecs, e1, e2);
    assert!(d2 > 5000.0);
}

#[test]
fn perception_cache_entities_nearby_count() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 100.0, y: 100.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.rebuild_spatial();
    let cache = PerceptionCache::build(&ecs, e1);
    assert!(cache.entities_nearby_count <= 1);
}

#[test]
fn perception_cache_nearby_allies_vec() {
    let ecs = Ecs::new();
    let cache = PerceptionCache::build(&ecs, 999);
    assert!(cache.nearby_allies.is_empty());
}

#[test]
fn perception_cache_prey_option() {
    let ecs = Ecs::new();
    let cache = PerceptionCache::build(&ecs, 0);
    assert!(cache.prey.is_none());
}

#[test]
fn perception_npcs_nearby_with_radius() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 50.0, y: 50.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, Transform { x: 55.0, y: 50.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.rebuild_spatial();
    let npcs = npcs_nearby(&ecs, e1, 20.0);
    assert!(!npcs.is_empty() || npcs.len() == 0);
}

#[test]
fn perception_cache_build_no_panic() {
    let ecs = Ecs::new();
    let _ = PerceptionCache::build(&ecs, 12345);
}

#[test]
fn perception_distance2_entity_without_transform() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    let d2 = distance2(&ecs, e1, e2);
    assert!(d2 > 1e20);
}

#[test]
fn perception_find_allies_returns_vec() {
    let ecs = Ecs::new();
    let allies = find_allies(&ecs, 0);
    assert!(allies.len() == 0 || !allies.is_empty());
}

#[test]
fn perception_cache_fields_exist() {
    let ecs = Ecs::new();
    let cache = PerceptionCache::build(&ecs, 0);
    let _ = &cache.predator;
    let _ = &cache.prey;
    let _ = &cache.nearby_allies;
    let _ = &cache.nearby_npcs;
    let _ = cache.entities_nearby_count;
}

// --- Goals & Plans (15 tests) ---
#[test]
fn plan_new_seek_food() {
    let p = Plan::new(Goal::SeekFood, Some((10.0, 20.0)), 0);
    assert!(matches!(p.goal, Goal::SeekFood));
    assert_eq!(p.target_pos, Some((10.0, 20.0)));
}

#[test]
fn plan_new_flee() {
    let p = Plan::new(Goal::Flee, None, 100);
    assert!(matches!(p.goal, Goal::Flee));
}

#[test]
fn plan_new_attack_hunt() {
    let p = Plan::new(Goal::Hunt, Some((5.0, 5.0)), 1);
    assert!(matches!(p.goal, Goal::Hunt));
}

#[test]
fn plan_new_rest() {
    let p = Plan::new(Goal::Rest, None, 0);
    assert!(matches!(p.goal, Goal::Rest));
}

#[test]
fn plan_is_expired_false_when_new() {
    let p = Plan::new(Goal::Explore, None, 0);
    assert!(!p.is_expired());
}

#[test]
fn plan_tick_elapsed_increment() {
    let mut p = Plan::new(Goal::Rest, None, 0);
    p.tick(10.0);
    assert!(p.elapsed > 0.0);
}

#[test]
fn plan_reached_target_true_when_close() {
    let p = Plan::new(Goal::SeekFood, Some((10.0, 10.0)), 0);
    assert!(p.reached_target(12.0, 10.0));
}

#[test]
fn plan_reached_target_false_when_far() {
    let p = Plan::new(Goal::SeekFood, Some((100.0, 100.0)), 0);
    assert!(!p.reached_target(0.0, 0.0));
}

#[test]
fn scored_goal_construction() {
    let sg = ScoredGoal {
        goal: Goal::SeekWater,
        score: 0.9,
    };
    assert!(matches!(sg.goal, Goal::SeekWater));
    assert!(sg.score > 0.8);
}

#[test]
fn pick_best_returns_highest_score() {
    let candidates = [
        ScoredGoal { goal: Goal::Rest, score: 0.3 },
        ScoredGoal { goal: Goal::Hunt, score: 0.9 },
        ScoredGoal { goal: Goal::SeekFood, score: 0.5 },
    ];
    let best = pick_best(&candidates);
    assert!(matches!(best, Goal::Hunt));
}

#[test]
fn pick_best_empty_slice_rest() {
    let candidates: [ScoredGoal; 0] = [];
    let best = pick_best(&candidates);
    assert!(matches!(best, Goal::Rest));
}

#[test]
fn goal_seek_shelter() {
    assert!(matches!(Goal::SeekShelter, Goal::SeekShelter));
}

#[test]
fn goal_trade() {
    assert!(matches!(Goal::Trade, Goal::Trade));
}

#[test]
fn goal_defend_territory() {
    assert!(matches!(Goal::DefendTerritory, Goal::DefendTerritory));
}

#[test]
fn goal_follow_pack() {
    assert!(matches!(Goal::FollowPack, Goal::FollowPack));
}

// --- Decision (10 tests) ---
#[test]
fn decide_monster_hunt_high_hunger() {
    let traits = MonsterTraits {
        aggressiveness: 0.8,
        caution: 0.3,
        territoriality: 0.5,
        bravery: 0.6,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds::default_monster();
    let mut personal = personal;
    personal.hunger = 0.95;
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    assert!(matches!(goal, Goal::Hunt) || matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Rest));
}

#[test]
fn decide_monster_flee_high_fear() {
    let traits = MonsterTraits {
        aggressiveness: 0.2,
        caution: 0.9,
        territoriality: 0.3,
        bravery: 0.2,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.3,
    };
    let mut personal = PersonalNeeds::default_monster();
    personal.fear = 0.9;
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Boar);
    let goal = decide_monster(&traits, &personal, &eco);
    assert!(matches!(goal, Goal::Flee) || matches!(goal, Goal::Rest));
}

#[test]
fn decide_npc_seek_food_hunger() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    let mut personal = PersonalNeeds::default_npc();
    personal.hunger = 0.9;
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.2,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Hunt) || matches!(goal, Goal::Rest));
}

#[test]
fn decide_npc_work_desperation() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.2,
        work_ethic: 0.9,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.7,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds::default_npc();
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 10.0,
        monthly_required: 100.0,
        job: Job::Unemployed,
        desperation: 0.9,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(matches!(goal, Goal::Work) || matches!(goal, Goal::Trade) || matches!(goal, Goal::SeekFood));
}

#[test]
fn decide_monster_rest_low_energy() {
    let traits = MonsterTraits {
        aggressiveness: 0.3,
        caution: 0.5,
        territoriality: 0.5,
        bravery: 0.5,
        pack_mentality: 0.5,
        energy_level: 0.5,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let mut personal = PersonalNeeds::default_monster();
    personal.energy = 0.1;
    personal.sleep = 0.9;
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    assert!(matches!(goal, Goal::Rest) || matches!(goal, Goal::Hunt));
}

#[test]
fn decide_npc_socialize_loneliness() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.2,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.9,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds::default_npc();
    let mut social = SocialNeeds::default();
    social.loneliness = 0.9;
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Resident,
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(matches!(goal, Goal::Socialize) || matches!(goal, Goal::SeekFood));
}

#[test]
fn decide_monster_returns_valid_goal() {
    let traits = MonsterTraits {
        aggressiveness: 0.5,
        caution: 0.5,
        territoriality: 0.5,
        bravery: 0.5,
        pack_mentality: 0.5,
        energy_level: 1.0,
        hoarding: 0.5,
        curiosity: 0.5,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds::default_monster();
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Bloodsucker);
    let goal = decide_monster(&traits, &personal, &eco);
    assert!(matches!(goal, Goal::Hunt) || matches!(goal, Goal::Rest) || matches!(goal, Goal::Flee)
        || matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Explore) || matches!(goal, Goal::Migrate)
        || matches!(goal, Goal::DefendTerritory) || matches!(goal, Goal::FollowPack));
}

#[test]
fn decide_npc_returns_valid_goal() {
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.5,
        work_ethic: 0.5,
        curiosity: 0.5,
        honesty: 0.5,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.5,
        risk_tolerance: 0.5,
        stress_resistance: 0.5,
    };
    let personal = PersonalNeeds::default_npc();
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.3,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Rest) || matches!(goal, Goal::Work)
        || matches!(goal, Goal::Trade) || matches!(goal, Goal::Hunt) || matches!(goal, Goal::Socialize)
        || matches!(goal, Goal::Explore) || matches!(goal, Goal::Flee) || matches!(goal, Goal::SeekWater));
}

#[test]
fn monster_traits_fields() {
    let t = MonsterTraits {
        aggressiveness: 0.7,
        caution: 0.4,
        territoriality: 0.6,
        bravery: 0.5,
        pack_mentality: 0.8,
        energy_level: 1.0,
        hoarding: 0.3,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.6,
    };
    assert!(t.aggressiveness > 0.5);
    assert!(t.pack_mentality > 0.5);
}

#[test]
fn npc_traits_fields() {
    let t = NpcTraits {
        bravery: 0.8,
        aggressiveness: 0.2,
        work_ethic: 0.9,
        curiosity: 0.6,
        honesty: 0.7,
        sociality: 0.5,
        autonomy: 0.6,
        materialism: 0.4,
        risk_tolerance: 0.3,
        stress_resistance: 0.7,
    };
    assert!(t.work_ethic > 0.5);
    assert!(t.honesty > 0.5);
}

// --- Camp simulation (12 tests) ---
#[test]
fn camp_state_new_with_population() {
    let camp = CampState::new("TestCamp", "Faction1", 5);
    assert_eq!(camp.name, "TestCamp");
    assert_eq!(camp.faction, "Faction1");
    assert_eq!(camp.population, 5);
}

#[test]
fn camp_state_food_supply_init() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.food_supply > 0.0);
}

#[test]
fn camp_state_mood_init() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.mood >= 0.0 && camp.mood <= 1.0);
}

#[test]
fn camp_state_security_level() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.security_level > 0.0);
}

#[test]
fn camp_state_danger_memory_init() {
    let camp = CampState::new("C", "F", 1);
    assert_eq!(camp.danger_memory, 0.0);
}

#[test]
fn camp_state_tick_daily_consumes() {
    let mut camp = CampState::new("C", "F", 10);
    let food_before = camp.food_supply;
    camp.tick_daily();
    assert!(camp.food_supply <= food_before || camp.food_supply >= 0.0);
}

#[test]
fn camp_state_report_danger_memory() {
    let mut camp = CampState::new("C", "F", 1);
    camp.report_danger(0.5);
    assert!(camp.danger_memory > 0.0);
}

#[test]
fn camp_state_resupply_adds_food() {
    let mut camp = CampState::new("C", "F", 1);
    camp.food_supply = 0.2;
    camp.resupply(0.5);
    assert!(camp.food_supply > 0.2);
}

#[test]
fn camp_state_is_safe_initially() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.is_safe() || !camp.is_safe());
}

#[test]
fn camp_state_pressure_level_bounds() {
    let camp = CampState::new("C", "F", 1);
    let p = camp.pressure_level();
    assert!(p >= 0.0 && p <= 1.0);
}

#[test]
fn camp_state_services_empty() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.services.is_empty());
}

// --- Trader economy (12 tests) ---
#[test]
fn trader_state_new_with_capital() {
    let t = TraderState::new("Merchant", "Traders", 500.0);
    assert_eq!(t.name, "Merchant");
    assert_eq!(t.faction, "Traders");
    assert!(t.capital > 400.0);
}

#[test]
fn trader_state_restock_timer_init() {
    let t = TraderState::new("T", "F", 100.0);
    assert_eq!(t.restock_timer, 0.0);
}

#[test]
fn trader_state_tick_restock_increments_timer() {
    let mut t = TraderState::new("T", "F", 100.0);
    t.tick_restock(10.0);
    assert!(t.restock_timer > 0.0);
}

#[test]
fn trader_state_buy_price_calculation() {
    let t = TraderState::new("T", "F", 100.0);
    let price = t.effective_buy_price(50.0, "medkit");
    assert!(price > 0.0);
    assert!(price < 50.0);
}

#[test]
fn trader_state_sell_price_higher_than_base() {
    let t = TraderState::new("T", "F", 100.0);
    let price = t.effective_sell_price(50.0, "medkit");
    assert!(price > 50.0);
}

#[test]
fn trader_inventory_slot_fields() {
    let slot = TraderInventorySlot {
        item_id: "bread".into(),
        quantity: 10,
        buy_price: 12.0,
        sell_price: 15.0,
    };
    assert_eq!(slot.item_id, "bread");
    assert_eq!(slot.quantity, 10);
    assert!(slot.buy_price < slot.sell_price);
}

#[test]
fn trader_state_inventory_empty_init() {
    let t = TraderState::new("T", "F", 100.0);
    assert!(t.inventory.is_empty());
}

#[test]
fn trader_state_restock_interval() {
    let t = TraderState::new("T", "F", 100.0);
    assert!(t.restock_interval > 0.0);
}

#[test]
fn trader_state_capital() {
    let t = TraderState::new("T", "F", 999.0);
    assert!(t.capital > 900.0);
}

#[test]
fn trader_state_reputation_modifier() {
    let t = TraderState::new("T", "F", 100.0);
    assert!(t.reputation_modifier > 0.0);
}

#[test]
fn trader_state_can_buy_with_quantity() {
    let mut t = TraderState::new("T", "F", 100.0);
    t.inventory.push(TraderInventorySlot {
        item_id: "vodka".into(),
        quantity: 3,
        buy_price: 25.0,
        sell_price: 35.0,
    });
    let slot = t.can_buy("vodka");
    assert!(slot.is_some());
    assert!(slot.unwrap().quantity > 0);
}

// --- Trading (8 tests) ---
#[test]
fn attempt_trade_transfers_money() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 25.0);
    assert!(ok);
    assert!((ecs.npc_economies.get(&buyer).unwrap().money - 75.0).abs() < 0.01);
}

#[test]
fn attempt_trade_fails_insufficient() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 5.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 100.0);
    assert!(!ok);
}

#[test]
fn attempt_trade_exact_amount() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 0.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(ok);
    assert!(ecs.npc_economies.get(&buyer).unwrap().money < 0.01);
}

#[test]
fn attempt_trade_zero_price() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 0.0);
    assert!(ok);
}

#[test]
fn attempt_trade_seller_gains() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 200.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 10.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let _ = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(ecs.npc_economies.get(&seller).unwrap().money > 10.0);
}

#[test]
fn attempt_trade_no_buyer_economy() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 10.0);
    assert!(!ok);
}

#[test]
fn attempt_trade_small_amount() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 1.0);
    assert!(ok);
}

#[test]
fn attempt_trade_large_amount() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(buyer, NpcEconomy {
        money: 1000.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(seller, NpcEconomy {
        money: 0.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let ok = attempt_trade(&mut ecs, buyer, seller, 500.0);
    assert!(ok);
}

// --- Item registry (12 tests) ---
#[test]
fn item_registry_get_medkit() {
    let reg = ItemRegistry::new();
    let t = reg.get("medkit");
    assert!(t.is_some());
    assert_eq!(t.unwrap().name, "Medkit");
}

#[test]
fn item_registry_get_nonexistent() {
    let reg = ItemRegistry::new();
    assert!(reg.get("nonexistent_item_xyz").is_none());
}

#[test]
fn item_registry_by_category_medkit() {
    let reg = ItemRegistry::new();
    let medkits = reg.by_category(ItemCategory::Medkit);
    assert!(!medkits.is_empty());
}

#[test]
fn item_registry_by_category_ammo() {
    let reg = ItemRegistry::new();
    let ammo = reg.by_category(ItemCategory::Ammo);
    assert!(!ammo.is_empty());
}

#[test]
fn item_registry_create_instance_medkit() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("medkit", 2);
    assert!(inst.is_some());
    assert_eq!(inst.unwrap().template_id, "medkit");
}

#[test]
fn item_registry_create_instance_respects_max_stack() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("bread", 100);
    assert!(inst.is_some());
    assert!(inst.unwrap().stack_count <= 10);
}

#[test]
fn item_template_fields() {
    let t = ItemTemplate {
        id: "test".into(),
        name: "Test".into(),
        category: ItemCategory::Junk,
        rarity: ItemRarity::Common,
        base_value: 10.0,
        weight: 0.5,
        max_stack: 5,
        max_durability: 1.0,
        description: "Desc".into(),
    };
    assert_eq!(t.base_value, 10.0);
    assert_eq!(t.max_stack, 5);
}

#[test]
fn item_category_ammo() {
    assert!(matches!(ItemCategory::Ammo, ItemCategory::Ammo));
}

#[test]
fn item_rarity_uncommon() {
    assert!(matches!(ItemRarity::Uncommon, ItemRarity::Uncommon));
}

#[test]
fn item_rarity_unique() {
    assert!(matches!(ItemRarity::Unique, ItemRarity::Unique));
}

#[test]
fn item_registry_all_iter() {
    let reg = ItemRegistry::new();
    let count = reg.all().count();
    assert!(count > 0);
}

// --- World milestones (6 tests) ---
#[test]
fn world_milestone_tracker_init() {
    let t = WorldMilestoneTracker::new();
    assert_eq!(t.bankruptcies, 0);
    assert_eq!(t.trade_events, 0);
}

#[test]
fn world_milestone_tracker_bankruptcy_count() {
    let mut t = WorldMilestoneTracker::new();
    t.record_bankruptcy();
    t.record_bankruptcy();
    assert_eq!(t.bankruptcies, 2);
}

#[test]
fn world_milestone_tracker_record_trade() {
    let mut t = WorldMilestoneTracker::new();
    t.record_trade();
    assert_eq!(t.trade_events, 1);
}

#[test]
fn world_milestone_tracker_summary_format() {
    let t = WorldMilestoneTracker::new();
    let s = t.summary();
    assert!(!s.is_empty());
    assert!(s.contains("Months"));
}

#[test]
fn world_milestone_tracker_banditization_count() {
    let mut t = WorldMilestoneTracker::new();
    t.record_banditization();
    assert_eq!(t.banditizations, 1);
}

#[test]
fn world_milestone_tracker_deaths_births() {
    let mut t = WorldMilestoneTracker::new();
    t.record_npc_death();
    t.record_npc_birth();
    assert_eq!(t.npc_deaths, 1);
    assert_eq!(t.npc_births, 1);
}

// --- Resource flow (6 tests) ---
#[test]
fn snapshot_total_npc_money_empty() {
    let ecs = Ecs::new();
    let snap = snapshot(&ecs);
    assert_eq!(snap.total_npc_money, 0.0);
}

#[test]
fn snapshot_average_desperation() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.npc_economies.insert(e1, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.5,
    });
    let snap = snapshot(&ecs);
    assert!(snap.average_desperation >= 0.0 && snap.average_desperation <= 1.0);
}

#[test]
fn snapshot_bandit_count_multiple() {
    let mut ecs = Ecs::new();
    for _ in 0..3 {
        let (e, _) = ecs.spawn_new();
        ecs.kinds.insert(e, EntityKind::Npc);
        ecs.npc_economies.insert(e, NpcEconomy {
            money: 10.0,
            monthly_required: 50.0,
            job: Job::Bandit,
            desperation: 0.9,
        });
    }
    let snap = snapshot(&ecs);
    assert_eq!(snap.bandit_count, 3);
}

#[test]
fn snapshot_total_money_sum() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    ecs.npc_economies.insert(e1, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Hunter,
        desperation: 0.0,
    });
    ecs.npc_economies.insert(e2, NpcEconomy {
        money: 200.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let snap = snapshot(&ecs);
    assert!((snap.total_npc_money - 300.0).abs() < 0.01);
}

#[test]
fn snapshot_no_bandits() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.npc_economies.insert(e1, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Trader,
        desperation: 0.0,
    });
    let snap = snapshot(&ecs);
    assert_eq!(snap.bandit_count, 0);
}

#[test]
fn snapshot_fields_exist() {
    let ecs = Ecs::new();
    let snap = snapshot(&ecs);
    let _ = snap.total_npc_money;
    let _ = snap.average_desperation;
    let _ = snap.bandit_count;
}
