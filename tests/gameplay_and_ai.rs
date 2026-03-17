// ===== ECS Core =====

#[test]
fn ecs_spawn_despawn_lifecycle() {
    use engene::core::ecs::Ecs;
    use engene::world::components::*;

    let mut ecs = Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    assert!(ecs.is_alive(e1));
    assert!(ecs.is_alive(e2));
    assert_eq!(ecs.alive.len(), 2);

    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Monster(MonsterSpecies::Wolf));
    assert_eq!(ecs.npcs().len(), 1);
    assert_eq!(ecs.monsters().len(), 1);
    assert_eq!(ecs.count_npcs(), 1);
    assert_eq!(ecs.count_species(MonsterSpecies::Wolf), 1);

    ecs.despawn(e1);
    assert!(!ecs.is_alive(e1));
    assert_eq!(ecs.npcs().len(), 0);
    assert_eq!(ecs.alive.len(), 1);
}

#[test]
fn ecs_spatial_rebuild() {
    use engene::core::ecs::Ecs;
    use engene::world::components::Transform;

    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(e, Transform { x: 100.0, y: 100.0, cell_x: 1, cell_y: 1 });
    ecs.rebuild_spatial();

    let candidates = ecs.spatial.candidates_in_radius(100.0, 100.0, 500.0);
    assert!(candidates.contains(&e));
}

// ===== World Components =====

#[test]
fn base_power_and_prey_relationships() {
    use engene::world::components::*;

    assert_eq!(base_power(&EntityKind::Monster(MonsterSpecies::Wolf)), 3.0);
    assert_eq!(base_power(&EntityKind::Npc), 5.0);
    assert_eq!(base_power(&EntityKind::Monster(MonsterSpecies::Boar)), 7.0);
    assert_eq!(base_power(&EntityKind::Monster(MonsterSpecies::Bloodsucker)), 10.0);

    assert!(is_prey_for(&EntityKind::Npc, &EntityKind::Monster(MonsterSpecies::Wolf)));
    assert!(!is_prey_for(&EntityKind::Monster(MonsterSpecies::Wolf), &EntityKind::Npc));
}

#[test]
fn life_info_stages_and_mating() {
    use engene::world::components::*;

    let young = LifeInfo::new_npc(400.0).with_age(40.0);
    assert_eq!(young.life_stage(), LifeStage::Young);
    assert_eq!(LifeStage::Young.speed_mult(), 1.2);
    assert_eq!(LifeStage::Young.learning_mult(), 1.5);

    let adult = LifeInfo::new_npc(400.0).with_age(200.0);
    assert_eq!(adult.life_stage(), LifeStage::Adult);

    let old = LifeInfo::new_npc(400.0).with_age(350.0);
    assert_eq!(old.life_stage(), LifeStage::Old);
    assert_eq!(LifeStage::Old.speed_mult(), 0.7);

    let monster = LifeInfo::new_monster(MonsterSpecies::Bloodsucker);
    assert_eq!(monster.max_age, 500.0);
    assert!(monster.can_mate(100));
}

#[test]
fn flammable_material_properties() {
    use engene::world::components::FlammableMaterial;

    assert!(FlammableMaterial::Cloth.flammability() > FlammableMaterial::Wood.flammability());
    assert_eq!(FlammableMaterial::Stone.flammability(), 0.0);
    assert!(FlammableMaterial::Wood.fuel() > FlammableMaterial::Cloth.fuel());
}

#[test]
fn ecosystem_needs_per_species() {
    use engene::world::components::*;

    let wolf = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    assert!(wolf.pack_following > 0.5);

    let boar = EcosystemNeeds::for_species(MonsterSpecies::Boar);
    assert!(boar.predator_avoidance > wolf.predator_avoidance);

    let bs = EcosystemNeeds::for_species(MonsterSpecies::Bloodsucker);
    assert!(bs.hunting > wolf.hunting);
}

// ===== Extension Components =====

#[test]
fn entity_tags_bit_operations() {
    use engene::world::extension_components::EntityTags;

    let mut tags = EntityTags::default();
    assert!(!tags.has_bit(3));
    tags.set_bit(3);
    assert!(tags.has_bit(3));
    tags.clear_bit(3);
    assert!(!tags.has_bit(3));

    tags.add_tag("hostile".into());
    assert!(tags.has_tag("hostile"));
    tags.add_tag("hostile".into());
    assert_eq!(tags.string_tags.len(), 1);
}

