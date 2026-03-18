use super::*;

// =============================================================================
// 2. GOALS / PLANS / DECISIONS (50 tests)
// =============================================================================

#[test]
fn decide_npc_hungry_seek_food() {
    let traits = NpcTraits {
        bravery: 0.1,
        aggressiveness: 0.05,
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
    assert_eq!(goal, Goal::Hunt);
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
        aggressiveness: 0.7,
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
    let mut personal = PersonalNeeds::default_monster();
    personal.hunger = 0.0;
    let mut eco = EcosystemNeeds::for_species(MonsterSpecies::Bloodsucker);
    eco.hunting = 0.0;
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
        work_ethic: 0.1,
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
        desperation: 0.0,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert_eq!(goal, Goal::Trade);
}

#[test]
fn decide_npc_steal_desperate() {
    let traits = NpcTraits {
        bravery: 0.1,
        aggressiveness: 0.3,
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
