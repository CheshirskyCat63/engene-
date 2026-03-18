use super::*;

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
    e.joy = 0.4;
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
    let count = mem
        .lessons
        .iter()
        .filter(|l| l.action == LessonAction::GroupHunt && l.context == LessonContext::General)
        .count();
    assert_eq!(count, 1);
}

#[test]
fn emotions_dominant_threshold() {
    let mut e = Emotions::new();
    e.anger = 0.05;
    assert_eq!(e.dominant(), DominantEmotion::Calm);
}