#[test]
fn attributes_crud() {
    use engene::world::extension_components::Attributes;

    let mut attrs = Attributes::default();
    attrs.set(1, 42.0);
    attrs.set(2, 100.0);
    assert_eq!(attrs.get(1), Some(42.0));

    attrs.set(1, 99.0);
    assert_eq!(attrs.get(1), Some(99.0));

    attrs.remove(1);
    assert_eq!(attrs.get(1), None);
}

#[test]
fn status_effects_stacking_and_decay() {
    use engene::world::extension_components::StatusEffects;

    let mut fx = StatusEffects::default();
    fx.add(1, 1, 5.0);
    assert!(fx.has(1));
    assert_eq!(fx.stacks(1), 1);

    fx.add(1, 2, 3.0);
    assert_eq!(fx.stacks(1), 3);

    fx.tick(4.0);
    assert!(fx.has(1));

    fx.tick(2.0);
    assert!(!fx.has(1));
}

#[test]
fn blackboard_typed_values() {
    use engene::world::extension_components::{Blackboard, Variant};

    let mut bb = Blackboard::default();
    bb.set("health".into(), Variant::Float(0.9));
    bb.set("level".into(), Variant::Int(5));
    bb.set("alive".into(), Variant::Bool(true));
    bb.set("name".into(), Variant::Str("Stalker".into()));

    assert_eq!(bb.get_float("health"), Some(0.9));
    assert_eq!(bb.get_int("level"), Some(5));
    assert_eq!(bb.get_bool("alive"), Some(true));
    assert_eq!(bb.get_str("name"), Some("Stalker"));
    assert_eq!(bb.get_float("missing"), None);
}

// ===== AI: Emotions =====

#[test]
fn emotions_dominant_and_decay() {
    use engene::game::ai::emotions::*;

    let mut emo = Emotions::new();
    assert_eq!(emo.dominant(), DominantEmotion::Calm);

    emo.anger = 0.8;
    assert_eq!(emo.dominant(), DominantEmotion::Anger);

    emo.decay(100.0);
    assert!(emo.anger < 0.8);
}

#[test]
fn emotions_mood_calculation() {
    use engene::game::ai::emotions::Emotions;

    let mut emo = Emotions::new();
    let base_mood = emo.mood();
    assert!((base_mood - 0.5).abs() < 0.01);

    emo.joy = 1.0;
    assert!(emo.mood() > base_mood);

    emo.joy = 0.0;
    emo.grief = 1.0;
    assert!(emo.mood() < base_mood);
}

#[test]
fn apply_npc_personality_modulates_emotions() {
    use engene::game::ai::emotions::*;
    use engene::world::components::NpcTraits;

    let traits = NpcTraits {
        bravery: 0.1, aggressiveness: 0.9, work_ethic: 0.5, curiosity: 0.5,
        honesty: 0.5, sociality: 0.8, autonomy: 0.5, materialism: 0.5,
        risk_tolerance: 0.5, stress_resistance: 0.0,
    };
    let mut emo = Emotions::new();
    apply_npc_personality(&mut emo, &traits, 0.5, 0.5, 0.5, 0.5);

    assert!(emo.anger > 0.3, "aggressive NPC should get anger boost");
    assert!(emo.fear > 0.3, "low-bravery NPC should feel fear");
}

#[test]
fn apply_monster_personality_modulates_emotions() {
    use engene::game::ai::emotions::*;
    use engene::world::components::MonsterTraits;

    let traits = MonsterTraits {
        aggressiveness: 0.9, caution: 0.8, territoriality: 0.5, bravery: 0.2,
        pack_mentality: 0.9, energy_level: 0.7, hoarding: 0.3, curiosity: 0.4,
        adaptability: 0.5, stress_tolerance: 0.1,
    };
    let mut emo = Emotions::new();
    apply_monster_personality(&mut emo, &traits, 0.5, 0.5, 0.5, 0.5);

    assert!(emo.anger > 0.3);
    assert!(emo.grief > 0.3, "high pack_mentality => grief from loss");
}

// ===== AI: Memory =====

#[test]
fn memory_event_recording_with_cap() {
    use engene::game::ai::memory::*;

    let mut mem = Memory::new();
    for i in 0..40 {
        mem.record_event(EventMemory {
            tick: i,
            kind: EventKind::Traded,
            location: (0, 0),
            other: None,
            emotional_impact: 0.1,
        });
    }
    assert!(mem.events.len() <= 30);
}

