#[test]
fn quest_system_generates_quest() {
    use engene::game::gameplay::quest_system::QuestSystem;

    let _quest_system = QuestSystem::new();
}

#[test]
fn quest_board_tracking() {
    use engene::tools::quest_board::{QuestBoardEntry, QuestBoardPanel};

    let mut panel = QuestBoardPanel::new();
    panel.entries.push(QuestBoardEntry {
        quest_id: 1,
        quest_type: "KillMonsters".into(),
        status: "Active".into(),
        giver_name: "Trader".into(),
        assignee_name: Some("Stalker".into()),
        progress: "0/3".into(),
        reward: 100.0,
    });
    panel.entries.push(QuestBoardEntry {
        quest_id: 2,
        quest_type: "FetchArtifact".into(),
        status: "Active".into(),
        giver_name: "Trader".into(),
        assignee_name: None,
        progress: "0/1".into(),
        reward: 50.0,
    });
    panel.entries.push(QuestBoardEntry {
        quest_id: 3,
        quest_type: "ScoutArea".into(),
        status: "Completed".into(),
        giver_name: "Trader".into(),
        assignee_name: Some("Stalker".into()),
        progress: "1/1".into(),
        reward: 75.0,
    });
    panel.total_generated = 5;
    panel.total_completed = 2;
    panel.total_failed = 1;
    panel.total_expired = 0;

    assert_eq!(panel.active_count(), 2);
    assert!((panel.completion_rate() - 2.0 / 3.0).abs() < 0.01);
}

#[test]
fn faction_relations_default() {
    use engene::game::gameplay::factions::{Faction, FactionRelations, FactionStance};

    let rel = FactionRelations::new();
    assert_eq!(
        rel.stance_between(Faction::Loners, Faction::Loners),
        FactionStance::Allied
    );
    assert_eq!(
        rel.stance_between(Faction::Duty, Faction::Freedom),
        FactionStance::Hostile
    );
    assert_eq!(
        rel.stance_between(Faction::Loners, Faction::Duty),
        FactionStance::Neutral
    );
    assert_eq!(
        rel.stance_between(Faction::Bandits, Faction::Loners),
        FactionStance::Suspicious
    );
}

#[test]
fn faction_hostility_check() {
    use engene::game::gameplay::factions::{Faction, FactionRelations};

    let mut rel = FactionRelations::new();
    assert!(!rel.is_hostile(Faction::Loners, Faction::Duty));

    rel.modify_relation(Faction::Loners, Faction::Duty, -2);
    assert!(rel.is_hostile(Faction::Loners, Faction::Duty));

    assert!(rel.is_hostile(Faction::Duty, Faction::Freedom));
    assert!(!rel.is_hostile(Faction::Loners, Faction::Scientists));
}

#[test]
fn faction_membership_component() {
    use engene::world::components::{Faction, FactionMembership};

    let membership = FactionMembership::default();
    assert_eq!(membership.faction, Faction::Loners);
    assert_eq!(membership.standing, 0.0);
}

#[test]
fn equipment_slots_default_stalker() {
    use engene::world::components::EquipmentSlots;

    let eq = EquipmentSlots::default_stalker();
    assert!((eq.weapon_condition - 0.8).abs() < 0.01);
    assert!((eq.armor_condition - 0.7).abs() < 0.01);
    assert_eq!(eq.medkits, 2);
    assert_eq!(eq.food_rations, 3);
    assert_eq!(eq.ammo, 30);
    assert!((eq.total_weight - 15.0).abs() < 0.01);
}

#[test]
fn equipment_degradation() {
    use engene::world::components::EquipmentSlots;

    let mut eq = EquipmentSlots::default_stalker();
    let init_weapon = eq.weapon_condition;
    let init_armor = eq.armor_condition;

    eq.degrade_combat();
    assert!(eq.weapon_condition < init_weapon);
    assert!(eq.armor_condition < init_armor);
    assert!((eq.weapon_condition - (init_weapon - 0.05)).abs() < 0.01);
    assert!((eq.armor_condition - (init_armor - 0.03)).abs() < 0.01);

    for _ in 0..30 {
        eq.degrade_combat();
    }
    assert_eq!(eq.weapon_condition, 0.0);
    assert!(eq.armor_condition >= 0.0);
}

#[test]
fn equipment_repair_cost() {
    use engene::world::components::EquipmentSlots;

    let mut eq = EquipmentSlots::default_stalker();
    eq.weapon_condition = 0.5;
    eq.armor_condition = 0.4;

    let cost = eq.repair_cost();
    assert!(cost > 0.0);
    assert!((cost - (0.5 * 30.0 + 0.6 * 20.0)).abs() < 0.01);
}
