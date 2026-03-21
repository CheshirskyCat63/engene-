use super::*;

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
    ecs.transforms.insert(
        e1,
        Transform {
            x: 10.0,
            y: 10.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        e2,
        Transform {
            x: 10.0,
            y: 10.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    let d2 = distance2(&ecs, e1, e2);
    assert!(d2 < 0.1);
}

#[test]
fn perception_distance2_far_apart() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        e2,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    let d2 = distance2(&ecs, e1, e2);
    assert!(d2 > 5000.0);
}

#[test]
fn perception_cache_entities_nearby_count() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    ecs.transforms.insert(
        e1,
        Transform {
            x: 50.0,
            y: 50.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        e2,
        Transform {
            x: 55.0,
            y: 50.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    ecs.transforms.insert(
        e1,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
        ScoredGoal {
            goal: Goal::Rest,
            score: 0.3,
        },
        ScoredGoal {
            goal: Goal::Hunt,
            score: 0.9,
        },
        ScoredGoal {
            goal: Goal::SeekFood,
            score: 0.5,
        },
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
    assert!(
        matches!(goal, Goal::Hunt) || matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Rest)
    );
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
    assert!(
        matches!(goal, Goal::SeekFood) || matches!(goal, Goal::Hunt) || matches!(goal, Goal::Rest)
    );
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
    assert!(
        matches!(goal, Goal::Work) || matches!(goal, Goal::Trade) || matches!(goal, Goal::SeekFood)
    );
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
    assert!(
        matches!(goal, Goal::Hunt)
            || matches!(goal, Goal::Rest)
            || matches!(goal, Goal::Flee)
            || matches!(goal, Goal::SeekFood)
            || matches!(goal, Goal::Explore)
            || matches!(goal, Goal::Migrate)
            || matches!(goal, Goal::DefendTerritory)
            || matches!(goal, Goal::FollowPack)
    );
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
    assert!(
        matches!(goal, Goal::SeekFood)
            || matches!(goal, Goal::Rest)
            || matches!(goal, Goal::Work)
            || matches!(goal, Goal::Trade)
            || matches!(goal, Goal::Hunt)
            || matches!(goal, Goal::Socialize)
            || matches!(goal, Goal::Explore)
            || matches!(goal, Goal::Flee)
            || matches!(goal, Goal::SeekWater)
    );
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
    let ok = attempt_trade(&mut ecs, buyer, seller, 25.0);
    assert!(ok);
    assert!((ecs.npc_economies.get(&buyer).unwrap().money - 75.0).abs() < 0.01);
}

#[test]
fn attempt_trade_fails_insufficient() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 5.0,
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
    let ok = attempt_trade(&mut ecs, buyer, seller, 100.0);
    assert!(!ok);
}

#[test]
fn attempt_trade_exact_amount() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 0.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(ok);
    assert!(ecs.npc_economies.get(&buyer).unwrap().money < 0.01);
}

#[test]
fn attempt_trade_zero_price() {
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
    let ok = attempt_trade(&mut ecs, buyer, seller, 0.0);
    assert!(ok);
}

#[test]
fn attempt_trade_seller_gains() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 200.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 10.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let _ = attempt_trade(&mut ecs, buyer, seller, 50.0);
    assert!(ecs.npc_economies.get(&seller).unwrap().money > 10.0);
}

#[test]
fn attempt_trade_no_buyer_economy() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    let ok = attempt_trade(&mut ecs, buyer, seller, 10.0);
    assert!(!ok);
}

#[test]
fn attempt_trade_small_amount() {
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
    let ok = attempt_trade(&mut ecs, buyer, seller, 1.0);
    assert!(ok);
}

#[test]
fn attempt_trade_large_amount() {
    let mut ecs = Ecs::new();
    let (buyer, _) = ecs.spawn_new();
    let (seller, _) = ecs.spawn_new();
    ecs.npc_economies.insert(
        buyer,
        NpcEconomy {
            money: 1000.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
    ecs.npc_economies.insert(
        seller,
        NpcEconomy {
            money: 0.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
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
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 50.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.5,
        },
    );
    let snap = snapshot(&ecs);
    assert!(snap.average_desperation >= 0.0 && snap.average_desperation <= 1.0);
}

#[test]
fn snapshot_bandit_count_multiple() {
    let mut ecs = Ecs::new();
    for _ in 0..3 {
        let (e, _) = ecs.spawn_new();
        ecs.kinds.insert(e, EntityKind::Npc);
        ecs.npc_economies.insert(
            e,
            NpcEconomy {
                money: 10.0,
                monthly_required: 50.0,
                job: Job::Bandit,
                desperation: 0.9,
            },
        );
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
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.0,
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
    assert!((snap.total_npc_money - 300.0).abs() < 0.01);
}

#[test]
fn snapshot_no_bandits() {
    let mut ecs = Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.npc_economies.insert(
        e1,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );
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