#[test]
fn memory_spatial_knowledge() {
    use engene::game::ai::memory::*;

    let mut mem = Memory::new();
    mem.mark_cell(5, 5, CellTag::Danger, 0.8);
    assert!((mem.cell_danger(5, 5) - 0.8).abs() < 0.01);
    assert_eq!(mem.cell_danger(0, 0), 0.0);

    mem.decay_spatial(0.5);
    assert!(mem.cell_danger(5, 5) < 0.8);
}

#[test]
fn memory_entity_opinions() {
    use engene::game::ai::memory::*;
    use engene::core::persistent_id::PersistentEntityId;

    let pid = PersistentEntityId(42);
    let mut mem = Memory::new();
    assert_eq!(mem.opinion_of(pid).trust, 0.0);

    mem.adjust_opinion(pid, |op| { op.trust = 0.8; op.familiarity = 0.5; });
    assert!((mem.opinion_of(pid).trust - 0.8).abs() < 0.01);
    assert_eq!(mem.best_ally(), Some(pid));

    mem.adjust_opinion(pid, |op| { op.trust = -0.5; });
    assert_eq!(mem.best_ally(), None);
}

#[test]
fn memory_lessons_learning() {
    use engene::game::ai::memory::*;

    let mut mem = Memory::new();
    mem.record_lesson(Lesson { action: LessonAction::SoloHunt, context: LessonContext::VsWolf, attempts: 5, successes: 4 });
    assert!((mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf) - 0.8).abs() < 0.01);

    mem.record_lesson(Lesson { action: LessonAction::SoloHunt, context: LessonContext::VsWolf, attempts: 5, successes: 1 });
    let score = mem.lesson_score(LessonAction::SoloHunt, LessonContext::VsWolf);
    assert!((score - 0.5).abs() < 0.01, "5/10 should give 0.5, got {score}");

    assert!((mem.lesson_score(LessonAction::Flee, LessonContext::General) - 0.5).abs() < 0.01);
}

#[test]
fn context_for_kind_mapping() {
    use engene::game::ai::memory::*;
    use engene::world::components::*;

    assert_eq!(context_for_kind(&EntityKind::Npc), LessonContext::VsNpc);
    assert_eq!(context_for_kind(&EntityKind::Monster(MonsterSpecies::Wolf)), LessonContext::VsWolf);
    assert_eq!(context_for_kind(&EntityKind::Monster(MonsterSpecies::Bloodsucker)), LessonContext::VsBloodsucker);
}

// ===== AI: Needs =====

#[test]
fn personal_needs_decay_increases_hunger() {
    use engene::game::ai::needs::*;
    use engene::world::components::PersonalNeeds;

    let mut needs = PersonalNeeds::default_npc();
    let initial_hunger = needs.hunger;
    decay_personal_needs(&mut needs, 10.0);
    assert!(needs.hunger > initial_hunger);
    assert!(needs.thirst > 0.2);
    assert!(needs.energy < 0.8);
}

#[test]
fn satisfy_needs_functions() {
    use engene::game::ai::needs::*;
    use engene::world::components::PersonalNeeds;

    let mut needs = PersonalNeeds::default_npc();
    needs.hunger = 0.8;
    satisfy_hunger(&mut needs, 0.5);
    assert!((needs.hunger - 0.3).abs() < 0.01);

    needs.thirst = 0.9;
    satisfy_thirst(&mut needs, 0.5);
    assert!((needs.thirst - 0.4).abs() < 0.01);

    needs.sleep = 0.7;
    satisfy_sleep(&mut needs, 0.3);
    assert!(needs.sleep < 0.7);
}

#[test]
fn damage_and_heal() {
    use engene::game::ai::needs::*;
    use engene::world::components::PersonalNeeds;

    let mut needs = PersonalNeeds::default_npc();
    take_damage(&mut needs, 0.3);
    assert!((needs.health - 0.7).abs() < 0.01);
    assert!(needs.fear > 0.0);

    heal(&mut needs, 0.2);
    assert!((needs.health - 0.9).abs() < 0.01);
}

#[test]
fn urgency_reflects_critical_needs() {
    use engene::game::ai::needs::*;
    use engene::world::components::PersonalNeeds;

    let mut needs = PersonalNeeds::default_npc();
    let low_urgency = urgency(&needs);

    needs.hunger = 0.95;
    assert!(urgency(&needs) > low_urgency);

    needs.hunger = 0.2;
    needs.health = 0.1;
    assert!(urgency(&needs) > 0.8);
}

// ===== AI: Body State =====

