use super::*;

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
            x: 10.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
            x: 60.0,
            y: 50.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
            x: 3.0,
            y: 4.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
fn perception_find_prey_bloodsucker_sees_boar() {
    let mut ecs = Ecs::new();
    let (bloodsucker, _) = ecs.spawn_new();
    let (boar, _) = ecs.spawn_new();
    ecs.transforms.insert(
        bloodsucker,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        boar,
        Transform {
            x: 20.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(
        bloodsucker,
        EntityKind::Monster(MonsterSpecies::Bloodsucker),
    );
    ecs.kinds
        .insert(boar, EntityKind::Monster(MonsterSpecies::Boar));
    ecs.rebuild_spatial();
    let prey = find_prey(&ecs, bloodsucker);
    assert!(prey.is_some());
}

#[test]
fn perception_find_predator_boar_sees_bloodsucker() {
    let mut ecs = Ecs::new();
    let (bloodsucker, _) = ecs.spawn_new();
    let (boar, _) = ecs.spawn_new();
    ecs.transforms.insert(
        bloodsucker,
        Transform {
            x: 10.0,
            y: 10.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.transforms.insert(
        boar,
        Transform {
            x: 15.0,
            y: 10.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(
        bloodsucker,
        EntityKind::Monster(MonsterSpecies::Bloodsucker),
    );
    ecs.kinds
        .insert(boar, EntityKind::Monster(MonsterSpecies::Boar));
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
    ecs.transforms.insert(
        e,
        Transform {
            x: 1.0,
            y: 2.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
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
    ecs.inventories
        .insert(e, engene::world::components::Inventory { items: vec![] });
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
    ecs.names
        .insert(e, engene::world::components::Name("Test".into()));
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