#[test]
fn body_state_healthy_adult() {
    use engene::game::ai::body::BodyState;
    use engene::world::components::PersonalNeeds;

    let pn = PersonalNeeds::default_npc();
    let body = BodyState::compute(&pn);
    assert!(body.move_speed_mult > 0.8);
    assert!(body.combat_power_mult > 0.7);
    assert!(body.work_efficiency_mult > 0.7);
}

#[test]
fn body_state_exhausted_old() {
    use engene::game::ai::body::BodyState;
    use engene::world::components::{PersonalNeeds, LifeStage};

    let pn = PersonalNeeds {
        hunger: 0.9, thirst: 0.9, sleep: 0.9, health: 0.3,
        energy: 0.1, fear: 0.8, curiosity: 0.0, ambitions: 0.0, discomfort: 0.8,
    };
    let body = BodyState::compute_with_stage(&pn, LifeStage::Old);
    assert!(body.move_speed_mult < 0.5);
    assert!(body.combat_power_mult < 0.5);
}

#[test]
fn is_night_and_time_of_day() {
    use engene::game::ai::body::{is_night, time_of_day_mult};

    assert!(is_night(0.1));
    assert!(is_night(0.8));
    assert!(!is_night(0.5));

    assert!(time_of_day_mult(0.1, true) > 1.0);
    assert!(time_of_day_mult(0.5, true) < 1.0);
    assert!(time_of_day_mult(0.1, false) < 1.0);
}

// ===== AI: Plan =====

#[test]
fn plan_expiration_and_target() {
    use engene::game::ai::plan::Plan;
    use engene::world::components::Goal;

    let mut plan = Plan::new(Goal::Hunt, Some((100.0, 200.0)), 0);
    assert!(!plan.is_expired());
    assert!(!plan.reached_target(0.0, 0.0));
    assert!(plan.reached_target(105.0, 200.0));

    for _ in 0..100 {
        plan.tick(1.0);
    }
    assert!(plan.is_expired());
}

#[test]
fn plan_durations_vary_by_goal() {
    use engene::game::ai::plan::Plan;
    use engene::world::components::Goal;

    let flee = Plan::new(Goal::Flee, None, 0);
    let explore = Plan::new(Goal::Explore, None, 0);
    assert!(flee.max_duration < explore.max_duration);
}

// ===== AI: Decision =====

#[test]
fn decide_npc_hungry_seeks_food() {
    use engene::game::ai::decision::decide_npc;
    use engene::world::components::*;

    let traits = NpcTraits {
        bravery: 0.5, aggressiveness: 0.3, work_ethic: 0.5, curiosity: 0.3,
        honesty: 0.5, sociality: 0.5, autonomy: 0.5, materialism: 0.5,
        risk_tolerance: 0.5, stress_resistance: 0.5,
    };
    let personal = PersonalNeeds {
        hunger: 0.95, thirst: 0.1, sleep: 0.1, health: 1.0,
        energy: 0.8, fear: 0.0, curiosity: 0.1, ambitions: 0.1, discomfort: 0.1,
    };
    let social = SocialNeeds::default();
    let economy = NpcEconomy { money: 100.0, monthly_required: 50.0, job: Job::Guard, desperation: 0.0 };

    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(goal == Goal::SeekFood || goal == Goal::Hunt, "hungry NPC should seek food or hunt, got {goal}");
}

#[test]
fn decide_monster_scared_flees() {
    use engene::game::ai::decision::decide_monster;
    use engene::world::components::*;

    let traits = MonsterTraits {
        aggressiveness: 0.1, caution: 0.9, territoriality: 0.1, bravery: 0.1,
        pack_mentality: 0.3, energy_level: 0.5, hoarding: 0.2, curiosity: 0.2,
        adaptability: 0.5, stress_tolerance: 0.2,
    };
    let personal = PersonalNeeds {
        hunger: 0.2, thirst: 0.1, sleep: 0.1, health: 0.3,
        energy: 0.5, fear: 0.95, curiosity: 0.0, ambitions: 0.0, discomfort: 0.5,
    };
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Boar);

    let goal = decide_monster(&traits, &personal, &eco);
    assert_eq!(goal, Goal::Flee, "scared cautious monster should flee");
}

// ===== Biome =====

#[test]
fn biome_properties() {
    use engene::world::biome::Biome;

    assert!(Biome::Forest.food_density() > Biome::Hills.food_density());
    assert!(Biome::Swamp.danger_level() > Biome::Settlement.danger_level());
    assert!(Biome::Swamp.water_density() > Biome::Plains.water_density());
    assert!(Biome::Swamp.night_danger_mult() > Biome::Settlement.night_danger_mult());
}

// ===== Ecosystem: Food Chain =====

#[test]
fn food_chain_relationships() {
    use engene::game::ecosystem::food_chain::*;
    use engene::world::components::MonsterSpecies;

    assert!(is_predator_of(MonsterSpecies::Wolf, MonsterSpecies::Boar));
    assert!(is_predator_of(MonsterSpecies::Bloodsucker, MonsterSpecies::Wolf));
    assert!(!is_predator_of(MonsterSpecies::Boar, MonsterSpecies::Wolf));

    assert_eq!(is_prey_of(MonsterSpecies::Boar), Some(MonsterSpecies::Wolf));
    assert_eq!(is_prey_of(MonsterSpecies::Bloodsucker), None);

    assert!(food_chain_rank(MonsterSpecies::Bloodsucker) > food_chain_rank(MonsterSpecies::Wolf));
    assert!(food_chain_rank(MonsterSpecies::Wolf) > food_chain_rank(MonsterSpecies::Boar));
}

// ===== Simulation Level =====

#[test]
fn simulation_level_distance_thresholds() {
    use engene::simulation::simulation_level::*;
    use engene::world::components::SimulationLevel;

    assert_eq!(level_for_distance(100.0), SimulationLevel::L0);
    assert_eq!(level_for_distance(300.0), SimulationLevel::L0);
    assert_eq!(level_for_distance(1000.0), SimulationLevel::L1);
    assert_eq!(level_for_distance(10000.0), SimulationLevel::L2);
    assert_eq!(level_for_distance(100000.0), SimulationLevel::L3);
}

#[test]
fn simulation_tick_intervals() {
    use engene::simulation::simulation_level::*;
    use engene::world::components::SimulationLevel;

    assert!(should_tick(SimulationLevel::L0, 1));
    assert!(should_tick(SimulationLevel::L0, 7));

    assert!(should_tick(SimulationLevel::L1, 12));
    assert!(!should_tick(SimulationLevel::L1, 7));

    assert!(should_tick(SimulationLevel::L2, 60));
    assert!(!should_tick(SimulationLevel::L2, 30));

    assert!(!should_tick(SimulationLevel::L3, 0));
    assert!(!should_tick(SimulationLevel::L3, 1000));
}

// ===== World: Fields =====

#[test]
fn wind_field_sampling() {
    use engene::world::fields::WindField;

    let wind = WindField::new();
    let sample = wind.sample(glam::Vec3::new(100.0, 0.0, 100.0), 0.0);
    assert!(sample.length() > 0.0);
}

#[test]
fn air_density_decreases_with_altitude() {
    use engene::world::fields::AirDensityField;

    let field = AirDensityField::new();
    let sea_level = field.sample(glam::Vec3::ZERO, 0.0);
    let high = field.sample(glam::Vec3::new(0.0, 5000.0, 0.0), 0.0);
    assert!(sea_level > high);
}

#[test]
fn rain_field_intensity() {
    use engene::world::fields::RainField;

    let mut rain = RainField::new();
    assert_eq!(rain.sample(glam::Vec3::ZERO, 0.0), 0.0);

    rain.set_rain(0.8, 0.5);
    assert!((rain.sample(glam::Vec3::ZERO, 0.0) - 0.4).abs() < 0.01);
}

#[test]
fn anomaly_field_forces() {
    use engene::world::fields::*;

    let mut field = AnomalyField::new();
    let outside = field.sample(glam::Vec3::new(1000.0, 0.0, 1000.0));
    assert!(!outside.inside_anomaly);

    field.zones.push(AnomalyZone {
        center: glam::Vec3::ZERO,
        radius: 50.0,
        force_strength: 10.0,
        force_type: AnomalyForceType::Gravity,
    });

    let inside = field.sample(glam::Vec3::new(10.0, 5.0, 0.0));
    assert!(inside.inside_anomaly);
    assert!(inside.acceleration.y < 0.0, "gravity anomaly should push down");
}

// ===== Sparse Set =====

#[test]
fn sparse_set_insert_get_remove() {
    use engene::core::sparse_set::SparseSet;

    let mut set = SparseSet::<String>::new();
    set.insert(5, "hello".into());
    set.insert(10, "world".into());

    assert_eq!(set.get(&5), Some(&"hello".to_string()));
    assert!(set.contains_key(&10));
    assert_eq!(set.len(), 2);

    set.remove(&5);
    assert!(!set.contains_key(&5));
    assert_eq!(set.len(), 1);
}
